//! Compiled Omega parses untrusted record bytes; Rust remains the independent oracle.
use crate::policy::imports as policy;
use cathedral_platform_contracts::imports::{self, Declaration, Import, Refusal};
use std::{vec, vec::Vec};

fn compare(actual: Result<policy::Declaration, Refusal>, expected: Result<Declaration, Refusal>) {
    match (actual, expected) {
        (Ok(actual), Ok(expected)) => {
            assert_eq!(actual.entries(), expected.entries());
            assert_eq!(actual.brokered, expected.brokered);
            assert_eq!(actual.profile, expected.profile);
            assert_eq!(actual.resources, expected.resources);
        }
        (Err(actual), Err(expected)) => assert_eq!(actual, expected),
        (actual, expected) => panic!("Omega {actual:?}, Rust {expected:?}"),
    }
}
fn record(bytes: &[u8]) {
    compare(policy::decode(bytes), Declaration::decode(bytes));
}
fn records(entries: &[Import]) -> [Vec<u8>; 4] {
    [
        imports::encode(entries).to_vec(),
        imports::encode_brokered(entries).to_vec(),
        imports::encode_funded(entries, u16::MAX, u16::MAX).to_vec(),
        imports::encode_profile(entries, 0, 0, [0, u64::MAX, 1]).to_vec(),
    ]
}
#[test]
fn all_record_forms_preserve_entries_and_untrusted_resource_demands() {
    let entries = [
        Import::v1(1, true),
        Import::v1(2, false),
        Import {
            contract: 4,
            minimum: 2,
            maximum: u64::MAX,
            required: true,
        },
        Import::v1(u64::MAX, false),
    ];
    for count in 0..=4 {
        for bytes in records(&entries[..count]) {
            record(&bytes);
            // Every bit includes reserved header bits, boolean encoding, identity,
            // intervals, unused padding, and each resource/profile field.
            for at in 0..bytes.len() {
                for bit in 0..8 {
                    let mut altered = bytes.clone();
                    altered[at] ^= 1 << bit;
                    record(&altered);
                }
            }
        }
    }
}
#[test]
fn exact_length_magic_and_empty_padding_are_required() {
    for entries in [&[][..], &imports::REQUIRED[..]] {
        for bytes in records(entries) {
            for length in 0..=imports::PROFILE_BYTES + 1 {
                let mut altered = bytes.clone();
                altered.resize(length, 0);
                record(&altered);
                if length != bytes.len() {
                    assert_eq!(policy::decode(&altered), Err(Refusal::Malformed));
                }
            }
            let mut oversize = bytes.clone();
            oversize.resize(256, 0);
            assert_eq!(policy::decode(&oversize), Err(Refusal::Malformed));
        }
    }
    for magic in [
        *b"CTHIMP00",
        *b"CTHIMP04",
        *b"CTHIMP06",
        *b"CTHIMP99",
        [0; 8],
    ] {
        let mut bytes = imports::encode(&imports::REQUIRED);
        bytes[..8].copy_from_slice(&magic);
        assert_eq!(policy::decode(&bytes), Err(Refusal::Malformed));
    }
}
#[test]
fn duplicate_and_malformed_later_entries_refuse_before_matching() {
    let entries = [Import::v1(u64::MAX, true), Import::v1(1, false)];
    for (offset, value) in [(48, u64::MAX), (48, 0), (56, 0), (56, 2), (72, 2)] {
        let mut bytes = imports::encode_brokered(&entries);
        bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
        record(&bytes);
        let result = policy::from_elf(&image(&bytes))
            .and_then(|d| policy::resolve(d.as_ref(), imports::SUPPORTED));
        assert_eq!(result, Err(Refusal::Malformed));
    }
    let bytes = imports::encode_brokered(&entries);
    let result = policy::from_elf(&image(&bytes))
        .and_then(|d| policy::resolve(d.as_ref(), imports::SUPPORTED));
    assert_eq!(result, Err(Refusal::Missing(u64::MAX)));
}
fn image(record: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0; 256 + record.len()];
    bytes[..6].copy_from_slice(b"\x7fELF\x02\x01");
    bytes[32..40].copy_from_slice(&64u64.to_le_bytes());
    bytes[54..56].copy_from_slice(&56u16.to_le_bytes());
    bytes[56..58].copy_from_slice(&1u16.to_le_bytes());
    bytes[64..72].copy_from_slice(&[1, 0, 0, 0, 4, 0, 0, 0]);
    bytes[72..80].copy_from_slice(&256u64.to_le_bytes());
    bytes[96..104].copy_from_slice(&(record.len() as u64).to_le_bytes());
    bytes[256..].copy_from_slice(record);
    bytes
}
fn executable(bytes: &[u8]) {
    match (policy::from_elf(bytes), imports::from_elf(bytes)) {
        (Ok(Some(actual)), Ok(Some(expected))) => compare(Ok(actual), Ok(expected)),
        (Ok(None), Ok(None)) => (),
        (Err(actual), Err(expected)) => assert_eq!(actual, expected),
        (actual, expected) => panic!("Omega {actual:?}, Rust {expected:?}"),
    }
}
#[test]
fn elf_boundary_preserves_legacy_absence_and_rejects_ambiguous_or_truncated_records() {
    for record in records(&imports::REQUIRED) {
        let bytes = image(&record);
        executable(&bytes);
        for length in 0..bytes.len() {
            executable(&bytes[..length]);
        }
        for at in [32, 54, 56, 64, 68, 72, 96] {
            let mut altered = bytes.clone();
            altered[at..at + 8].fill(255);
            executable(&altered);
        }
        let mut duplicate = bytes.clone();
        duplicate[56..58].copy_from_slice(&2u16.to_le_bytes());
        duplicate.copy_within(64..120, 120);
        assert_eq!(policy::from_elf(&duplicate), Err(Refusal::Malformed));
        let mut absent = bytes.clone();
        absent[256..264].fill(0);
        assert_eq!(policy::from_elf(&absent), Ok(None));
        absent[256..264].copy_from_slice(b"CTHIMP04");
        assert_eq!(policy::from_elf(&absent), Err(Refusal::Malformed));
    }
}
