//! Display control client. Protocol: contracts/display.rs; peer:
//! platform/services/display/main.rs -> surface.rs. The app uses a separate data link.
use super::super::child::Service;
use cathedral_contracts::{display, user as abi};
use cathedral_user_runtime::Error;
pub fn ensure_ready(service: &mut Service) -> Result<bool, Error> {
    for attempt in 0..3 {
        let valid = service
            .send
            .send(&display::encode([display::INFO, 0, 0, 0, 0, 0]))
            .and_then(|()| service.receive())
            .is_ok_and(|(bytes, length)| {
                display::decode(&bytes[..length]).is_some_and(|reply| reply[0] == 0)
            });
        if valid {
            if attempt != 0 {
                service.recovered()?;
            }
            return Ok(attempt != 0);
        }
        if attempt < 2 {
            service.restart()?;
        }
    }
    Err(Error(abi::IO_ERROR as i64))
}
