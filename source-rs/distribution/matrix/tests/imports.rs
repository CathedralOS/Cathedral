//! Compare compiled Omega matching with the existing platform contract oracle.
use crate::policy::imports as policy;
use cathedral_platform_contracts::imports::{self, Declaration, Import, Refusal};
#[test]
fn common_matching_preserves_versions_optionality_and_precise_refusals() {
    for contract in [0, 1, 2, 3, 4, 8, 16, u64::MAX] {
        for minimum in [0, 1, 2, u64::MAX] {
            for maximum in [0, 1, 2, u64::MAX] {
                for required in [false, true] {
                    let row = Import {
                        contract,
                        minimum,
                        maximum,
                        required,
                    };
                    for offered in 0..32 {
                        for entries in [&[row][..], &[Import::v1(1, false), row][..]] {
                            assert_eq!(
                                policy::bind(entries, offered),
                                imports::bind(entries, offered),
                                "{entries:?} offered={offered}"
                            );
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn matcher_rejects_duplicates_and_excess_without_accidentally_publishing_a_binding() {
    for count in 0..=5 {
        let entries = [
            Import::v1(1, false),
            Import::v1(2, false),
            Import::v1(4, false),
            Import::v1(8, false),
            Import::v1(16, false),
        ];
        assert_eq!(
            policy::bind(&entries[..count], 15),
            imports::bind(&entries[..count], 15)
        );
    }
    for contract in [1, 2, 4, 8, 16, u64::MAX] {
        let rows = [Import::v1(contract, false); 2];
        assert_eq!(policy::bind(&rows, 15), Err(Refusal::Malformed));
    }
    // Preserve the common matcher's ordered refusal. Image decoding separately
    // validates all syntax before any owner-specific binding is attempted.
    let rows = [Import::v1(8, true), Import::v1(0, true)];
    assert_eq!(policy::bind(&rows, 1), Err(Refusal::Missing(8)));
    assert_eq!(policy::bind(&rows, 16), Err(Refusal::Malformed));
}
fn declaration(entries: &[Import], brokered: bool) -> Declaration {
    Declaration::decode(&if brokered {
        imports::encode_brokered(entries)
    } else {
        imports::encode(entries)
    })
    .unwrap()
}
#[test]
fn stock_offer_never_invents_capture_or_a_positional_producer() {
    for brokered in [false, true] {
        for offered in [0, 1, 3, 7, 15, u64::MAX] {
            let d = declaration(
                &[
                    Import::v1(1, true),
                    Import::v1(2, false),
                    Import::v1(4, false),
                    Import::v1(8, false),
                ],
                brokered,
            );
            let expected = if offered & 1 == 0 {
                Err(Refusal::Missing(1))
            } else {
                Ok(offered & if brokered { 7 } else { 3 })
            };
            assert_eq!(policy::resolve(Some(&d), offered), expected);
        }
    }
    let capture = declaration(&[Import::v1(1, true), Import::v1(8, true)], true);
    assert_eq!(
        policy::resolve(Some(&capture), 15),
        Err(Refusal::Missing(8))
    );
    let producer = declaration(&[Import::v1(1, true), Import::v1(4, true)], false);
    assert_eq!(
        policy::resolve(Some(&producer), 7),
        Err(Refusal::Missing(4))
    );
}
#[test]
fn stock_requires_interaction_and_legacy_requires_both_original_connections() {
    assert_eq!(policy::resolve(None, 1), Err(Refusal::Missing(2)));
    assert_eq!(policy::resolve(None, 2), Err(Refusal::Missing(1)));
    assert_eq!(policy::resolve(None, u64::MAX), Ok(3));
    for entries in [&[][..], &[Import::v1(2, false)][..]] {
        let d = declaration(entries, true);
        assert_eq!(policy::resolve(Some(&d), 7), Err(Refusal::Missing(1)));
    }
    let d = declaration(
        &[Import {
            contract: 1,
            minimum: 2,
            maximum: 3,
            required: true,
        }],
        true,
    );
    assert_eq!(policy::resolve(Some(&d), 7), Err(Refusal::Version(1)));
    let optional = declaration(&[Import::v1(1, true), Import::v1(2, false)], true);
    assert_eq!(policy::resolve(Some(&optional), 1), Ok(1));
}
