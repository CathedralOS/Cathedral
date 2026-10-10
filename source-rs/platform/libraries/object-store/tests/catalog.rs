use super::*;
#[test]
fn transactions_are_root_local_atomic_and_do_not_resurrect_deleted_generations() {
    let mut store = Store::open(Disk::blank()).unwrap();
    let create = |id, value: &[u8]| Change::value(wire::CREATE, id, value).unwrap();
    store
        .transact(1, 0, &[create(1, b"private"), create(2, b"other")], |_| {})
        .unwrap();
    assert_eq!(store.read(0, 1).unwrap().payload(), b"");
    assert_eq!(store.read(0, 2), Err(wire::NOT_FOUND));
    assert_eq!(store.read(2, 1), Err(abi::DENIED));
    let before = store.catalog;
    let operations = store.disk.operations;
    assert_eq!(
        store.transact(1, 1, &[Change::Delete(1), create(2, b"collision")], |_| {}),
        Err(wire::EXISTS)
    );
    assert_eq!(store.catalog, before);
    assert_eq!(store.disk.operations, operations);
    store
        .transact(1, 1, &[Change::Delete(1), Change::Delete(2)], |_| {})
        .unwrap();
    assert_eq!(store.root(1).unwrap().list().length, 0);
    store.transact(1, 2, &[create(1, b"new")], |_| {}).unwrap();
    assert_eq!(
        store.transact(1, 1, &[Change::Delete(1)], |_| {}),
        Err(abi::WOULD_BLOCK)
    );
    assert_eq!(store.read(1, 1).unwrap().payload(), b"new");
}
