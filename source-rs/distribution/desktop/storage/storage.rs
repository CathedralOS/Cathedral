// Two private roots delegated through the same matrix executable. Leaves relay
// test requests through their sole parent connection, with no disk/link grant.
use crate::fixture::Fixture;
use crate::*;
use cathedral_platform_contracts::storage as store;
use cathedral_user_runtime::task::{Launch, Port};
/// Relay one object request; the reply keeps the provider's 64-byte layout.
fn request(f: &Fixture, leaf: usize, request: store::Request, payload: &[u8]) -> [u8; 64] {
    f.query(leaf, [wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0]);
    let mut words = request.words();
    words[0] += wire::STORAGE_BASE;
    assert!(payload.len() <= 24);
    let mut bytes = [0; 24];
    bytes[..payload.len()].copy_from_slice(payload);
    for (i, word) in words[5..].iter_mut().enumerate() {
        *word = u64::from_le_bytes(bytes[i * 8..i * 8 + 8].try_into().unwrap());
    }
    wire::encode(f.query(leaf, words))
}
fn packet(op: u64, object: u64, expected: u64, len: u64) -> store::Request {
    store::Request {
        op,
        object,
        expected,
        offset: 0,
        len,
    }
}
fn publish(
    f: &Fixture,
    leaf: usize,
    object: u64,
    expected: u64,
    payload: &[u8],
) -> Result<store::Reply, u64> {
    store::Reply::decode(&request(
        f,
        leaf,
        packet(store::PUBLISH, object, expected, payload.len() as u64),
        payload,
    ))
}
fn read(f: &Fixture, leaf: usize, object: u64) -> Result<([u8; 32], usize), u64> {
    let head = store::Reply::decode(&request(f, leaf, packet(store::HEAD, object, 0, 0), &[]))?;
    let reply = request(
        f,
        leaf,
        packet(store::READ, object, head.revision, head.length),
        &[],
    );
    store::Reply::decode(&reply)?;
    Ok((reply[32..].try_into().unwrap(), head.length as usize))
}
fn text(f: &Fixture, leaf: usize, object: u64) -> Result<[u8; 32], u64> {
    read(f, leaf, object).map(|(bytes, _)| bytes)
}
fn expect(bytes: [u8; 32], payload: &[u8]) {
    assert_eq!(&bytes[..payload.len()], payload);
}
fn listing(f: &Fixture, leaf: usize) -> Result<[u64; 8], u64> {
    let reply = request(f, leaf, packet(store::LIST, 0, 0, 0), &[]);
    let words = store::unpack(&reply).unwrap();
    if words[0] != 0 {
        return Err(words[0]);
    }
    Ok(words)
}
fn grant(f: &Fixture) {
    f.owner(0, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]);
    f.owner(1, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]);
    // The leaf has not refreshed its old local binding yet. Even a valid object
    // number cannot authorize that stale request in the newly granted root.
    assert_eq!(
        f.query(2, [wire::STORAGE_BASE + store::HEAD, 1, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
}
pub fn run() -> u64 {
    let provider = Launch::at(4).unwrap().spawn(0).unwrap();
    let control = Port::at(4).unwrap().connect().unwrap();
    control.0.send(store::HEALTH).unwrap();
    assert_eq!(control.1.receive(&mut [0; 64]).unwrap(), 5);
    let fixture = Fixture::open();
    let f = &fixture;
    probes::delegation(f);
    probes::revocation(f);
    probes::replacement(f);
    probes::sharing(f);
    recovery::exercise(f, provider);
    cathedral_user_runtime::write(b"Cathedral matrix storage: scoped roots, recursive delegation, revocation, cheap copies and replacement passed\n").unwrap();
    loop {
        let _ = event_sleep();
    }
}
fn event_sleep() -> Result<usize, Error> {
    cathedral_user_runtime::event::wait(&[], time::after(100)?)
}
