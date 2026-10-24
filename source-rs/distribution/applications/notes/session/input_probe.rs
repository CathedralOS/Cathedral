//! Disposable wire observer: track application-visible holds, reject stale name
//! reuse and log lifecycle evidence. F8 deliberately stalls only this consumer.
use super::*;
#[derive(Clone, Copy, Default)]
struct Channel {
    name: u64,
    group: u64,
    keys: u64,
    buttons: u64,
}
#[derive(Default)]
pub struct Receiver {
    epoch: u64,
    retired: [u64; 32],
    retired_groups: [u64; 16],
    channels: [Channel; wire::INPUT_STREAMS],
}
impl Receiver {
    pub fn observe(&mut self, packet: wire::Packet) {
        let [status, epoch, kind, key, state, buttons, name, group] = packet;
        if status != 0 || epoch == 0 {
            return;
        }
        if self.epoch != epoch {
            *self = Self {
                epoch,
                ..Self::default()
            };
        }
        if kind == wire::GROUP_RETIRED {
            assert!(
                self.channels
                    .iter()
                    .all(|c| c.name == 0 || c.group != group)
            );
            assert!(!self.retired_groups.contains(&group));
            *self
                .retired_groups
                .iter_mut()
                .find(|g| **g == 0)
                .expect("probe group history capacity") = group;
            time::storage_sample("receiver-group-retired", &[epoch, group]);
            return;
        }
        if kind == wire::NONE {
            return;
        }
        assert_ne!(name, 0);
        let index = self
            .channels
            .iter()
            .position(|c| c.name == name)
            .unwrap_or_else(|| {
                assert!(!self.retired.contains(&name), "retired channel name reused");
                assert!(
                    !self.retired_groups.contains(&group),
                    "retired group name reused"
                );
                let index = self
                    .channels
                    .iter()
                    .position(|c| c.name == 0)
                    .expect("receiver channel capacity");
                self.channels[index] = Channel {
                    name,
                    group,
                    ..Channel::default()
                };
                index
            });
        let channel = &mut self.channels[index];
        assert_eq!(channel.group, group);
        time::storage_sample(
            "receiver-input",
            &[
                epoch,
                kind,
                name,
                group,
                channel.keys,
                channel.buttons,
                key,
                state,
            ],
        );
        match kind {
            wire::POINTER => channel.buttons = buttons,
            wire::KEY => {
                let bit = 1 << (key - 1);
                if state == input::RELEASE as u64 {
                    assert_ne!(channel.keys & bit, 0, "orphan release after interruption");
                    channel.keys &= !bit;
                } else {
                    if state == input::REPEAT as u64 {
                        assert_ne!(channel.keys & bit, 0, "orphan repeat after interruption");
                    }
                    channel.keys |= bit;
                }
            }
            wire::CANCEL => {
                channel.keys = 0;
                channel.buttons = 0;
            }
            wire::CANCEL_KEYS => channel.keys = 0,
            wire::RETIRED => {
                *self
                    .retired
                    .iter_mut()
                    .find(|n| **n == 0)
                    .expect("probe channel history capacity") = name;
                *channel = Channel::default();
            }
            _ => panic!("unknown receiver event"),
        }
        if kind == wire::KEY && key == input::F8 as u64 && state == input::PRESS as u64 {
            cathedral_user_runtime::write(b"Cathedral receiver probe: stalled\n").unwrap();
            assert_eq!(
                cathedral_user_runtime::event::wait(&[], time::after(300).unwrap()),
                Err(Error(abi::TIMED_OUT as i64))
            );
            cathedral_user_runtime::write(b"Cathedral receiver probe: resumed\n").unwrap();
        }
    }
}
