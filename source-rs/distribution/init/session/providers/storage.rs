//! Storage control client. Protocol: contracts/storage.rs; peer:
//! platform/services/storage/main.rs -> service.rs. Health does not grant object access.
use super::super::child::Service;
use cathedral_contracts::user as abi;
use cathedral_user_runtime::{Error, write};
pub fn ensure_ready(service: &mut Service) -> Result<bool, Error> {
    for attempt in 0..3 {
        if service
            .send
            .send(cathedral_contracts::storage::HEALTH)
            .and_then(|()| service.receive())
            .is_ok_and(|(bytes, len)| &bytes[..len] == b"ready")
        {
            if attempt != 0 {
                write(b"Cathedral: storage recovered\n")?;
            }
            return Ok(attempt != 0);
        }
        if attempt < 2 {
            service.restart()?;
        }
    }
    Err(Error(abi::IO_ERROR as i64))
}
