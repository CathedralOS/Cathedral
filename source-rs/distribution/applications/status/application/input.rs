//! Input data client: contracts/input.rs -> platform/services/input/main.rs.
//! Reconnect after provider loss; init owns the replacement decision.
use super::connection::Connections;
use cathedral_contracts::{input, user as abi};
use cathedral_user_runtime::{Error, time};
pub(super) fn next(connections: &mut Connections) -> Result<input::Event, Error> {
    for _ in 0..3 {
        let (send, receive) = connections.pair(cathedral_session_protocol::link::INPUT)?;
        let mut bytes = [0; 64];
        let result = send
            .send(&input::NEXT)
            .and_then(|()| receive.receive_until(&mut bytes, time::after(150)?));
        if let Ok(length) = result
            && let Some(event) = input::Event::decode(&bytes[..length])
        {
            return Ok(event);
        }
        connections.input = None;
        connections.refresh()?;
    }
    Err(Error(abi::IO_ERROR as i64))
}
