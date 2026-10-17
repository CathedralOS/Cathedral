//! Scope isolation, stale bindings, replacement, content sharing and subtree
//! delegation over real IPC.
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
    let copy = copy(f, 2, 1, 1, 0).unwrap();
    assert_eq!((copy.object, copy.revision, copy.length), (2, 1, 11));
    expect(text(f, 2, 2).unwrap(), b"nested root");
    let edited = publish(f, 2, 2, 1, b"nested copy").unwrap();
    assert_eq!(edited.revision, 2);
    expect(text(f, 2, 1).unwrap(), b"nested root");
    expect(text(f, 2, 2).unwrap(), b"nested copy");
    let ids = listing(f, 2).unwrap();
    assert_eq!(ids[1..4], [2, 1, 2]);
    assert_eq!(listing(f, 3).unwrap()[1], 1);
    assert_eq!(super::copy(f, 2, 9, 1, 0), Err(store::MISSING));
}
pub(super) fn scoping(f: &Fixture) {
    // The owner carves a world out of root 0 on the matrix's own authority;
    // a child has no such authority on its data connection.
    let world = owner_request(f, store::PUBLISH, 0, 0, 0);
    assert_eq!((world[0], world[1], world[2]), (0, 3, 1));
    let world = world[1];
    assert_eq!(
        f.query(2, [wire::STORAGE_REQUEST, 0, store::LIST, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    // Granted the world, the nested matrix and its leaf see it as their home
    // and nothing beside it; an object outside and one that does not exist
    // are refused alike.
    regrant_scoped(f, world);
    assert_eq!(listing(f, 2).unwrap()[1], 0);
    assert_eq!(text(f, 2, 1), Err(abi::DENIED));
    assert_eq!(text(f, 2, 99), Err(abi::DENIED));
    let inside = publish(f, 2, 0, 0, b"inside world").unwrap();
    assert_eq!((inside.object, inside.revision), (4, 1));
    assert_eq!(listing(f, 2).unwrap()[1..3], [1, 4]);
    // Copies cross the boundary in neither direction, and the leaf cannot
    // widen itself: scope words it fills only narrow, and filling them all
    // leaves the matrix no room for its stamp.
    assert_eq!(copy(f, 2, 1, 1, world), Err(abi::DENIED));
    assert_eq!(copy(f, 2, 4, 1, 0), Err(abi::DENIED));
    assert_eq!(
        f.query(2, [wire::STORAGE_BASE + store::HEAD, 4, 0, 0, 0, 4, 0, 0])[0],
        0
    );
    assert_eq!(
        f.query(2, [wire::STORAGE_BASE + store::HEAD, 4, 0, 0, 0, 1, 0, 0]),
        wire::error(abi::DENIED)
    );
    assert_eq!(
        f.query(2, [wire::STORAGE_BASE + store::HEAD, 4, 0, 0, 0, 4, 4, 4]),
        wire::error(abi::DENIED)
    );
    // The owner clones the world: the copy and its note share content, and
    // editing the copy's note leaves the original world as it was.
    assert_eq!(
        owner_request(f, store::COPY_TREE, world, 1, world),
        wire::error(abi::INVALID_ARGUMENT)
    );
    let copied = owner_request(f, store::COPY_TREE, world, 1, 0);
    assert_eq!((copied[0], copied[1], copied[2]), (0, 5, 1));
    regrant_scoped(f, copied[1]);
    assert_eq!(listing(f, 2).unwrap()[1..3], [1, 6]);
    expect(text(f, 2, 6).unwrap(), b"inside world");
    assert_eq!(publish(f, 2, 6, 1, b"inside copy").unwrap().revision, 2);
    regrant_scoped(f, world);
    expect(text(f, 2, 4).unwrap(), b"inside world");
    assert_eq!(text(f, 2, 6), Err(abi::DENIED));
    // A world with notes in it cannot be deleted from under them.
    assert_eq!(
        owner_request(f, store::DELETE, world, 1, 0),
        wire::error(abi::BUSY)
    );
    regrant_scoped(f, 0);
    assert_eq!(listing(f, 2).unwrap()[1..6], [4, 1, 2, 3, 5]);
}
