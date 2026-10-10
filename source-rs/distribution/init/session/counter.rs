//! Supervise the independent private-counter app without replacing its storage provider.
use super::child::Service;
use cathedral_contracts::user as abi;
use cathedral_user_runtime::{Error, write};
pub fn ensure_ready(counter: &mut Service) -> Result<(), Error> {
    for attempt in 0..3 {
        if counter
            .send
            .send(b"health")
            .and_then(|()| counter.receive())
            .is_ok_and(|(bytes, len)| &bytes[..len] == b"ready")
        {
            if attempt != 0 {
                write(b"Cathedral: counter recovered; status app and providers preserved\n")?;
            }
            return Ok(());
        }
        if attempt < 2 {
            counter.restart()?;
        }
    }
    Err(Error(abi::IO_ERROR as i64))
}
