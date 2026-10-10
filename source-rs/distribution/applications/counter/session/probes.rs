//! Execute hostile requests from the second real principal, never with storage authority.
use cathedral_contracts::{storage as wire, user as abi};
use cathedral_user_runtime::{Error, storage::Client, time, write, yield_now};
pub fn catalog(store: &mut Client, generation: u64) -> Result<(), Error> {
    let original = super::state::read(store)?;
    assert_eq!(
        store.call(wire::READ, u64::MAX, 0, &[]),
        Err(Error(abi::DENIED as i64))
    );
    assert_eq!(
        store.call(wire::REPLACE, 1, original.0.wrapping_add(1), b"stale"),
        Err(Error(abi::WOULD_BLOCK as i64))
    );
    store.call(wire::BEGIN, 0, original.0, &[])?;
    store.call(wire::STAGE_CREATE, 3, 0, b"third")?;
    store.call(wire::STAGE_CREATE, 4, 0, b"fourth")?;
    assert_eq!(
        store.call(wire::STAGE_DELETE, 1, 0, &[]),
        Err(Error(abi::NO_MEMORY as i64))
    );
    let created = store.call(wire::COMMIT, 0, 0, &[])?;
    assert_eq!(created.length, 32);
    assert_eq!(store.call(wire::READ, 3, 0, &[])?.payload(), b"third");
    store.call(wire::BEGIN, 0, created.generation, &[])?;
    store.call(wire::STAGE_DELETE, 3, 0, &[])?;
    store.call(wire::STAGE_DELETE, 4, 0, &[])?;
    let deleted = store.call(wire::COMMIT, 0, 0, &[])?;
    assert_eq!(deleted.length, 16);
    assert_eq!(
        store.call(wire::READ, 3, 0, &[]),
        Err(Error(wire::NOT_FOUND as i64))
    );
    assert_eq!(super::state::read(store)?.1, original.1);
    // Fill the response queue repeatedly, then drain it. Other channels must remain live.
    let (send, receive) = store.pair()?;
    for _ in 0..8 {
        let deadline = time::after(100)?;
        loop {
            match send.send(&wire::request(wire::LIST, 0, 0, &[]).unwrap()) {
                Ok(()) => break,
                Err(Error(code))
                    if code == abi::WOULD_BLOCK as i64
                        && !time::reached(time::now()?, deadline) =>
                {
                    yield_now()
                }
                Err(error) => return Err(error),
            }
        }
    }
    while receive.receive_until(&mut [0; 64], time::after(3)?).is_ok() {}
    assert_eq!(super::state::read(store)?.1, original.1);
    write(b"Cathedral: two-root catalog bounds isolation and backpressure passed\n")?;
    if generation == 0 {
        store.call(wire::BEGIN, 0, deleted.generation, &[])?;
        store.call(wire::STAGE_CREATE, 3, 0, b"must not survive client exit")?;
        write(b"Cathedral: counter exits with uncommitted changes\n")?;
        return Err(Error(abi::IO_ERROR as i64));
    }
    assert_eq!(
        store.call(wire::READ, 3, 0, &[]),
        Err(Error(wire::NOT_FOUND as i64))
    );
    write(b"Cathedral: counter replacement discarded staged changes\n")
}
