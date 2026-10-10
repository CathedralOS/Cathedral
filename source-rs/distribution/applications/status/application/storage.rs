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
fn read_primary(c: &mut Connections) -> Result<wire::Record, Error> {
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
/// Old single-object data is read without rewriting it. The next user edit creates
/// the second record and converts both records in the same durable transaction.
pub fn load(c: &mut Connections) -> Result<wire::Record, Error> {
    let primary = read_primary(c)?;
    if primary.payload().first() != Some(&2) {
        return Ok(primary);
    }
    let secondary = exchange(c, &wire::request(wire::READ, 2, 0, &[]).unwrap())?;
    if primary.length != 10
        || secondary.length != 10
        || secondary.bytes[0] != 2
        || primary.generation != secondary.generation
        || primary.bytes[2..10] != primary.generation.to_le_bytes()
        || secondary.bytes[2..10] != primary.generation.to_le_bytes()
    {
        return Err(Error(abi::IO_ERROR as i64));
    }
    let mut restored = wire::Record {
        generation: primary.generation,
        length: 3,
        ..wire::Record::default()
    };
    restored.bytes[..3].copy_from_slice(&[1, primary.bytes[1], secondary.bytes[1]]);
    Ok(restored)
}
pub fn save(c: &mut Connections, expected: u64, selected: u8, active: u8) -> Result<u64, Error> {
    let payload = [1, selected, active];
    for _ in 0..3 {
        let current = load(c)?;
        if expected.checked_add(1) == Some(current.generation) && current.payload() == payload {
            return Ok(current.generation); // Both durable records verified after a lost reply.
        }
        if current.generation != expected {
            return Err(Error(abi::WOULD_BLOCK as i64));
        }
        match transaction(c, expected, selected, active) {
            Ok(generation) => return Ok(generation),
            Err(_) => {
                c.storage = None;
                c.refresh()?;
            }
        }
    }
    Err(Error(abi::IO_ERROR as i64))
}
fn transaction(c: &mut Connections, expected: u64, selected: u8, active: u8) -> Result<u64, Error> {
    let listing = exchange(c, &wire::request(wire::LIST, 0, 0, &[]).unwrap())?;
    let has_secondary = listing
        .payload()
        .chunks_exact(8)
        .any(|id| id == 2u64.to_le_bytes());
    exchange(c, &wire::request(wire::BEGIN, 0, expected, &[]).unwrap())?;
    let generation = expected.checked_add(1).ok_or(Error(abi::IO_ERROR as i64))?;
    for (object, value) in [(1, selected), (2, active)] {
        let mut record = [0; 10];
        record[0] = 2;
        record[1] = value;
        record[2..].copy_from_slice(&generation.to_le_bytes());
        let op = if object == 2 && !has_secondary {
            wire::STAGE_CREATE
        } else {
            wire::STAGE_REPLACE
        };
        exchange(c, &wire::request(op, object, 0, &record).unwrap())?;
    }
    let committed = exchange(c, &wire::request(wire::COMMIT, 0, 0, &[]).unwrap())?;
    if committed.generation != generation {
        return Err(Error(abi::IO_ERROR as i64));
    }
    Ok(generation)
}

pub fn catalog(c: &mut Connections) -> Result<u8, Error> {
    let list = exchange(c, &wire::request(wire::LIST, 0, 0, &[]).unwrap())?;
    let mut mask = 0;
    for bytes in list.payload().chunks_exact(8) {
        let object = u64::from_le_bytes(bytes.try_into().unwrap());
        if !(1..=wire::OBJECTS as u64).contains(&object) {
            return Err(Error(abi::IO_ERROR as i64));
        }
        mask |= 1 << (object - 1);
    }
    Ok(mask)
}
