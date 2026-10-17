// Two private catalogs delegated through the same matrix executable. Leaves
// relay test requests through their sole parent connection, with no disk/link grant.
use crate::fixture::Fixture;
use crate::*;
use cathedral_platform_contracts::storage as store;
use cathedral_user_runtime::task::{Launch, Port};
fn request(
    f: &Fixture,
    leaf: usize,
    op: u64,
    object: u64,
    generation: u64,
    payload: &[u8],
) -> Result<store::Record, u64> {
    f.query(leaf, [wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0]);
    let bytes = store::request(wire::STORAGE_BASE + op, object, generation, payload).unwrap();
    store::Record::decode(&wire::encode(f.query(leaf, wire::decode(&bytes).unwrap())))
}
fn listing(f: &Fixture, leaf: usize) -> store::Record {
    request(f, leaf, store::LIST, 0, 0, &[]).unwrap()
}
fn grant(f: &Fixture) {
    f.owner(0, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]);
    f.owner(1, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]);
    // The leaf has not refreshed its old local binding yet. Even a valid object
    // number cannot authorize that stale request in the newly granted namespace.
    assert_eq!(
        f.query(2, [wire::STORAGE_BASE + store::READ, 2, 0, 0, 0, 0, 0, 0]),
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
    probes::commit(f);
    recovery::exercise(f, provider);
    cathedral_user_runtime::write(b"Cathedral matrix storage: scoped catalogs, recursive delegation, revocation and replacement passed\n").unwrap();
    loop {
        let _ = event_sleep();
    }
}
fn event_sleep() -> Result<usize, Error> {
    cathedral_user_runtime::event::wait(&[], time::after(100)?)
}
