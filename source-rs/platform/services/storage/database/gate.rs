//! Opt-in owner-controlled fault fixture. Production catalog builds omit it.
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
    pub fn arm(&mut self, server: &mut Server, request: &Request) -> bool {
        if let [
            command @ (0xf2 | 0xf5),
            phase @ 1..=2,
            opcode @ (2 | 4 | 10),
            root @ 0..=1,
        ] = request.bytes[..request.len]
        {
            *self = Self {
                phase,
                opcode,
                root,
                quiet: command == 0xf5,
            };
            server.reply(request, b"armed").unwrap();
            true
        } else {
            false
        }
    }
    pub fn wait(&mut self, phase: u8, opcode: u64, root: usize) {
        if self.phase != phase || self.opcode as u64 != opcode || self.root as usize != root {
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
            assert_eq!(&bytes[..n], cathedral_platform_contracts::storage::HEALTH);
        }
    }
}
