use super::*;
#[test]
fn sources_are_exclusive_connection_bound_and_regrant_is_fresh() {
    let mut scopes = Scopes::<2>::default();
    scopes.grant(0, 7, 0, 0, 2).unwrap();
    assert_eq!(scopes.grant(1, 8, 0, 0, 2), Err(abi::BUSY));
    scopes.grant(1, 8, 1, 5, 2).unwrap();
    assert_eq!(scopes.binding(0, 8), Err(abi::DENIED));
    assert_eq!(scopes.binding(0, 7), Ok((0, 0)));
    assert_eq!(scopes.binding(1, 8), Ok((1, 5)));
    let sibling = scopes.revision(1, 8);
    let old = scopes.revision(0, 7).unwrap();
    assert_eq!(scopes.revoke(0), Some(0));
    assert_eq!(scopes.binding(0, 7), Err(abi::DENIED));
    // The same connection regranted with a scope gets a fresh revision, so
    // nothing authorized under the wider binding carries over.
    scopes.grant(0, 9, 0, 3, 2).unwrap();
    assert!(scopes.revision(0, 9).unwrap() > old);
    assert_eq!(scopes.authorize(0, 9, old), Err(abi::DENIED));
    assert_eq!(
        scopes.authorize(0, 9, scopes.revision(0, 9).unwrap()),
        Ok((0, 3))
    );
    assert_eq!(scopes.revision(1, 8), sibling);
    scopes.clear();
    assert_eq!(scopes.binding(0, 9), Err(abi::DENIED));
    assert_eq!(scopes.grant(0, 9, 1, 0, 1), Err(abi::DENIED));
    // A subtree may be held beside a whole source or another subtree; only
    // two whole-source grants of one source conflict.
    scopes.grant(0, 9, 0, 3, 2).unwrap();
    scopes.grant(1, 8, 0, 5, 2).unwrap();
    assert_eq!(scopes.binding(1, 8), Ok((0, 5)));
    scopes.revoke(1);
    scopes.grant(1, 8, 0, 0, 2).unwrap();
    scopes.revoke(0);
    assert_eq!(scopes.grant(0, 9, 0, 0, 2), Err(abi::BUSY));
    scopes.grant(0, 9, 0, 3, 2).unwrap();
}

#[test]
fn forwarded_requests_require_live_revision_and_never_drop_scope_stamps() {
    let mut scopes = Scopes::<2>::empty();
    scopes.grant(0, 7, 0, 40, 1).unwrap();
    let revision = scopes.revision(0, 7).unwrap();
    let mut words = [
        (wire::STORAGE_BASE + protocol::HEAD) | (revision << wire::STORAGE_SCOPE_BITS),
        999,
        0,
        0,
        0,
        10,
        20,
        0,
    ];
    match scopes.prepare(0, 7, words, 1).unwrap() {
        Prepared::Forward { source, words } => {
            assert_eq!(source, 0);
            assert_eq!(words[5..], [10, 20, 40]);
        }
        _ => panic!("expected forwarding"),
    }
    words[7] = 30;
    assert!(matches!(scopes.prepare(0, 7, words, 1), Err(abi::DENIED)));
    words[7] = 0;
    assert!(matches!(scopes.prepare(0, 8, words, 1), Err(abi::DENIED)));
    scopes.clear();
    scopes.grant(0, 7, 0, 40, 1).unwrap();
    assert!(matches!(scopes.prepare(0, 7, words, 1), Err(abi::DENIED)));
    match scopes
        .prepare(0, 7, [wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0], 1)
        .unwrap()
    {
        Prepared::Reply(reply) => {
            assert!(reply[1] > revision);
            assert_eq!(reply[2], 40);
        }
        _ => panic!("expected local binding"),
    }
}
