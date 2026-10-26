//! Opt-in owner-controlled fault fixture. Ordinary builds omit it entirely.
//! Arm: `[0xf2, phase 1..=2, operation, root]` stalls that root's next
//! matching request before execution (1) or after commit before reply (2);
//! `[0xf6]` queries armed/blocked/idle; `[0xf3]` releases with an acknowledgement.
//! `[0xf4]` reports each root's
//! lowest object number so a probe can tell a fresh database from a used one.
use cathedral_database_objects::Store;
use cathedral_platform_contracts::storage as wire;
use cathedral_user_runtime::server::{Event, Request, Server};
#[derive(Default)]
pub struct Gate {
    phase: u8,
    opcode: u8,
    root: u8,
}
impl Gate {
    pub fn control(&mut self, store: &Store<'_>, server: &mut Server, request: &Request) -> bool {
        match request.bytes[..request.len] {
            [0xf2, phase @ 1..=2, opcode, root @ 0..=1] if u64::from(opcode) <= wire::LAST => {
                *self = Self {
                    phase,
                    opcode,
                    root,
                };
                #[cfg(feature = "trace-storage")]
                cathedral_user_runtime::time::storage_trace(
                    "gate-armed",
                    phase as u64,
                    opcode as u64,
                );
                server.reply(request, b"armed").unwrap();
                true
            }
            [0xf6] => {
                server
                    .reply(request, if self.phase == 0 { b"idle" } else { b"armed" })
                    .unwrap();
                true
            }
            [0xf4] => {
                let mut reply = [0; 16];
                for root in 0..2u8 {
                    let mut first = [0];
                    if store.list(root, 0, 0, &mut first).unwrap() == 1 {
                        reply[root as usize * 8..root as usize * 8 + 8]
                            .copy_from_slice(&first[0].to_le_bytes());
                    }
                }
                server.reply(request, &reply).unwrap();
                true
            }
            _ => false,
        }
    }
    pub fn wait(&mut self, server: &mut Server, phase: u8, opcode: u64, root: u8) {
        if self.phase != phase || self.opcode as u64 != opcode || self.root != root {
            return;
        }
        self.phase = 0;
        #[cfg(feature = "trace-storage")]
        cathedral_user_runtime::time::storage_trace("gate-stall", phase as u64, opcode);
        // Retain the in-flight store request. Other data requests remain queued;
        // only authenticated owner control can observe or release this fixture.
        server.pause(((1 << wire::ROOTS) - 1) << 1);
        loop {
            let request = match server.next_event().unwrap() {
                Event::Request(request) => request,
                Event::Memory(request, object) => {
                    drop(object);
                    server
                        .reply(
                            &request,
                            &wire::error(cathedral_kernel_contracts::user::INVALID_ARGUMENT),
                        )
                        .unwrap();
                    continue;
                }
                Event::Disconnected { .. } => continue,
            };
            assert!(request.control);
            match &request.bytes[..request.len] {
                [0xf6] => server.reply(&request, b"blocked").unwrap(),
                [0xf3] => {
                    server.reply(&request, b"released").unwrap();
                    server.pause(0);
                    break;
                }
                bytes if bytes == wire::HEALTH => {
                    // Deliberately unresponsive: the watchdog must retire us.
                }
                _ => server
                    .reply(
                        &request,
                        &wire::error(cathedral_kernel_contracts::user::INVALID_ARGUMENT),
                    )
                    .unwrap(),
            }
        }
    }
}
