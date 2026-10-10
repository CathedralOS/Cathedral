//! Test-only fault requests and verification of the surviving sibling's identity.
use super::child::Service;
use cathedral_contracts::user as abi;
use cathedral_user_runtime::{Error, write};

#[derive(Default)]
pub struct Probes {
    counts: [u8; 3],
    pending: Option<(usize, [u64; 3], u64)>,
}
impl Probes {
    pub fn inject(
        &mut self,
        index: usize,
        services: [&Service; 3],
        app: &Service,
    ) -> Result<(), Error> {
        let target = services[index];
        assert!(self.pending.is_none());
        let mode = self.counts[index] % 3 + 1;
        self.counts[index] = (self.counts[index] + 1) % 3;
        target.send.send(&[0xf0, mode])?;
        let (bytes, length) = target.receive()?;
        if &bytes[..length] != b"armed" {
            return Err(Error(abi::IO_ERROR as i64));
        }
        self.pending = Some((index, services.map(Service::ticket), app.ticket()));
        write(match mode {
            1 => b"Cathedral: probe crash armed\n",
            2 => b"Cathedral: probe spin armed\n",
            _ => b"Cathedral: probe block armed\n",
        })
    }
    pub fn verify(
        &mut self,
        display: &Service,
        input: &Service,
        storage: &Service,
        app: &Service,
    ) -> Result<(), Error> {
        if let Some((index, old, application)) = self.pending {
            assert_eq!(app.ticket(), application);
            let services = [display, input, storage];
            for other in 0..3 {
                if other != index {
                    assert_eq!(services[other].ticket(), old[other]);
                }
            }
            if services[index].ticket() != old[index] {
                self.pending = None;
                write(b"Cathedral: recovered with sibling and application identities preserved\n")?;
            }
        }
        Ok(())
    }
}
