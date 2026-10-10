//! Input control client. Protocol: contracts/input.rs; peer:
//! platform/services/input/main.rs. RESET is consumed only during readiness.
use super::super::child::Service;
use cathedral_contracts::{input, user as abi};
use cathedral_user_runtime::Error;
pub fn await_ready(input: &mut Service) -> Result<(), Error> {
    for attempt in 0..3 {
        if let Ok((bytes, length)) = input.receive()
            && input::Event::decode(&bytes[..length]) == Some(input::Event::reset())
        {
            if attempt != 0 {
                input.recovered()?;
            }
            return Ok(());
        }
        if attempt < 2 {
            input.restart()?;
        }
    }
    Err(Error(abi::IO_ERROR as i64))
}
pub fn ensure_ready(service: &mut Service) -> Result<bool, Error> {
    for attempt in 0..3 {
        let valid = service
            .send
            .send(&input::HEALTH)
            .and_then(|()| service.receive())
            .is_ok_and(|(bytes, length)| {
                input::Event::decode(&bytes[..length]) == Some(input::Event::idle())
            });
        if valid {
            if attempt != 0 {
                service.recovered()?;
            }
            return Ok(attempt != 0);
        }
        if attempt < 2 {
            service.restart()?;
            await_ready(service)?;
        }
    }
    Err(Error(abi::IO_ERROR as i64))
}
