use super::*;

#[test]
fn producer_requires_declaration_and_explicit_offer() {
    let required = [
        Import::v1(INTERACTION, true),
        Import::v1(INPUT_PRODUCER, true),
    ];
    assert_eq!(ALL & INPUT_PRODUCER, 0);
    assert_eq!(bind(&required, ALL), Err(Refusal::Missing(INPUT_PRODUCER)));
    assert_eq!(bind(&[Import::v1(INPUT_PRODUCER, false)], ALL), Ok(0));
    assert_eq!(bind(&required, SUPPORTED), Ok(INTERACTION | INPUT_PRODUCER));
    assert_eq!(bind(&REQUIRED, SUPPORTED), Ok(ALL));
}

#[test]
fn required_optional_and_unrequested_services_have_distinct_bindings() {
    assert_eq!(bind(&REQUIRED, ALL), Ok(ALL));
    assert_eq!(bind(&REQUIRED, INTERACTION), Err(Refusal::Missing(STORAGE)));
    assert_eq!(bind(&OPTIONAL_STORAGE, INTERACTION), Ok(INTERACTION));
    assert_eq!(
        bind(&OPTIONAL_STORAGE, STORAGE),
        Err(Refusal::Missing(INTERACTION))
    );
    assert_eq!(bind(&[Import::v1(INTERACTION, true)], ALL), Ok(INTERACTION));
    assert_eq!(bind(&[Import::v1(99, false)], ALL), Ok(0));
    assert_eq!(
        bind(&[Import::v1(99, true)], ALL),
        Err(Refusal::Missing(99))
    );
}

#[test]
fn versions_are_checked_before_binding_and_optional_incompatibility_is_absent() {
    let mut future = Import {
        contract: STORAGE,
        minimum: 2,
        maximum: 3,
        required: true,
    };
    assert_eq!(bind(&[future], ALL), Err(Refusal::Version(STORAGE)));
    future.required = false;
    assert_eq!(bind(&[future], ALL), Ok(0));
    future.minimum = 1;
    assert_eq!(bind(&[future], ALL), Ok(STORAGE));
}

#[test]
fn declaration_rejects_ambiguous_or_malformed_entries() {
    assert_eq!(
        Declaration::decode(&encode(&OPTIONAL_STORAGE))
            .unwrap()
            .entries(),
        OPTIONAL_STORAGE
    );
    assert_eq!(
        Declaration::decode(&encode(&[REQUIRED[0], REQUIRED[0]])).unwrap_err(),
        Refusal::Malformed
    );
    let mut bytes = encode(&REQUIRED);
    bytes[40] = 2; // Required is a boolean, not permission flags.
    assert!(Declaration::decode(&bytes).is_err());
    let mut bytes = encode(&REQUIRED);
    bytes[8] = 5;
    assert!(Declaration::decode(&bytes).is_err());
    let mut bytes = encode(&REQUIRED);
    bytes[100] = 1; // Unused records must be zero, not hidden extra grants.
    assert!(Declaration::decode(&bytes).is_err());
    assert!(Declaration::decode(&bytes[..40]).is_err());
}

fn image() -> [u8; 512] {
    let mut bytes = [0; 512];
    bytes[..6].copy_from_slice(b"\x7fELF\x02\x01");
    bytes[32..40].copy_from_slice(&64u64.to_le_bytes());
    bytes[54..56].copy_from_slice(&56u16.to_le_bytes());
    bytes[56..58].copy_from_slice(&1u16.to_le_bytes());
    bytes[64..72].copy_from_slice(&[1, 0, 0, 0, 4, 0, 0, 0]);
    bytes[72..80].copy_from_slice(&128u64.to_le_bytes());
    bytes[96..104].copy_from_slice(&(RECORD_BYTES as u64).to_le_bytes());
    bytes[128..128 + RECORD_BYTES].copy_from_slice(&encode(&OPTIONAL_STORAGE));
    bytes
}

#[test]
fn imports_survive_missing_section_headers_and_truncated_headers_refuse() {
    let bytes = image();
    assert_eq!(
        from_elf(&bytes).unwrap().unwrap().entries(),
        OPTIONAL_STORAGE
    );
    for length in [0, 63, 100, 130, 128 + RECORD_BYTES - 1] {
        assert!(from_elf(&bytes[..length]).is_err());
    }
    let mut bytes = image();
    bytes[72..80].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(from_elf(&bytes).is_err());
}

#[test]
fn legacy_images_and_duplicate_declarations_are_distinguished() {
    let mut future = image();
    future[135] = b'9';
    assert!(from_elf(&future).is_err());
    let mut bytes = image();
    bytes[128..136].fill(0);
    assert!(from_elf(&bytes).unwrap().is_none());
    let mut bytes = image();
    bytes[56..58].copy_from_slice(&2u16.to_le_bytes());
    // Two read-only load headers cannot claim separate import declarations.
    bytes.copy_within(64..120, 120);
    bytes[72..80].copy_from_slice(&256u64.to_le_bytes());
    bytes[128..136].copy_from_slice(&256u64.to_le_bytes());
    bytes[256..256 + RECORD_BYTES].copy_from_slice(&encode(&REQUIRED));
    assert!(from_elf(&bytes).is_err());
}

#[test]
fn broker_bootstrap_is_explicit_and_keeps_import_semantics() {
    let mut bytes = image();
    bytes[128..128 + RECORD_BYTES].copy_from_slice(&encode_brokered(&OPTIONAL_STORAGE));
    let declaration = from_elf(&bytes).unwrap().unwrap();
    assert!(declaration.brokered);
    assert_eq!(bind(declaration.entries(), INTERACTION), Ok(INTERACTION));
    assert!(!Declaration::decode(&encode(&REQUIRED)).unwrap().brokered);
}

#[test]
fn requested_resources_are_bounded_metadata_and_do_not_add_services() {
    let mut bytes = image();
    bytes[128..128 + RECORD_BYTES].copy_from_slice(&encode_funded(&REQUIRED, 15, 3));
    let declaration = from_elf(&bytes).unwrap().unwrap();
    assert!(declaration.brokered);
    assert_eq!(declaration.resources, Some((15, 3)));
    assert_eq!(
        bind(declaration.entries(), INTERACTION),
        Err(Refusal::Missing(STORAGE))
    );
    assert_eq!(
        Declaration::decode(&encode_funded(&OPTIONAL_STORAGE, 0, 0))
            .unwrap()
            .resources,
        Some((0, 0))
    );
    assert_eq!(
        Declaration::decode(&encode_brokered(&REQUIRED))
            .unwrap()
            .resources,
        None
    );
    bytes[137] = 1; // Reserved header bits are not another permission channel.
    assert!(from_elf(&bytes).is_err());
}

#[test]
fn profile_requires_its_complete_load_segment_and_preserves_service_requirements() {
    let mut bytes = image();
    let profile = encode_profile(&REQUIRED, 40, 5, [320, 512, 24]);
    bytes[128..128 + PROFILE_BYTES].copy_from_slice(&profile);
    assert!(from_elf(&bytes).is_err()); // Old segment size cannot contain the extension.
    bytes[96..104].copy_from_slice(&(PROFILE_BYTES as u64).to_le_bytes());
    let declaration = from_elf(&bytes).unwrap().unwrap();
    assert!(declaration.brokered);
    assert_eq!(declaration.resources, Some((40, 5)));
    assert_eq!(declaration.profile, Some([320, 512, 24]));
    assert_eq!(
        bind(declaration.entries(), INTERACTION),
        Err(Refusal::Missing(STORAGE))
    );
    assert!(from_elf(&bytes[..128 + PROFILE_BYTES - 1]).is_err());
    bytes[128..136].copy_from_slice(b"CTHIMP04");
    assert!(from_elf(&bytes).is_err()); // Retired positional profile is not a broker.
    bytes[128..136].copy_from_slice(&PROFILE_MAGIC.to_le_bytes());
    bytes[137] = 1;
    assert!(from_elf(&bytes).is_err());
}
