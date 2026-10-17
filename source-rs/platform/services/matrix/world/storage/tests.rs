use super::*;
#[test]
fn sources_are_exclusive_connection_bound_and_regrant_is_fresh() {
    let mut scopes = Scopes::default();
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
}
