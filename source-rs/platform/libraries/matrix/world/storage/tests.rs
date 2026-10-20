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
fn forwarded_requests_require_live_revision_and_a_proof_before_folding() {
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
        0,
        1200,
    ];
    match scopes.prepare(0, 7, words, 1).unwrap() {
        Prepared::Forward { source, words } => {
            assert_eq!(source, 0);
            assert_eq!(words[5..], [10, 40, 1200]);
        }
        _ => panic!("expected forwarding"),
    }
    words[6] = 20;
    match scopes.prepare(0, 7, words, 1).unwrap() {
        Prepared::Validate {
            check,
            words: folded,
            ..
        } => {
            assert_eq!(check[1], 10);
            assert_eq!(check[5..], [20, 40, 1200]);
            assert_eq!(folded[5..], [10, 0, 1200]);
        }
        _ => panic!("must prove containment before forwarding"),
    }
    words[6] = 0;
    assert!(matches!(scopes.prepare(0, 8, words, 1), Err(abi::DENIED)));
    scopes.clear();
    scopes.grant(0, 7, 0, 40, 1).unwrap();
    assert!(matches!(scopes.prepare(0, 7, words, 1), Err(abi::DENIED)));
    match scopes
        .prepare(0, 7, [wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 1200], 1)
        .unwrap()
    {
        Prepared::Reply(reply) => {
            assert!(reply[1] > revision);
            assert_eq!(reply[2], 40);
        }
        _ => panic!("expected local binding"),
    }
}

#[test]
fn repeated_folding_preserves_ancestor_and_sibling_confinement() {
    // Model an immutable chain 1/2/.../9 and an unrelated sibling 99. Run the
    // same proof/forward protocol recursively, not just its packet encoding.
    fn forward(
        chain: &mut [Scopes<1>],
        mut words: wire::Packet,
        counts: &mut [usize; 2],
        cache: bool,
    ) -> Result<(), u64> {
        let level = chain.len() as u64;
        if level == 0 {
            counts[0] += 1;
            let request = protocol::Request::decode(&protocol::pack(words)).unwrap();
            let (subjects, count) = request.subjects();
            for &scope in request.scopes.iter().filter(|s| **s != 0) {
                for &subject in &subjects[..count] {
                    if subject < scope || subject > 9 {
                        return Err(abi::DENIED);
                    }
                }
                if request.op == protocol::PUBLISH && request.object == scope {
                    return Err(abi::DENIED);
                }
            }
            if request.op == protocol::PUBLISH {
                counts[1] += 1;
            }
            return Ok(());
        }
        let (scopes, ancestors) = chain.split_last_mut().unwrap();
        words[0] = (words[0] & 0xffff) | (1 << wire::STORAGE_SCOPE_BITS);
        let narrowed = match scopes.prepare(0, 1, words, level - 1)? {
            Prepared::Forward { words, .. } => words,
            Prepared::Validate {
                check,
                words,
                proof,
                ..
            } => {
                let mut check = check;
                if level == 1 {
                    check[0] = (check[0] & 0xffff) - wire::STORAGE_BASE;
                }
                forward(ancestors, check, counts, cache)?;
                scopes.authorize(0, 1, 1)?;
                if cache {
                    scopes.confirm(proof)?;
                }
                words
            }
            Prepared::Reply(_) => unreachable!(),
        };
        let mut narrowed = narrowed;
        if level == 1 {
            narrowed[0] = (narrowed[0] & 0xffff) - wire::STORAGE_BASE;
        }
        forward(ancestors, narrowed, counts, cache)
    }
    let mut chain = core::array::from_fn::<_, 8, _>(|index| {
        let mut scopes = Scopes::<1>::empty();
        scopes.grant(0, 1, 0, index as u64 + 1, 1).unwrap();
        scopes
    });
    let mut counts = [0, 0];
    let mut words = protocol::Request::new(protocol::PUBLISH, 9, 1, 0, 0).words();
    words[0] += wire::STORAGE_BASE;
    words[7] = 1200;
    forward(&mut chain, words, &mut counts, false).unwrap();
    assert_eq!(counts, [21, 1]); // One write expanded to twenty ancestry HEADs.
    counts = [0, 0];
    forward(&mut chain, words, &mut counts, true).unwrap();
    assert_eq!(counts[1], 1);
    counts = [0, 0];
    forward(&mut chain, words, &mut counts, true).unwrap();
    assert_eq!(counts, [1, 1]); // A warm chain forwards only the actual request.
    for object in [1, 7, 8, 99] {
        words[1] = object;
        assert_eq!(
            forward(&mut chain, words, &mut counts, true),
            Err(abi::DENIED)
        );
    }
    assert_eq!(counts[1], 1);
    words[1] = 9;
    words[5] = 99; // A forged inner restriction cannot erase the actual realm.
    assert_eq!(
        forward(&mut chain, words, &mut counts, true),
        Err(abi::DENIED)
    );
    assert_eq!(counts[1], 1);
}

#[test]
fn proofs_require_success_and_die_with_the_binding_or_source() {
    let mut scopes = Scopes::<3>::empty();
    scopes.grant(0, 7, 0, 10, 2).unwrap();
    scopes.grant(1, 8, 1, 10, 2).unwrap();
    scopes.grant(2, 9, 0, 10, 2).unwrap();
    let words = [
        wire::STORAGE_BASE + protocol::PUBLISH | (1 << wire::STORAGE_SCOPE_BITS),
        90,
        1,
        0,
        0,
        30,
        20,
        1200,
    ];
    let proof = match scopes.prepare(0, 7, words, 1).unwrap() {
        Prepared::Validate { proof, .. } => proof,
        _ => panic!("uncached check required"),
    };
    // A failed, timed-out or unconsumed check cannot populate the cache.
    assert!(matches!(
        scopes.prepare(0, 7, words, 1),
        Ok(Prepared::Validate { .. })
    ));
    scopes.confirm(proof).unwrap();
    assert!(matches!(
        scopes.prepare(0, 7, words, 1),
        Ok(Prepared::Forward { .. })
    ));
    // Same numbers on other connections/sources do not inherit the proof.
    assert!(matches!(
        scopes.prepare(1, 8, words, 1),
        Ok(Prepared::Validate { .. })
    ));
    assert!(matches!(
        scopes.prepare(2, 9, words, 1),
        Ok(Prepared::Validate { .. })
    ));
    if let Prepared::Validate { proof, .. } = scopes.prepare(1, 8, words, 1).unwrap() {
        scopes.confirm(proof).unwrap();
    }
    for (inner, outer) in [(99, 20), (30, 99), (20, 30)] {
        let mut forged = words;
        forged[5] = inner;
        forged[6] = outer;
        assert!(matches!(
            scopes.prepare(0, 7, forged, 1),
            Ok(Prepared::Validate { .. })
        ));
    }
    scopes.revoke_source(0);
    assert_eq!(scopes.confirm(proof), Err(abi::DENIED));
    assert!(scopes.binding(2, 9).is_err());
    assert_eq!(scopes.binding(1, 8), Ok((1, 10)));
    assert!(matches!(
        scopes.prepare(1, 8, words, 1),
        Ok(Prepared::Forward { .. })
    ));
    scopes.grant(0, 7, 0, 10, 2).unwrap();
    assert_eq!(scopes.confirm(proof), Err(abi::DENIED));
    assert!(matches!(scopes.prepare(0, 7, words, 1), Err(abi::DENIED)));
    let mut fresh = words;
    fresh[0] = wire::STORAGE_BASE + protocol::PUBLISH | (2 << wire::STORAGE_SCOPE_BITS);
    assert!(matches!(
        scopes.prepare(0, 7, fresh, 1),
        Ok(Prepared::Validate { .. })
    ));
    scopes.clear();
    assert_eq!(scopes.confirm(proof), Err(abi::DENIED));
}

#[test]
fn proof_eviction_rechecks_instead_of_broadening_the_binding() {
    let mut scopes = Scopes::<1>::empty();
    scopes.grant(0, 1, 0, 10, 1).unwrap();
    let mut words = [
        wire::STORAGE_BASE + protocol::HEAD | (1 << wire::STORAGE_SCOPE_BITS),
        90,
        0,
        0,
        0,
        30,
        20,
        1200,
    ];
    for inner in 30..30 + PROOFS as u64 + 1 {
        words[5] = inner;
        let Prepared::Validate { proof, .. } = scopes.prepare(0, 1, words, 1).unwrap() else {
            panic!("new restriction must be checked");
        };
        scopes.confirm(proof).unwrap();
    }
    words[5] = 30;
    assert!(matches!(
        scopes.prepare(0, 1, words, 1),
        Ok(Prepared::Validate { .. })
    ));
    words[5] += PROOFS as u64;
    assert!(matches!(
        scopes.prepare(0, 1, words, 1),
        Ok(Prepared::Forward { .. })
    ));
}
