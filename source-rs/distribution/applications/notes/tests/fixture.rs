//! Controlled peers execute the real Omega export; no Rust editor state mirror.
use super::code::Code;
use cathedral_omega_app::{Instance, Step};
use cathedral_platform_contracts::matrix as wire;
use std::{collections::VecDeque, vec, vec::Vec};
include!(env!("OMEGA_APP_CONSTANTS"));
pub(super) const ERROR: u64 = (-5i64) as u64;
pub(super) fn with_app(run: impl FnOnce(&mut Peer<'_>)) {
    let code = Code::new(BYTES);
    // SAFETY: Validated ABI 1 export remains RX-mapped until every borrow drops.
    let step: Step = unsafe { core::mem::transmute(code.address.add(ENTRY)) };
    let mut state = [0; 512];
    let mut packet = vec![0u64; 4096];
    // SAFETY: Live code and disjoint exclusively borrowed buffers.
    let app = unsafe { Instance::new(step, &mut state, packet.as_mut_slice().try_into().unwrap()) }
        .unwrap();
    run(&mut Peer {
        app,
        records: vec![],
        commands: vec![],
        images: vec![],
        replies: vec![],
        events: VecDeque::new(),
        staged: vec![],
        epoch: 7,
        frames: true,
        demand: false,
        save_error: false,
        commit_before_error: false,
        temporary: false,
        exited: false,
        next_surface: 500,
        storage_requests: 0,
        truncated_bulk: false,
    });
}
pub(super) struct Peer<'a> {
    pub app: Instance<'a>,
    pub records: Vec<(u64, Vec<u8>)>,
    pub commands: Vec<[u64; 8]>,
    pub images: Vec<([u64; 8], Vec<u8>)>,
    pub replies: Vec<[u64; 8]>,
    pub events: VecDeque<[u64; 8]>,
    staged: Vec<u8>,
    pub epoch: u64,
    pub frames: bool,
    pub demand: bool,
    pub save_error: bool,
    pub commit_before_error: bool,
    pub temporary: bool,
    pub exited: bool,
    next_surface: u64,
    pub storage_requests: usize,
    pub truncated_bulk: bool,
}
impl Peer<'_> {
    pub fn pump(&mut self) {
        for _ in 0..256 {
            let op = self.app.advance();
            let p = &mut self.app.packet;
            let q: [u64; 8] = p[..8].try_into().unwrap();
            let mut payload: Vec<u8> = p[16..]
                .iter()
                .flat_map(|n| n.to_le_bytes())
                .take(p[8] as usize)
                .collect();
            let limit = p[11];
            p[9] = 0;
            p[8] = 0;
            p[11] = 0;
            match op {
                0 => {
                    self.exited = true;
                    return;
                }
                1 | 2 => {
                    assert_eq!(&q[..2], &[1, 1]);
                    if op == 2 && self.temporary {
                        p[9] = cathedral_platform_contracts::services::UNAVAILABLE;
                    }
                }
                3 => {
                    assert!(!self.temporary);
                    p[..8].fill(0);
                    p[1] = 901;
                }
                4 => {
                    assert!(!self.temporary);
                    self.storage_requests += 1;
                    self.commands.push(q);
                    p[..8].fill(0);
                    match q[0] {
                        6 => {
                            assert_eq!(q[1], 901);
                            let start = if q[3] == 0 {
                                0
                            } else {
                                (q[3] - 1000 + 1) as usize
                            };
                            let count = (self.records.len() - start).min(6);
                            p[1] = count as u64;
                            for i in 0..count {
                                p[2 + i] = 1000 + (start + i) as u64;
                            }
                        }
                        1 | 7 => {
                            let record = &self.records[(q[1] - 1000) as usize];
                            p[1] = q[1];
                            p[2] = record.0;
                            p[3] = record.1.len() as u64;
                            if q[0] == 7 {
                                assert_eq!(q[2], record.0);
                                assert_eq!(limit, 244);
                                assert_eq!(q[4], p[3]);
                                let n = record.1.len() - usize::from(self.truncated_bulk);
                                p[8] = n as u64;
                                p[4] = n as u64;
                                for (i, chunk) in record.1[..n].chunks(8).enumerate() {
                                    let mut word = [0u8; 8];
                                    word[..chunk.len()].copy_from_slice(chunk);
                                    p[16 + i] = u64::from_le_bytes(word);
                                }
                            }
                        }
                        3 => {
                            assert_eq!(q[4] as usize, payload.len());
                            if !self.save_error || self.commit_before_error {
                                let index = if q[1] == 0 {
                                    assert_eq!(q[3], 901);
                                    self.records.push((0, vec![]));
                                    self.records.len() - 1
                                } else {
                                    (q[1] - 1000) as usize
                                };
                                assert_eq!(q[2], self.records[index].0);
                                self.records[index].0 += 1;
                                self.records[index].1 = payload;
                                p[1] = index as u64 + 1000;
                                p[2] = self.records[index].0;
                            }
                            if self.save_error {
                                p[9] = ERROR;
                            }
                        }
                        4 => {
                            assert_eq!(q[3], 901);
                            let record = self.records[(q[1] - 1000) as usize].clone();
                            assert_eq!(q[2], record.0);
                            self.records.push((1, record.1));
                            p[1] = 999 + self.records.len() as u64;
                            p[2] = 1;
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
                                if self.frames { 0 } else { wire::NO_FRAMES }
                                    | if self.demand {
                                        wire::FRAME_REQUESTED
                                    } else {
                                        0
                                    },
                                event[2]
                                    | (event[5] << 8)
                                    | (event[6] << 32)
                                    | (event[7] << 17)
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
                        wire::PAINT => (),
                        wire::SURFACE_IMAGE => {
                            if !self.staged.is_empty() {
                                assert!(payload.is_empty());
                                payload = core::mem::take(&mut self.staged);
                            }
                            assert_eq!(payload.len() as u64, q[6] * q[7] * 4);
                            self.images.push((q, payload));
                        }
                        wire::FRAME_DEMAND => p[2] = 99,
                        wire::FRAME_READY => assert_eq!(q[2], 99),
                        wire::CURSOR_PREFERENCE => (),
                        other => panic!("unrequested interaction operation {other}"),
                    }
                }
                6 => return,
                7 => self.replies.push(q),
                8 => (),
                9 => {
                    assert_eq!(q[0], 46080);
                    assert_eq!(q[1] as usize, self.staged.len());
                    assert!(self.staged.len() + payload.len() <= 46080);
                    self.staged.extend_from_slice(&payload);
                }
                other => panic!("unknown host operation {other}"),
            }
        }
        panic!("app did not yield to wait");
    }
    pub fn wake(&mut self) {
        self.app.packet[9] = 0;
        self.pump();
    }
    pub fn event(&mut self, kind: u64, x: u64, y: u64, buttons: u64, channel: u64, group: u64) {
        self.events
            .push_back([0, self.epoch, kind, x, y, buttons, channel, group]);
        self.wake();
    }
    pub fn key(&mut self, key: u64) {
        self.event(wire::KEY, key, 1, 0, 17, 1);
    }
    pub fn text(&mut self, text: &str) {
        for ch in text.bytes() {
            self.key((ch - b'a') as u64 + 15);
        }
    }
    pub fn status(&mut self) -> [u64; 8] {
        self.app.packet[9] = 1;
        self.app.packet[0] = 0;
        self.pump();
        *self.replies.last().unwrap()
    }
    pub fn loss(&mut self) -> u64 {
        self.app.packet[9] = 1;
        self.app.packet[0] = wire::LOSS;
        self.pump();
        self.replies.last().unwrap()[1]
    }
    pub fn click(&mut self, x: u64, y: u64, channel: u64) {
        self.event(wire::POINTER, x, y, 1, channel, 1);
    }
    pub fn text_image(&self) -> &[u8] {
        &self
            .images
            .iter()
            .rev()
            .find(|(q, _)| q[5] == 180 && q[6] == 64)
            .unwrap()
            .1
    }
}
