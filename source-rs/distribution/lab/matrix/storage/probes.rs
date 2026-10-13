//! Scope isolation, stale transactions and replacement over real IPC.
use super::*;
pub(super) fn delegation(f: &Fixture) {
    assert_eq!(request(f, 2, store::LIST, 0, 0, &[]), Err(abi::DENIED));
    grant(f);
    assert_eq!(
        f.query(0, [wire::STORAGE_GRANT, 1, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::BUSY)
    );
    f.owner(0, [wire::STORAGE_GRANT, 1, 1, 0, 0, 0, 0, 0]);
    assert_eq!(
        f.query(1, [wire::STORAGE_GRANT, 1, 1, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    let a = listing(f, 2);
    let b = listing(f, 3);
    request(f, 2, store::CREATE, 2, a.generation, b"nested catalog").unwrap();
    assert_eq!(listing(f, 3), b); // No sibling generation or membership leaks.
    assert_eq!(request(f, 3, store::READ, 2, 0, &[]), Err(store::NOT_FOUND));
    request(f, 3, store::CREATE, 2, b.generation, b"outside catalog").unwrap();
    assert_eq!(
        request(f, 2, store::READ, 2, 0, &[]).unwrap().payload(),
        b"nested catalog"
    );
    assert_eq!(
        f.query(2, [wire::STORAGE_GRANT, 0, 1, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
}
pub(super) fn revocation(f: &Fixture) {
    // Revoke an ancestor mid-transaction. Regrant must not revive descendants or
    // permit their old commit to publish; an explicit child grant starts fresh.
    request(f, 2, store::BEGIN, 0, listing(f, 2).generation, &[]).unwrap();
    request(f, 2, store::STAGE_REPLACE, 2, 0, b"must disappear").unwrap();
    // Revoke and regrant the direct leaf without allowing it an INFO refresh.
    f.owner(1, [wire::STORAGE_REVOKE, 0, 0, 0, 0, 0, 0, 0]);
    f.owner(1, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        f.query(2, [wire::STORAGE_BASE + store::READ, 2, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    request(f, 2, store::BEGIN, 0, listing(f, 2).generation, &[]).unwrap();
    request(f, 2, store::STAGE_REPLACE, 2, 0, b"must disappear").unwrap();
    f.owner(0, [wire::STORAGE_REVOKE, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(request(f, 2, store::COMMIT, 0, 0, &[]), Err(abi::DENIED));
    f.owner(0, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(request(f, 2, store::READ, 2, 0, &[]), Err(abi::DENIED));
    f.owner(1, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        request(f, 2, store::COMMIT, 0, 0, &[]),
        Err(abi::BAD_HANDLE)
    );
    assert_eq!(
        request(f, 2, store::READ, 2, 0, &[]).unwrap().payload(),
        b"nested catalog"
    );
}
pub(super) fn replacement(f: &Fixture) {
    // Child death aborts staged work; replacement gets no implicit storage grant.
    request(f, 2, store::BEGIN, 0, listing(f, 2).generation, &[]).unwrap();
    request(f, 2, store::STAGE_DELETE, 2, 0, &[]).unwrap();
    f.owner(1, [wire::STOP, 0, 0, 0, 0, 0, 0, 0]);
    f.owner(1, [wire::SPAWN, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(request(f, 2, store::READ, 2, 0, &[]), Err(abi::DENIED));
    f.owner(1, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        request(f, 2, store::COMMIT, 0, 0, &[]),
        Err(abi::BAD_HANDLE)
    );
    assert_eq!(
        request(f, 2, store::READ, 2, 0, &[]).unwrap().payload(),
        b"nested catalog"
    );
    assert_eq!(
        request(f, 3, store::READ, 2, 0, &[]).unwrap().payload(),
        b"outside catalog"
    );
}
pub(super) fn commit(f: &Fixture) {
    // Normal transactions still reach the durable provider through both hops.
    request(f, 2, store::BEGIN, 0, listing(f, 2).generation, &[]).unwrap();
    request(f, 2, store::STAGE_REPLACE, 2, 0, b"committed nested").unwrap();
    request(f, 2, store::COMMIT, 0, 0, &[]).unwrap();
    assert_eq!(
        request(f, 2, store::READ, 2, 0, &[]).unwrap().payload(),
        b"committed nested"
    );
}
