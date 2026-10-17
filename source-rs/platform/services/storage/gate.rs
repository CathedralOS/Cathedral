//! Opt-in owner-controlled fault fixture. Ordinary builds omit it entirely.
//! Arm: `[0xf2 | 0xf5, phase 1..=2, operation, root]` stalls that root's next
//! matching request before execution (1) or after commit before reply (2);
//! 0xf5 stalls silently. `[0xf3]` releases. `[0xf4]` reports each root's
//! lowest object number so a probe can tell a fresh database from a used one.
use cathedral_database_objects::Store;
use cathedral_platform_contracts::storage as wire;
use cathedral_user_runtime::{
    ipc::Handle,
    server::{Request, Server},
};
#[derive(Default)]
pub struct Gate {
    phase: u8,
    opcode: u8,
    root: u8,
    quiet: bool,
}
impl Gate {
    pub fn control(&mut self, store: &Store<'_>, server: &mut Server, request: &Request) -> bool {
        match request.bytes[..request.len] {
            [command @ (0xf2 | 0xf5), phase @ 1..=2, opcode, root @ 0..=1]
                if u64::from(opcode) <= wire::LAST =>
            {
                *self = Self {
                    phase,
                    opcode,
                    root,
                    quiet: command == 0xf5,
                };
                server.reply(request, b"armed").unwrap();
                true
            }
            [0xf4] => {
                let mut reply = [0; 16];
                for root in 0..2u8 {
                    let mut first = [0];
                    if store.list(root, 0, &mut first).unwrap() == 1 {
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
    pub fn wait(&mut self, phase: u8, opcode: u64, root: u8) {
        if self.phase != phase || self.opcode as u64 != opcode || self.root != root {
            return;
        }
        self.phase = 0;
        if !self.quiet {
            Handle::bootstrap(1).unwrap().send(b"blocked").unwrap();
        }
        let mut bytes = [0; 64];
        loop {
            let n = Handle::bootstrap(0).unwrap().receive(&mut bytes).unwrap();
            if bytes[..n] == [0xf3] {
                break;
            }
            // Simulate a live nonresponsive provider, including owner health probes.
            assert_eq!(&bytes[..n], wire::HEALTH);
        }
    }
}
