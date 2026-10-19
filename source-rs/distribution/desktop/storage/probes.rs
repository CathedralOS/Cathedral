//! Scope isolation, stale bindings, replacement, content sharing, realm
//! attenuation and programs run from the realm, over real IPC. Object one of
//! root 0 is the program the harness installed; everything made here follows.
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
    // The installed program is the only thing in root 0; root 1 is empty.
    assert_eq!(listing(f, 2).unwrap()[1..3], [1, 1]);
    assert_eq!(listing(f, 3).unwrap()[1], 0);
    let nested = publish(f, 2, 0, 0, b"nested root").unwrap();
    assert_eq!((nested.object, nested.revision), (2, 1));
    // No sibling membership leaks: the outside root is still empty and its
    // own first object gets its own local number.
    assert_eq!(listing(f, 3).unwrap()[1], 0);
    assert_eq!(text(f, 3, 1), Err(store::MISSING));
    let outside = publish(f, 3, 0, 0, b"outside root").unwrap();
    assert_eq!(outside.object, 1);
    expect(text(f, 2, 2).unwrap(), b"nested root");
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
    f.owner(1, [wire::RESTORE_STORAGE, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(text(f, 2, 2), Err(abi::DENIED));
    f.owner(1, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        f.query(2, [wire::STORAGE_BASE + store::HEAD, 2, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    expect(text(f, 2, 2).unwrap(), b"nested root");
    // Revoking the ancestor's grant cuts the descendant; regranting the
    // ancestor never implicitly revives it.
    f.owner(0, [wire::STORAGE_REVOKE, 0, 0, 0, 0, 0, 0, 0]);
    f.owner(0, [wire::RESTORE_STORAGE, 0, 0, 0, 0, 0, 0, 0]);
    // Restore keeps the outside child's source 1; it cannot switch to root 0.
    expect(text(f, 3, 1).unwrap(), b"outside root");
    assert_eq!(publish(f, 2, 2, 1, b"must fail"), Err(abi::DENIED));
    f.owner(0, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(text(f, 2, 2), Err(abi::DENIED));
    f.owner(1, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]);
    expect(text(f, 2, 2).unwrap(), b"nested root");
    // A stale revision never overwrites; the refused write leaves no trace.
    assert_eq!(publish(f, 2, 2, 7, b"stale"), Err(abi::WOULD_BLOCK));
    expect(text(f, 2, 2).unwrap(), b"nested root");
}
pub(super) fn replacement(f: &Fixture) {
    // Child death and replacement: the new leaf gets no implicit storage grant.
    f.owner(1, [wire::STOP, 0, 0, 0, 0, 0, 0, 0]);
    f.owner(1, [wire::SPAWN, 0, 0, 0, 0, 0, 0, 0]);
    f.owner(1, [wire::RESTORE_STORAGE, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(text(f, 2, 2), Err(abi::DENIED));
    f.owner(1, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]);
    expect(text(f, 2, 2).unwrap(), b"nested root");
    expect(text(f, 3, 1).unwrap(), b"outside root");
}
pub(super) fn sharing(f: &Fixture) {
    // Copies through two hops: a new number, revision one, identical bytes,
    // and the original's revision untouched.
    let copy = copy(f, 2, 2, 1, 0).unwrap();
    assert_eq!((copy.object, copy.revision, copy.length), (3, 1, 11));
    expect(text(f, 2, 3).unwrap(), b"nested root");
    let edited = publish(f, 2, 3, 1, b"nested copy").unwrap();
    assert_eq!(edited.revision, 2);
    expect(text(f, 2, 2).unwrap(), b"nested root");
    expect(text(f, 2, 3).unwrap(), b"nested copy");
    let ids = listing(f, 2).unwrap();
    assert_eq!(ids[1..5], [3, 1, 2, 3]);
    assert_eq!(listing(f, 3).unwrap()[1], 1);
    assert_eq!(super::copy(f, 2, 9, 1, 0), Err(store::MISSING));
}
pub(super) fn attenuation(f: &Fixture) {
    // A program holding the whole root makes a subtree: an empty object at
    // the top. The owner can then grant the nested matrix that subtree alone.
    let realm = publish(f, 2, 0, 0, b"").unwrap();
    assert_eq!((realm.object, realm.revision, realm.length), (4, 1, 0));
    let realm = realm.object;
    // Granted the subtree, the nested matrix and its leaf see it as their realm
    // and nothing beside it; an object outside and one that does not exist
    // are refused alike.
    regrant_scoped(f, realm);
    assert_eq!(listing(f, 2).unwrap()[1], 0);
    assert_eq!(text(f, 2, 2), Err(abi::DENIED));
    assert_eq!(text(f, 2, 99), Err(abi::DENIED));
    let inside = publish(f, 2, 0, 0, b"inside realm").unwrap();
    assert_eq!((inside.object, inside.revision), (5, 1));
    assert_eq!(listing(f, 2).unwrap()[1..3], [1, 5]);
    // Copies cross the boundary in neither direction, and the leaf cannot
    // widen itself: a full packet folds only after a successful containment proof.
    assert_eq!(copy(f, 2, 2, 1, realm), Err(abi::DENIED));
    assert_eq!(copy(f, 2, 5, 1, 0), Err(abi::DENIED));
    assert_eq!(
        f.query(2, [wire::STORAGE_BASE + store::HEAD, 5, 0, 0, 0, 5, 0, 0])[0],
        0
    );
    assert_eq!(
        f.query(2, [wire::STORAGE_BASE + store::HEAD, 5, 0, 0, 0, 2, 0, 0]),
        wire::error(abi::DENIED)
    );
    assert_eq!(
        f.query(2, [wire::STORAGE_BASE + store::HEAD, 5, 0, 0, 0, 5, 2, 0]),
        wire::error(abi::DENIED)
    );
    assert_eq!(
        f.query(2, [wire::STORAGE_BASE + store::HEAD, 5, 0, 0, 0, 5, 5, 0])[0],
        0
    );
    assert_eq!(
        f.query(2, [wire::STORAGE_BASE + store::HEAD, 5, 0, 0, 0, 0, 0, 1]),
        wire::error(abi::TIMED_OUT)
    );
    // Refuse an already expired request before touching/retiring upstream IPC.
    assert_eq!(
        f.query(2, [wire::STORAGE_BASE + store::HEAD, 5, 0, 0, 0, 0, 0, 0])[0],
        0
    );
    // A subtree cannot be copied into itself.
    assert_eq!(copy_tree(f, 2, realm, 1, realm), Err(abi::INVALID_ARGUMENT));
    // Back at the whole root, the same program copies the subtree: the copy
    // and its object share content, and editing the copy leaves the original.
    // A subtree holding objects cannot be deleted from under them.
    regrant_scoped(f, 0);
    assert_eq!(
        store::Reply::decode(&request(
            f,
            2,
            store::Request::new(store::DELETE, realm, 1, 0, 0),
            &[]
        )),
        Err(abi::BUSY)
    );
    let copied = copy_tree(f, 2, realm, 1, 0).unwrap();
    assert_eq!((copied.object, copied.revision), (6, 1));
    regrant_scoped(f, copied.object);
    assert_eq!(listing(f, 2).unwrap()[1..3], [1, 7]);
    expect(text(f, 2, 7).unwrap(), b"inside realm");
    assert_eq!(publish(f, 2, 7, 1, b"inside copy").unwrap().revision, 2);
    regrant_scoped(f, realm);
    expect(text(f, 2, 5).unwrap(), b"inside realm");
    assert_eq!(text(f, 2, 7), Err(abi::DENIED));
    regrant_scoped(f, 0);
    assert_eq!(listing(f, 2).unwrap()[1..7], [5, 1, 2, 3, 4, 6]);
}
pub(super) fn launching(f: &Fixture) {
    // The root runs the installed program in the sibling's place. It is
    // admitted from the object itself and gets that object's subtree as its
    // realm: it can read its own code, list and create beneath it, and see
    // nothing beside it, but never replace or delete the code it runs from.
    f.owner(0, [wire::STOP, 1, 0, 0, 0, 0, 0, 0]);
    f.owner(0, [wire::RUN_PAGES, 1, 2, 0, 0, 0, 0, 0]);
    assert_eq!(
        f.query(0, [wire::RUN_OBJECT, 1, 99, 99, 0, 0, 0, 0]),
        wire::error(store::MISSING)
    );
    f.owner(0, [wire::RUN_OBJECT, 1, 1, 1, 0, 0, 0, 0]);
    assert_eq!(realm(f, 3), Ok(1));
    assert_eq!(listing(f, 3).unwrap()[1], 0);
    expect(prefix(f, 3, 1, 4).unwrap(), b"\x7fELF");
    let state = publish(f, 3, 0, 0, b"program state").unwrap();
    assert_eq!((state.object, state.revision), (8, 1));
    assert_eq!(listing(f, 3).unwrap()[1..3], [1, 8]);
    assert_eq!(text(f, 3, 2), Err(abi::DENIED));
    assert_eq!(publish(f, 3, 1, 1, b"code"), Err(abi::DENIED));
    assert_eq!(
        store::Reply::decode(&request(
            f,
            3,
            store::Request::new(store::DELETE, 1, 1, 0, 0),
            &[]
        )),
        Err(abi::DENIED)
    );
    // The whole-root holder still sees the program and its state beneath it.
    assert_eq!(listing(f, 2).unwrap()[1..7], [5, 1, 2, 3, 4, 6]);
}
pub(super) fn sibling(f: &Fixture) {
    // The program copies itself into its realm, cheaply, and asks its host to
    // run the copy as a sibling. The host reads it with the requester's scope,
    // admits it into the slot the owner opened, and grants it the copy's
    // subtree. What is not a program is refused by the kernel; what lies
    // outside the requester's realm is refused by the provider.
    let copy = copy(f, 3, 1, 1, 1).unwrap();
    assert_eq!((copy.object, copy.revision), (9, 1));
    assert_eq!(
        f.query(3, [wire::LAUNCH, 9, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::BUSY)
    );
    f.owner(0, [wire::STOP, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        f.query(3, [wire::LAUNCH, 9, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    f.owner(0, [wire::LAUNCH_POLICY, 0, 2, 0, 0, 0, 0, 0]);
    // Declared required storage cannot be manufactured by a child request.
    // Refusals happen before admission and leave funding available for retry.
    let imports = cathedral_platform_contracts::imports::INTERACTION;
    f.owner(0, [wire::LAUNCH_IMPORTS, 0, imports, 0, 0, 0, 0, 0]);
    assert_eq!(
        f.query(3, [wire::LAUNCH, 9, 0, 0, 0, 0, 0, 0]),
        wire::error(cathedral_platform_contracts::imports::MISSING)
    );
    assert_eq!(
        f.query(3, [wire::LAUNCH_IMPORTS, 0, 3, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    f.owner(
        0,
        [
            wire::LAUNCH_IMPORTS,
            0,
            cathedral_platform_contracts::imports::ALL,
            0,
            0,
            0,
            0,
            0,
        ],
    );
    assert_eq!(
        f.query(
            3,
            [
                wire::LAUNCH,
                9,
                cathedral_platform_contracts::imports::STORAGE,
                0,
                0,
                0,
                0,
                0
            ]
        ),
        wire::error(cathedral_platform_contracts::imports::MISSING)
    );
    assert_eq!(
        f.query(3, [wire::LAUNCH, 9, 4, 0, 0, 0, 0, 0]),
        wire::error(abi::INVALID_ARGUMENT)
    );

    assert_eq!(
        f.query(3, [wire::LAUNCH, 8, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::BAD_EXECUTABLE)
    );
    assert_eq!(
        f.query(3, [wire::LAUNCH, 2, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    // This headless host has no surface capacity. Placement failure must reclaim
    // the admitted child and its funding, not report an invisible running app.
    f.owner(0, [wire::LAUNCH_POLICY, 0, 2, 0, 0, 0, 0, 1]);
    for _ in 0..3 {
        assert_eq!(
            f.query(3, [wire::LAUNCH, 9, 0, 0, 0, 0, 0, 0]),
            wire::error(abi::NO_MEMORY)
        );
        assert_eq!(
            f.query(3, [wire::LAUNCH_STATUS, 1, 0, 0, 0, 0, 0, 0]),
            wire::error(abi::DENIED)
        );
    }
    f.owner(0, [wire::LAUNCH_POLICY, 0, 2, 0, 0, 0, 0, 0]);
    let launched = f.query(3, [wire::LAUNCH, 9, 0, 0, 0, 0, 0, 0]);
    assert_eq!(launched, [0, 0, 1, 0, 0, 0, 0, 0]);
    assert_eq!(
        f.query(3, [wire::LAUNCH_STATUS, 1, 0, 0, 0, 0, 0, 0])[1],
        wire::LAUNCH_RUNNING
    );
    assert_eq!(
        f.query(1, [wire::LAUNCH_STATUS, 1, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    assert_eq!(realm(f, 1), Ok(9));
    let state = publish(f, 1, 0, 0, b"sibling state").unwrap();
    assert_eq!((state.object, state.revision), (10, 1));
    assert_eq!(text(f, 1, 8), Err(abi::DENIED));
    assert_eq!(listing(f, 3).unwrap()[1..4], [2, 8, 9]);
    // The launcher may ask for its sibling to go; the host asks the sibling
    // first, over the same forwarded control the owner's probes use, so the
    // leaf asks on its own and the owner collects the answer afterwards.
    f.owner(3, [leaf::DETACHED_PROBE, wire::CLOSE, 0, 0, 0, 0, 0, 0]);
    assert_eq!(f.query(3, [leaf::LAST_PROBE, 0, 0, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(
        f.query(3, [wire::LAUNCH_STATUS, 1, 0, 0, 0, 0, 0, 0])[1],
        wire::LAUNCH_STOPPED
    );
    // The next successful launch invalidates the old receipt. An owner-launched
    // replacement afterwards must not inherit that result or expose its state.
    assert_eq!(
        f.query(3, [wire::LAUNCH, 9, 0, 0, 0, 0, 0, 0]),
        [0, 0, 2, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        f.query(3, [wire::LAUNCH_STATUS, 1, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    assert_eq!(
        f.query(3, [wire::LAUNCH_STATUS, 2, 0, 0, 0, 0, 0, 0])[1],
        wire::LAUNCH_RUNNING
    );
    f.owner(1, [leaf::DROP_CONTROL_PROBE, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        f.query(1, [wire::HEALTH, 0, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::BUSY)
    );
    f.owner(3, [leaf::DETACHED_PROBE, wire::CLOSE, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        f.query(3, [leaf::LAST_PROBE, 0, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::BUSY)
    );
    assert_eq!(
        f.query(3, [wire::LAUNCH_STATUS, 2, 0, 0, 0, 0, 0, 0])[1],
        wire::LAUNCH_RUNNING
    );
    f.owner(0, [wire::STOP, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        f.query(3, [wire::LAUNCH, 9, 0, 0, 0, 0, 0, 0]),
        [0, 0, 3, 0, 0, 0, 0, 0]
    );
    f.owner(1, [leaf::DROP_LOSS_REPLY_PROBE, 0, 0, 0, 0, 0, 0, 0]);
    f.owner(3, [leaf::DETACHED_PROBE, wire::CLOSE, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        f.query(3, [leaf::LAST_PROBE, 0, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::BUSY)
    );
    assert_eq!(
        f.query(3, [wire::LAUNCH_STATUS, 3, 0, 0, 0, 0, 0, 0])[1],
        wire::LAUNCH_RUNNING
    );
    // Owner health routing also reports uncertainty rather than claiming death.
    assert_eq!(
        f.query(1, [wire::HEALTH, 0, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::BUSY)
    );
    f.owner(0, [wire::STOP, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        f.query(3, [wire::LAUNCH, 9, 0, 0, 0, 0, 0, 0]),
        [0, 0, 4, 0, 0, 0, 0, 0]
    );
    f.owner(1, [leaf::LOSS_PROBE, 0, 150, 0, 0, 0, 0, 0]);
    f.owner(3, [leaf::DETACHED_PROBE, wire::CLOSE, 0, 0, 0, 0, 0, 0]);
    f.owner(0, [wire::STOP, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(f.query(3, [leaf::LAST_PROBE, 0, 0, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(
        f.query(3, [wire::LAUNCH, 9, 0, 0, 0, 0, 0, 0]),
        [0, 0, 5, 0, 0, 0, 0, 0]
    );
    f.owner(1, [leaf::EXIT_ON_LOSS_PROBE, 0, 0, 0, 0, 0, 0, 0]);
    assert_ne!(f.query(0, [wire::LOSS, 0, 0, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(
        f.query(3, [wire::LAUNCH_STATUS, 5, 0, 0, 0, 0, 0, 0])[1],
        wire::LAUNCH_EXITED
    );
    f.owner(0, [wire::RUN_PAGES, 0, 2, 0, 0, 0, 0, 0]);
    f.owner(0, [wire::RUN_OBJECT, 0, 9, 9, 0, 0, 0, 0]);
    assert_eq!(
        f.query(3, [wire::LAUNCH_STATUS, 5, 0, 0, 0, 0, 0, 0])[1],
        wire::LAUNCH_EXITED
    );
    assert_eq!(
        f.query(1, [wire::LAUNCH_STATUS, 5, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    f.owner(0, [wire::STOP, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        f.query(1, [wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::PEER_CLOSED)
    );
    f.owner(3, [leaf::DETACHED_PROBE, wire::CLOSE, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        f.query(3, [leaf::LAST_PROBE, 0, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::PEER_CLOSED)
    );
}
