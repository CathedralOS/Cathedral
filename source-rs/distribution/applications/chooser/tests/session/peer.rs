// Real compiled chooser, supplied with deterministic IPC peers.
use super::code::Code;
use cathedral_omega_app::{Instance, Step};
use cathedral_platform_contracts::{
    matrix as input,
    storage::{self, Entry, metadata::Metadata},
};
use std::{collections::VecDeque, vec, vec::Vec};
include!(env!("OMEGA_CHOOSER_APP_CONSTANTS"));
const ERROR: u64 = (-5i64) as u64;
fn with_app(picker: bool, run: impl FnOnce(&mut Peer<'_>)) {
    let code = Code::new(BYTES);
    // SAFETY: Validated ABI export stays RX-mapped for the entire instance.
    let step: Step = unsafe { core::mem::transmute(code.address.add(ENTRY)) };
    let mut state = [0; 512];
    let mut packet = vec![0u64; 4096];
    // SAFETY: Live code and disjoint exclusive buffers of the declared sizes.
    let app = unsafe { Instance::new(step, &mut state, packet.as_mut_slice().try_into().unwrap()) }
        .unwrap();
    app.packet[12] = (2 << 8) | u64::from(picker);
    let mut config = vec![0; 104];
    config[..8].copy_from_slice(b"CTHMAT02");
    config[8..15].copy_from_slice(b"Chooser");
    for (at, value) in [
        (40, 48u64),
        (48, 48),
        (56, 96),
        (64, 8),
        (72, 80),
        (80, 8),
        (88, 16),
    ] {
        config[at..at + 8].copy_from_slice(&value.to_le_bytes());
    }
    run(&mut Peer {
        app,
        entries: vec![
            entry(2, "Chooser"),
            entry(3, "Notes"),
            entry(4, "Colors"),
            entry(5, "Background"),
        ],
        commands: vec![],
        storage_commands: vec![],
        images: vec![],
        replies: vec![],
        events: VecDeque::new(),
        staged: vec![],
        epoch: 7,
        width: 1024,
        height: if picker { 768 } else { 80 },
        next_surface: 500,
        now: 0,
        managed: 0,
        tasks: [[0; 8]; 3],
        config,
        revision: 1,
        config_object: 90,
        fail_save: false,
        commit_before_error: false,
        malformed_entries: false,
        exited: false,
        race: false,
        refresh: 0,
    });
}
fn entry(object: u64, name: &str) -> Entry {
    Entry {
        object,
        revision: 1,
        metadata: Metadata::new(name.as_bytes(), storage::metadata::EXECUTABLE).unwrap(),
    }
}
struct Peer<'a> {
    app: Instance<'a>,
    entries: Vec<Entry>,
    commands: Vec<[u64; 8]>,
    storage_commands: Vec<[u64; 8]>,
    images: Vec<([u64; 8], Vec<u8>)>,
    replies: Vec<[u64; 8]>,
    events: VecDeque<[u64; 8]>,
    staged: Vec<u8>,
    epoch: u64,
    width: u64,
    height: u64,
    next_surface: u64,
    now: u64,
    managed: u64,
    tasks: [[u64; 8]; 3],
    config: Vec<u8>,
    revision: u64,
    config_object: u64,
    fail_save: bool,
    commit_before_error: bool,
    malformed_entries: bool,
    exited: bool,
    race: bool,
    refresh: u64,
}
impl Peer<'_> {
    fn pump(&mut self) {
        for _ in 0..1024 {
            let op = self.app.advance();
            let p = &mut self.app.packet;
            let q: [u64; 8] = p[..8].try_into().unwrap();
            let payload: Vec<u8> = p[16..]
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
                1 | 2 => assert_eq!(&q[..2], &[1, 1]),
                3 => {
                    p[..8].fill(0);
                    p[1] = 1;
                }
                4 => {
                    self.storage_commands.push(q);
                    p[..8].fill(0);
                    match q[0] {
                        storage::ENTRIES => {
                            assert_eq!(q[1], 1);
                            assert_eq!(limit, 4080);
                            let rows: Vec<_> = self
                                .entries
                                .iter()
                                .filter(|e| e.object > q[3])
                                .take(51)
                                .collect();
                            p[1] = rows.len() as u64;
                            p[2] = rows.last().map_or(q[3], |e| e.object);
                            let mut bytes: Vec<_> = rows.iter().flat_map(|e| e.encode()).collect();
                            if self.malformed_entries {
                                bytes.pop();
                            }
                            payload_reply(p, &bytes);
                        }
                        storage::LOOKUP => {
                            assert_eq!(&payload, b"Matrix");
                            p[1] = self.config_object;
                        }
                        storage::HEAD => {
                            p[1] = q[1];
                            p[2] = self.revision;
                            p[3] = self.config.len() as u64;
                        }
                        storage::READ => {
                            assert_eq!(q[1], self.config_object);
                            assert_eq!(q[2], self.revision);
                            p[1] = q[1];
                            p[2] = self.revision;
                            p[3] = 104;
                            let start = q[3] as usize;
                            let n = q[4] as usize;
                            for (i, chunk) in self.config[start..start + n].chunks(8).enumerate() {
                                let mut bytes = [0; 8];
                                bytes[..chunk.len()].copy_from_slice(chunk);
                                p[4 + i] = u64::from_le_bytes(bytes);
                            }
                        }
                        storage::PUBLISH => {
                            assert_eq!(q[1], self.config_object);
                            assert_eq!(q[2], self.revision);
                            assert_eq!(payload.len(), 104);
                            if !self.fail_save || self.commit_before_error {
                                self.config = payload;
                                self.revision += 1;
                            }
                            if self.fail_save {
                                p[9] = ERROR;
                            } else {
                                p[1] = q[1];
                                p[2] = self.revision;
                            }
                        }
                        storage::COPY_TREE => {
                            assert_eq!(q[3], 1);
                            assert_eq!(&payload, b"User 1");
                            let id = self.entries.last().unwrap().object + 1;
                            self.entries.push(entry(id, "User 1"));
                            p[1] = id;
                        }
                        other => panic!("unexpected storage {other}: {q:?}"),
                    }
                }
                5 => {
                    self.commands.push(q);
                    p[..8].fill(0);
                    match q[0] {
                        1 | 5 => {
                            p[1] = self.epoch;
                            p[2] = self.width;
                            p[3] = self.height;
                            if q[0] == 5
                                && let Some(event) = self.events.pop_front()
                            {
                                p[5..8].copy_from_slice(&event[5..8]);
                            }
                        }
                        6 => {
                            p[1] = self.next_surface;
                            self.next_surface += 1;
                        }
                        3 => {}
                        8 => {
                            assert_eq!(self.staged.len() as u64, q[5] * q[6] * 4);
                            if self.race {
                                self.race = false;
                                self.epoch += 1;
                                p[0] = ERROR;
                                self.staged.clear();
                            } else {
                                self.images.push((q, core::mem::take(&mut self.staged)));
                            }
                        }
                        14 => {
                            p[2] = 77;
                        }
                        15 | 16 | 19 | 22 | 24 => {}
                        17 => {
                            p[1] = 1;
                        }
                        18 => {
                            p[1] = self.managed;
                        }
                        21 => {}
                        23 => p[..8].copy_from_slice(&self.tasks[q[1] as usize]),
                        other => panic!("unexpected interaction {other}: {q:?}"),
                    }
                }
                6 => {
                    self.refresh = p[13];
                    p[13] = 0;
                    return;
                }
                7 => self.replies.push(q),
                9 => {
                    assert_eq!(q[1] as usize, self.staged.len());
                    assert!(q[0] <= 65536);
                    self.staged.extend_from_slice(&payload);
                    assert!(self.staged.len() <= q[0] as usize);
                }
                11 => {
                    p[..8].fill(0);
                    self.now += 10;
                    p[0] = self.now;
                }
                other => panic!("unexpected host {other}"),
            }
        }
        panic!("chooser did not yield");
    }
    fn wake(&mut self) {
        self.app.packet[9] = 0;
        self.pump();
        assert!(!self.exited);
    }
    fn click(&mut self, x: u64, y: u64) {
        self.event(input::POINTER | (1 << 32), x, y);
        self.event(input::POINTER | (1 << 32) | (1 << 8), x, y);
    }
    fn event(&mut self, kind: u64, a: u64, b: u64) {
        self.events.push_back([0, 0, 0, 0, 0, kind, a, b]);
        self.wake();
    }
    fn loss(&mut self) -> u64 {
        self.app.packet[..8].fill(0);
        self.app.packet[0] = 121;
        self.app.packet[9] = 1;
        self.pump();
        self.replies.last().unwrap()[1]
    }
    fn saves(&self) -> usize {
        self.storage_commands
            .iter()
            .filter(|q| q[0] == storage::PUBLISH)
            .count()
    }
}
fn payload_reply(p: &mut [u64; 4096], bytes: &[u8]) {
    p[8] = bytes.len() as u64;
    for (i, chunk) in bytes.chunks(8).enumerate() {
        let mut word = [0; 8];
        word[..chunk.len()].copy_from_slice(chunk);
        p[16 + i] = u64::from_le_bytes(word);
    }
}
