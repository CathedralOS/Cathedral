use super::*;

#[test]
fn producer_parts_are_scoped_and_never_added_to_ordinary_offers() {
    for (part, index) in [(0, 3), (1, 4)] {
        let words = request(imports::INPUT_PRODUCER, 1, 1, part);
        assert_eq!(select(imports::ALL, words), Err(UNAVAILABLE));
        assert_eq!(select(imports::SUPPORTED, words), Ok(index));
    }
    assert_eq!(
        select(
            imports::SUPPORTED,
            request(imports::INPUT_PRODUCER, 1, 1, 2)
        ),
        Err(INVALID)
    );
    assert_eq!(
        select(
            imports::SUPPORTED,
            request(imports::INPUT_PRODUCER, 2, 2, 0)
        ),
        Err(INCOMPATIBLE)
    );
}
#[test]
fn acquisition_cannot_widen_the_resolved_manifest() {
    let grants = imports::bind(
        &[imports::Import::v1(imports::INTERACTION, true)],
        imports::ALL,
    )
    .unwrap();
    assert_eq!(
        select(grants, request(imports::STORAGE, 1, 1, 0)),
        Err(UNAVAILABLE)
    );
    let grants = imports::bind(&imports::OPTIONAL_STORAGE, imports::INTERACTION).unwrap();
    assert_eq!(
        select(grants, request(imports::STORAGE, 1, 1, 1)),
        Err(UNAVAILABLE)
    );
    assert_eq!(select(grants, request(99, 1, 1, 0)), Err(UNAVAILABLE));
    assert_eq!(
        select(grants, request(imports::INTERACTION, 1, 1, 0)),
        Ok(0)
    );
}
#[test]
fn version_shape_and_endpoint_selection_are_independent() {
    assert_eq!(
        select(imports::ALL, request(imports::STORAGE, 1, 2, 0)),
        Ok(1)
    );
    assert_eq!(
        select(imports::ALL, request(imports::STORAGE, 1, 1, 1)),
        Ok(2)
    );
    assert_eq!(
        select(imports::ALL, request(imports::STORAGE, 2, 2, 0)),
        Err(INCOMPATIBLE)
    );
    assert_eq!(
        select(imports::ALL, request(imports::STORAGE, 0, 1, 0)),
        Err(INVALID)
    );
    let mut forged = request(imports::INTERACTION, 1, 1, 0);
    forged[6] = 123; // No caller-selected identity/scope extension.
    assert_eq!(select(imports::ALL, forged), Err(INVALID));
    assert_eq!(
        select(imports::ALL, request(imports::STORAGE, 1, 1, 2)),
        Err(INVALID)
    );
}

#[test]
fn capture_is_explicit_and_distinct_from_production_and_storage() {
    for (part, index) in [(0, 5), (1, 6)] {
        let words = request(imports::CAPTURE, 1, 1, part);
        assert_eq!(
            select(imports::ALL | imports::INPUT_PRODUCER, words),
            Err(UNAVAILABLE)
        );
        assert_eq!(select(imports::SUPPORTED, words), Ok(index));
    }
    assert_eq!(
        select(imports::SUPPORTED, request(imports::CAPTURE, 1, 1, 2)),
        Err(INVALID)
    );
    assert_eq!(
        imports::bind(&[imports::Import::v1(imports::CAPTURE, true)], imports::ALL),
        Err(imports::Refusal::Missing(imports::CAPTURE))
    );
    assert_eq!(
        imports::bind(&imports::REQUIRED, imports::SUPPORTED),
        Ok(imports::ALL)
    );
}
