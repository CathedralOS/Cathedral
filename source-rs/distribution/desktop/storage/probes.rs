//! Scope isolation, stale bindings, replacement and content sharing over real IPC.
use super::*;
pub(super) fn delegation(f: &Fixture) {
    assert_eq!(listing(f, 2), Err(abi::DENIED));
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
    assert_eq!(listing(f, 2).unwrap()[1], 0);
    assert_eq!(listing(f, 3).unwrap()[1], 0);
    let nested = publish(f, 2, 0, 0, b"nested root").unwrap();
    assert_eq!((nested.object, nested.revision), (1, 1));
    // No sibling membership leaks: the outside root is still empty and its
    // own first object gets the same local number.
    assert_eq!(listing(f, 3).unwrap()[1], 0);
    assert_eq!(text(f, 3, 1), Err(store::MISSING));
    let outside = publish(f, 3, 0, 0, b"outside root").unwrap();
    assert_eq!(outside.object, 1);
    expect(text(f, 2, 1).unwrap(), b"nested root");
    expect(text(f, 3, 1).unwrap(), b"outside root");
    assert_eq!(
        f.query(2, [wire::STORAGE_GRANT, 0, 1, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
}
pub(super) fn revocation(f: &Fixture) {
    // Revoke and regrant the direct leaf without allowing it an INFO refresh:
    // its old binding number must not authorize anything in the new grant.
    f.owner(1, [wire::STORAGE_REVOKE, 0, 0, 0, 0, 0, 0, 0]);
    f.owner(1, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        f.query(2, [wire::STORAGE_BASE + store::HEAD, 1, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    expect(text(f, 2, 1).unwrap(), b"nested root");
    // Revoking the ancestor's grant cuts the descendant; regranting the
    // ancestor never implicitly revives it.
    f.owner(0, [wire::STORAGE_REVOKE, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(publish(f, 2, 1, 1, b"must fail"), Err(abi::DENIED));
    f.owner(0, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(text(f, 2, 1), Err(abi::DENIED));
    f.owner(1, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]);
    expect(text(f, 2, 1).unwrap(), b"nested root");
    // A stale revision never overwrites; the refused write leaves no trace.
    assert_eq!(publish(f, 2, 1, 7, b"stale"), Err(abi::WOULD_BLOCK));
    expect(text(f, 2, 1).unwrap(), b"nested root");
}
pub(super) fn replacement(f: &Fixture) {
    // Child death and replacement: the new leaf gets no implicit storage grant.
    f.owner(1, [wire::STOP, 0, 0, 0, 0, 0, 0, 0]);
    f.owner(1, [wire::SPAWN, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(text(f, 2, 1), Err(abi::DENIED));
    f.owner(1, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]);
    expect(text(f, 2, 1).unwrap(), b"nested root");
    expect(text(f, 3, 1).unwrap(), b"outside root");
}
pub(super) fn sharing(f: &Fixture) {
    // Copies through two hops: a new number, revision one, identical bytes,
    // and the original's revision untouched.
    let copy = store::Reply::decode(&request(f, 2, packet(store::COPY, 1, 1, 0), &[])).unwrap();
    assert_eq!((copy.object, copy.revision, copy.length), (2, 1, 11));
    expect(text(f, 2, 2).unwrap(), b"nested root");
    let edited = publish(f, 2, 2, 1, b"nested copy").unwrap();
    assert_eq!(edited.revision, 2);
    expect(text(f, 2, 1).unwrap(), b"nested root");
    expect(text(f, 2, 2).unwrap(), b"nested copy");
    let ids = listing(f, 2).unwrap();
    assert_eq!(ids[1..4], [2, 1, 2]);
    assert_eq!(listing(f, 3).unwrap()[1], 1);
    assert_eq!(
        store::Reply::decode(&request(f, 2, packet(store::COPY, 9, 1, 0), &[])),
        Err(store::MISSING)
    );
}
