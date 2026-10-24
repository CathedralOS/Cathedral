//! Replay the real compiled app against controlled protocol peers. No Rust
//! copy of app state: observations are IPC requests, pixels and durable bytes.
use super::code::Code;
use cathedral_omega_app::{Instance, Step};
use cathedral_platform_contracts::matrix as wire;
use std::{collections::VecDeque, vec, vec::Vec};
include!(env!("OMEGA_APP_CONSTANTS"));
pub(super) const BLUE: u64 = 0x315ca8;
pub(super) const GOLD: u64 = 0xd18b35;
pub(super) const ERROR: u64 = (-5i64) as u64;
pub(super) fn with_app(run: impl FnOnce(&mut Peer<'_>)) {
    let code = Code::new(BYTES);
    // SAFETY: Packaging validated this pure export, RX-mapped until after app
    // and every borrowed buffer have dropped. Queries and step implement ABI 1.
    let step: Step = unsafe { core::mem::transmute(code.address.add(ENTRY)) };
    let mut state = [0; 512];
    let mut packet = vec![0u64; 4096];
    // SAFETY: Same live export, disjoint exclusive buffers of the required size.
    let app = unsafe { Instance::new(step, &mut state, packet.as_mut_slice().try_into().unwrap()) }
        .unwrap();
    let mut peer = Peer {
        app,
        record: None,
        revision: 0,
        commands: vec![],
        images: vec![],
        replies: vec![],
        paints: vec![],
        events: VecDeque::new(),
        epoch: 7,
        frames: true,
        next_surface: 500,
        save_error: false,
        commit_before_error: false,
        create_error: false,
        exited: false,
        acquired: vec![],
    };
    run(&mut peer);
}
pub(super) struct Peer<'a> {
    pub app: Instance<'a>,
    pub record: Option<Vec<u8>>,
    pub revision: u64,
    pub commands: Vec<[u64; 8]>,
    pub images: Vec<([u64; 8], Vec<u8>)>,
    pub replies: Vec<[u64; 8]>,
    pub paints: Vec<[u64; 8]>,
    pub events: VecDeque<[u64; 8]>,
    pub epoch: u64,
    pub frames: bool,
    next_surface: u64,
    pub save_error: bool,
    pub commit_before_error: bool,
    pub create_error: bool,
    pub exited: bool,
    pub acquired: Vec<u64>,
}
impl Peer<'_> {
    pub fn pump(&mut self) {
        for _ in 0..128 {
            let op = self.app.advance();
            let p = &mut self.app.packet;
            let q: [u64; 8] = p[..8].try_into().unwrap();
            let payload: Vec<u8> = p[16..]
                .iter()
                .flat_map(|n| n.to_le_bytes())
                .take(p[8] as usize)
                .collect();
            p[9] = 0;
            match op {
                0 => {
                    self.exited = true;
                    return;
                }
                1 | 2 => {
                    assert_eq!(&q[..2], &[1, 1]);
                    self.acquired.push(op);
                }
                3 => {
                    p[..8].fill(0);
                    p[1] = 901;
                }
                4 => {
                    self.commands.push(q);
                    p[..8].fill(0);
                    match q[0] {
                        6 => {
                            assert_eq!(q[1], 901);
                            p[1] = u64::from(self.record.is_some());
                            p[2] = 902;
                        }
                        3 => {
                            assert_eq!(q[4], 6);
                            assert_eq!(payload.len(), 6);
                            let creating = q[1] == 0;
                            if creating {
                                assert_eq!(q[3], 901);
                                assert!(self.record.is_none());
                            } else {
                                assert_eq!(q[1], 902);
                                assert_eq!(q[2], self.revision);
                            }
                            if !self.save_error || self.commit_before_error || creating {
                                self.record = Some(payload);
                                self.revision += 1;
                            }
                            p[0] = if self.save_error || (creating && self.create_error) {
                                ERROR
                            } else {
                                0
                            };
                            p[1] = 902;
                            p[2] = self.revision;
                            p[3] = 6;
                        }
                        2 => {
                            assert_eq!(q[1], 902);
                            assert_eq!(q[4], 6);
                            p[1] = 902;
                            p[2] = self.revision;
                            let bytes = self.record.as_ref().unwrap();
                            p[3] = bytes.len() as u64;
                            let mut small = [0u8; 8];
                            small[..bytes.len()].copy_from_slice(bytes);
                            p[4] = u64::from_le_bytes(small);
                        }
                        other => panic!("unrequested storage operation {other}"),
                    }
                }
                5 => {
                    self.commands.push(q);
                    p[..8].fill(0);
                    match q[0] {
                        wire::POLL => {
                            let event = self.events.pop_front().unwrap_or([0; 8]);
                            p[..8].copy_from_slice(&[
                                0,
                                self.epoch,
                                320,
                                200,
                                if self.frames { 0 } else { wire::NO_FRAMES },
                                event[2]
                                    | (event[5] << 8)
                                    | (event[6] << 32)
                                    | if self.events.is_empty() {
                                        0
                                    } else {
                                        wire::MORE
                                    },
                                event[3],
                                event[4],
                            ]);
                        }
                        wire::SURFACE_CREATE => {
                            assert_eq!(q[1], self.epoch);
                            p[1] = self.next_surface;
                            self.next_surface += 1;
                        }
                        wire::PRESENT_RECTS => {
                            let (rects, count) =
                                wire::rectangle_updates(q).expect("platform-compatible packet");
                            assert_eq!(count, 2);
                            self.paints.extend(rects);
                        }
                        wire::SURFACE_IMAGE => {
                            assert_eq!(payload.len() as u64, q[6] * q[7] * 4);
                            self.images.push((q, payload));
                        }
                        other => panic!("unrequested interaction operation {other}"),
                    }
                }
                6 => return,
                7 => self.replies.push(q),
                8 => (),
                other => panic!("unknown host operation {other}"),
            }
        }
        panic!("app failed to yield to wait within bounded replay");
    }
    pub fn event(&mut self, kind: u64, x: u64, y: u64, buttons: u64, channel: u64) {
        self.events
            .push_back([0, self.epoch, kind, x, y, buttons, channel, 1]);
        self.wake();
    }
    pub fn enter(&mut self) {
        self.event(wire::KEY, 5, 1, 0, 17);
    }
    pub fn wake(&mut self) {
        self.app.packet[9] = 0;
        self.pump();
    }
    pub fn loss(&mut self) -> u64 {
        self.app.packet[9] = 1;
        self.app.packet[0] = wire::LOSS;
        self.pump();
        self.replies.last().unwrap()[1]
    }
    pub fn color(&self) -> u64 {
        self.paints[self.paints.len() - 2][7]
    }
    pub fn click(&mut self, channel: u64) {
        self.event(wire::POINTER, 310, 8, 1, channel);
    }
}
