//! Test-only fault requests and verification of the surviving sibling's identity.
use super::service::Service;
use cathedral_contracts::user as abi;
use cathedral_user_runtime::{Error, write};

#[derive(Default)]
pub struct Probes {
    counts: [u8; 2],
    pending: Option<(usize, u64, u64, u64)>,
}
impl Probes {
    pub fn inject(
        &mut self,
        index: usize,
        target: &Service,
        peer: &Service,
        app: &Service,
    ) -> Result<(), Error> {
        assert!(self.pending.is_none());
        let mode = self.counts[index] % 3 + 1;
        self.counts[index] = (self.counts[index] + 1) % 3;
        target.send.send(&[0xf0, mode])?;
        let (bytes, length) = target.receive()?;
        if &bytes[..length] != b"armed" {
            return Err(Error(abi::IO_ERROR as i64));
        }
        self.pending = Some((index, target.ticket(), peer.ticket(), app.ticket()));
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
        app: &Service,
    ) -> Result<(), Error> {
        if let Some((index, old, sibling, application)) = self.pending {
            assert_eq!(app.ticket(), application);
            let services = [display, input];
            assert_eq!(services[1 - index].ticket(), sibling);
            if services[index].ticket() != old {
                self.pending = None;
                write(b"Cathedral: recovered with sibling and application identities preserved\n")?;
            }
        }
        Ok(())
    }
}
