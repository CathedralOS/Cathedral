//! Storage data client: contracts/storage.rs -> platform/services/storage/main.rs -> service.rs.
//! Reconcile ambiguous completion by reading the private object before retrying.
use super::connection::Connections;
use cathedral_contracts::{storage as wire, user as abi};
use cathedral_user_runtime::{Error, time};

fn exchange(c: &mut Connections, request: &[u8; 64]) -> Result<wire::Record, Error> {
    let (send, receive) = c.pair(cathedral_session_protocol::link::STORAGE)?;
    send.send(request)?;
    let mut bytes = [0; 64];
    let len = receive.receive_until(&mut bytes, time::after(150)?)?;
    wire::Record::decode(&bytes[..len]).map_err(|e| Error(e as i64))
}
pub fn load(c: &mut Connections) -> Result<wire::Record, Error> {
    for _ in 0..3 {
        match exchange(c, &wire::request(wire::READ, wire::OBJECT, 0, &[]).unwrap()) {
            Ok(record) => return Ok(record),
            Err(_) => {
                c.storage = None;
                c.refresh()?;
            }
        }
    }
    Err(Error(abi::IO_ERROR as i64))
}
pub fn save(c: &mut Connections, expected: u64, selected: u8, active: u8) -> Result<u64, Error> {
    let payload = [1, selected, active];
    for _ in 0..3 {
        let current = load(c)?;
        if expected.checked_add(1) == Some(current.generation) && current.payload() == payload {
            return Ok(current.generation); // Durable commit, lost reply.
        }
        if current.generation != expected {
            return Err(Error(abi::WOULD_BLOCK as i64));
        }
        match exchange(
            c,
            &wire::request(wire::REPLACE, wire::OBJECT, expected, &payload).unwrap(),
        ) {
            Ok(record)
                if expected.checked_add(1) == Some(record.generation)
                    && record.payload() == payload =>
            {
                return Ok(record.generation);
            }
            _ => {
                c.storage = None;
                c.refresh()?;
            }
        }
    }
    Err(Error(abi::IO_ERROR as i64))
}
