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
/// The object a leaf's binding is rooted at, once it has a binding.
fn realm(f: &Fixture, leaf: usize) -> Result<u64, u64> {
    let info = f.query(leaf, [wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0]);
    if info[0] != 0 {
        return Err(info[0]);
    }
    Ok(info[2])
}
fn publish(
    f: &Fixture,
    leaf: usize,
    object: u64,
    expected: u64,
    payload: &[u8],
) -> Result<store::Reply, u64> {
    let parent = if object == 0 { realm(f, leaf)? } else { 0 };
    store::Reply::decode(&request(
        f,
        leaf,
        store::Request::new(
            store::PUBLISH,
            object,
            expected,
            parent,
            payload.len() as u64,
        ),
        payload,
    ))
}
fn copy(
    f: &Fixture,
    leaf: usize,
    object: u64,
    expected: u64,
    parent: u64,
) -> Result<store::Reply, u64> {
    store::Reply::decode(&request(
        f,
        leaf,
        store::Request::new(store::COPY, object, expected, parent, 0),
        &[],
    ))
}
fn copy_tree(
    f: &Fixture,
    leaf: usize,
    object: u64,
    expected: u64,
    parent: u64,
) -> Result<store::Reply, u64> {
    store::Reply::decode(&request(
        f,
        leaf,
        store::Request::new(store::COPY_TREE, object, expected, parent, 0),
        &[],
    ))
}
fn read(f: &Fixture, leaf: usize, object: u64) -> Result<([u8; 32], usize), u64> {
    let head = store::Reply::decode(&request(
        f,
        leaf,
        store::Request::new(store::HEAD, object, 0, 0, 0),
        &[],
    ))?;
    let reply = request(
        f,
        leaf,
        store::Request::new(store::READ, object, head.revision, 0, head.length),
        &[],
    );
    store::Reply::decode(&reply)?;
    Ok((reply[32..].try_into().unwrap(), head.length as usize))
}
fn text(f: &Fixture, leaf: usize, object: u64) -> Result<[u8; 32], u64> {
    read(f, leaf, object).map(|(bytes, _)| bytes)
}
/// The first bytes of an object too large to read whole through the inline path.
fn prefix(f: &Fixture, leaf: usize, object: u64, len: u64) -> Result<[u8; 32], u64> {
    let reply = request(
        f,
        leaf,
        store::Request::new(store::READ, object, 0, 0, len),
        &[],
    );
    store::Reply::decode(&reply)?;
    Ok(reply[32..].try_into().unwrap())
}
fn expect(bytes: [u8; 32], payload: &[u8]) {
    assert_eq!(&bytes[..payload.len()], payload);
}
/// The children of the leaf's realm: count, then the numbers.
fn listing(f: &Fixture, leaf: usize) -> Result<[u64; 8], u64> {
    let parent = realm(f, leaf)?;
    let reply = request(
        f,
        leaf,
        store::Request::new(store::LIST, parent, 0, 0, 0),
        &[],
    );
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
/// Give the nested matrix one subtree of root 0 and its leaf the whole of that.
fn regrant_scoped(f: &Fixture, scope: u64) {
    f.owner(0, [wire::STORAGE_REVOKE, 0, 0, 0, 0, 0, 0, 0]);
    f.owner(0, [wire::STORAGE_GRANT, 0, 0, scope, 0, 0, 0, 0]);
    f.owner(1, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(realm(f, 2), Ok(scope));
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
    probes::attenuation(f);
    probes::launching(f);
    probes::sibling(f);
    recovery::exercise(f, provider);
    cathedral_user_runtime::write(b"Cathedral matrix storage: scoped roots, recursive delegation, revocation, cheap copies, attenuated realms, programs from the realm, launched siblings and replacement passed\n").unwrap();
    loop {
        let _ = event_sleep();
    }
}
fn event_sleep() -> Result<usize, Error> {
    cathedral_user_runtime::event::wait(&[], time::after(100)?)
}
