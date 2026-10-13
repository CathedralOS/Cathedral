use super::*;
#[test]
fn sources_are_exclusive_connection_bound_and_regrant_is_fresh() {
    let mut scopes = Scopes::default();
    scopes.grant(0, 7, 0, 2).unwrap();
    assert_eq!(scopes.grant(1, 8, 0, 2), Err(abi::BUSY));
    scopes.grant(1, 8, 1, 2).unwrap();
    assert_eq!(scopes.source(0, 8), Err(abi::DENIED));
    assert_eq!(scopes.source(0, 7), Ok(0));
    let sibling = scopes.revision(1, 8);
    let old = scopes.revision(0, 7).unwrap();
    assert_eq!(scopes.revoke(0), Some(0));
    assert_eq!(scopes.source(0, 7), Err(abi::DENIED));
    scopes.grant(0, 9, 0, 2).unwrap();
    assert!(scopes.revision(0, 9).unwrap() > old);
    assert_eq!(scopes.authorize(0, 9, old), Err(abi::DENIED));
    assert_eq!(
        scopes.authorize(0, 9, scopes.revision(0, 9).unwrap()),
        Ok(0)
    );
    assert_eq!(scopes.revision(1, 8), sibling);
    scopes.clear();
    assert_eq!(scopes.source(0, 9), Err(abi::DENIED));
    assert_eq!(scopes.grant(0, 9, 1, 1), Err(abi::DENIED));
}
