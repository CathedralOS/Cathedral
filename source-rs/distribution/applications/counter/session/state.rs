//! App-owned schema: COUNTER and MIRROR must contain the same persisted count.
use cathedral_contracts::{storage as wire, user as abi};
use cathedral_user_runtime::{Error, storage::Client};
pub fn read(store: &mut Client) -> Result<(u64, u64), Error> {
    let first = store.call(wire::READ, 1, 0, &[])?;
    let second = store.call(wire::READ, 2, 0, &[])?;
    if first.length != 16
        || second.length != 16
        || &first.bytes[..8] != b"COUNTER!"
        || &second.bytes[..8] != b"MIRROR!!"
        || first.bytes[8..16] != second.bytes[8..16]
        || first.generation != second.generation
    {
        return Err(Error(abi::IO_ERROR as i64));
    }
    Ok((
        first.generation,
        u64::from_le_bytes(first.bytes[8..16].try_into().unwrap()),
    ))
}
pub fn advance(store: &mut Client) -> Result<(), Error> {
    let listing = store.call(wire::LIST, 0, 0, &[])?;
    let count = if listing.length == 0 {
        0
    } else {
        read(store)?.1
    };
    let next = count.checked_add(1).ok_or(Error(abi::IO_ERROR as i64))?;
    store.call(wire::BEGIN, 0, listing.generation, &[])?;
    for (object, label) in [(1, b"COUNTER!"), (2, b"MIRROR!!")] {
        let mut bytes = [0; 16];
        bytes[..8].copy_from_slice(label);
        bytes[8..].copy_from_slice(&next.to_le_bytes());
        store.call(
            if listing.length == 0 {
                wire::STAGE_CREATE
            } else {
                wire::STAGE_REPLACE
            },
            object,
            0,
            &bytes,
        )?;
    }
    store.call(wire::COMMIT, 0, 0, &[])?;
    assert_eq!(read(store)?.1, next);
    Ok(())
}
