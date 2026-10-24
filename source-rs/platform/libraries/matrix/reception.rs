//! Consumer-side pointer edges for a supplied interaction binding. Call once per
//! received event, before dispatching it to widgets; application state is separate.
use cathedral_platform_contracts::matrix as wire;

#[derive(Clone, Copy, Default)]
struct Pointer {
    name: u32,
    buttons: u8,
}

#[derive(Default)]
pub struct Pointers {
    epoch: u64,
    streams: [Pointer; wire::INPUT_STREAMS],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Buttons {
    pub held: u8,
    pub pressed: u8,
    pub released: u8,
}
impl Buttons {
    pub fn left_pressed(self) -> bool {
        self.pressed & 1 != 0
    }
}

impl Pointers {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    /// Consume NEXT / poll_event's packet, not the packed POLL response. Names
    /// are scoped to its view epoch, never global identities or array indices.
    /// CANCEL ends this receiver's hold; the sender filters residual releases.
    pub fn update(&mut self, event: wire::Packet) -> Option<Buttons> {
        if event[0] != 0 || event[1] == 0 {
            self.reset();
            return None;
        }
        if self.epoch != event[1] {
            self.reset();
            self.epoch = event[1];
        }
        if !matches!(event[2], wire::POINTER | wire::CANCEL) {
            return None;
        }
        let name = u32::try_from(event[6]).ok().filter(|&name| name != 0)?;
        let existing = self.streams.iter().position(|p| p.name == name);
        if event[2] == wire::CANCEL {
            if let Some(index) = existing {
                self.streams[index].buttons = 0;
            }
            return None;
        }
        let buttons = u8::try_from(event[5]).ok().filter(|&b| b <= 7)?;
        let index = existing.or_else(|| self.streams.iter().position(|p| p.name == 0))?;
        let prior = self.streams[index].buttons;
        self.streams[index] = Pointer { name, buttons };
        Some(Buttons {
            held: buttons,
            pressed: buttons & !prior,
            released: prior & !buttons,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event(name: u64, buttons: u64) -> wire::Packet {
        [0, 1, wire::POINTER, 10, 20, buttons, name, 0]
    }
    #[test]
    fn independent_holds_and_cancellation_do_not_manufacture_clicks() {
        let mut p = Pointers::default();
        assert!(p.update(event(19, 1)).unwrap().left_pressed());
        assert!(p.update(event(73, 1)).unwrap().left_pressed());
        p.update([0, 1, wire::CANCEL_KEYS, 0, 0, 0, 73, 1]);
        assert!(!p.update(event(73, 1)).unwrap().left_pressed());
        p.update([0, 1, wire::CANCEL, 0, 0, 0, 19, 0]);
        assert!(!p.update(event(73, 1)).unwrap().left_pressed());
        assert!(p.update(event(19, 1)).unwrap().left_pressed());
        assert_eq!(p.update(event(73, 0)).unwrap().released, 1);
        assert!(!p.update(event(19, 1)).unwrap().left_pressed());
    }
    #[test]
    fn bindings_reset_without_treating_metadata_as_a_press() {
        let mut p = Pointers::default();
        p.update(event(19, 3));
        assert_eq!(p.update([0, 2, wire::NONE, 0, 0, 0, 0, 0]), None);
        let mut fresh = event(19, 2);
        fresh[1] = 2;
        assert_eq!(p.update(fresh).unwrap().pressed, 2);
        assert_eq!(p.update([u64::MAX, 0, 0, 0, 0, 0, 0, 0]), None);
        assert_eq!(p.update(fresh).unwrap().pressed, 2);
    }
    #[test]
    fn malformed_or_excess_streams_cannot_mutate_another_hold() {
        let mut p = Pointers::default();
        for name in 1..=wire::INPUT_STREAMS as u64 {
            assert!(p.update(event(name * 19, 1)).unwrap().left_pressed());
        }
        for name in [0, 999, u64::MAX] {
            assert_eq!(p.update(event(name, 0)), None);
        }
        assert_eq!(p.update(event(19, 8)), None);
        assert!(!p.update(event(19, 1)).unwrap().left_pressed());
    }
}
