//! Invoke the installed Omega export; the peer supplies only completed IPC work.
use super::code::Code;
use cathedral_omega_app::{Instance, Step};
use cathedral_platform_contracts::{matrix as wire, services};
use std::{vec, vec::Vec};
include!(env!("OMEGA_APP_CONSTANTS"));

pub(super) const VIEW: [u64; 8] = [0, 7, 320, 200, 0, 0, 0, 0];
pub(super) const SURFACE: u64 = 33;
pub(super) const BOOT: [u64; 7] = [1, 5, 2, 8, 5, 5, 6];
pub(super) const SCOPE_MESSAGE: &[u8] =
    b"Cathedral background: shared interaction ready; storage denied\n";

pub(super) fn with_app(run: impl FnOnce(&mut Peer<'_>)) {
    let code = Code::new(BYTES);
    // SAFETY: Packaging checked the export, which stays RX-mapped until the
    // instance and its disjoint, exclusively borrowed buffers have dropped.
    let step: Step = unsafe { core::mem::transmute(code.address.add(ENTRY)) };
    let mut state = [0; 512];
    let mut packet = vec![0; 4096];
    // SAFETY: Same live export and buffer lifetime contract as above.
    let app = unsafe { Instance::new(step, &mut state, packet.as_mut_slice().try_into().unwrap()) }
        .unwrap();
    run(&mut Peer { app });
}

pub(super) struct Peer<'a> {
    pub app: Instance<'a>,
}

impl Peer<'_> {
    pub fn next(&mut self, operation: u64) -> [u64; 8] {
        assert_eq!(self.app.advance(), operation);
        if operation == 6 {
            assert_eq!(self.app.packet[13], 100);
            // The real host consumes the timeout before delivering completion.
            self.app.packet[13] = 0;
        }
        self.app.packet[..8].try_into().unwrap()
    }

    pub fn reply(&mut self, words: [u64; 8], transport: u64) {
        self.app.packet[..8].copy_from_slice(&words);
        self.app.packet[8] = 0;
        self.app.packet[9] = transport;
    }

    pub fn payload(&self) -> Vec<u8> {
        self.app.packet[16..]
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .take(self.app.packet[8] as usize)
            .collect()
    }

    /// Stop at a host operation, before supplying its result. These expectations
    /// describe the old installed program's externally visible startup sequence.
    pub fn through(&mut self, last: usize) -> [u64; 8] {
        for (index, operation) in BOOT.into_iter().enumerate().take(last + 1) {
            let request = self.next(operation);
            match index {
                0 | 2 => assert_eq!(request, [1, 1, 0, 0, 0, 0, 0, 0]),
                1 => assert_eq!(request, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]),
                3 => assert_eq!(self.payload(), SCOPE_MESSAGE),
                4 => assert_eq!(request, [wire::SURFACE_CREATE, 7, 0, 0, 0, 0, 0, 0]),
                5 => assert_eq!(request, [wire::PAINT, 7, SURFACE, 0, 0, 320, 200, 0x101827]),
                _ => {}
            }
            if index == last {
                return request;
            }
            match index {
                1 => self.reply(VIEW, 0),
                2 => self.reply([0; 8], services::UNAVAILABLE),
                4 => self.reply([0, SURFACE, 0, 0, 0, 0, 0, 0], 0),
                _ => self.reply([0; 8], 0),
            }
        }
        panic!("invalid startup checkpoint");
    }

    pub fn failed(&mut self, code: u64) {
        assert_eq!(self.next(12), [code, 0, 0, 0, 0, 0, 0, 0]);
        // A failing host stops the task. Even an extra call cannot retry work.
        assert_eq!(self.next(0), [0; 8]);
    }
}
