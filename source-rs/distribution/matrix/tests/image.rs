//! Image discovery compares independent parsers; the host knows only requested spans.
use super::declaration::{executable, image};
use crate::policy::imports as policy;
use cathedral_platform_contracts::imports::{self, Refusal};

#[test]
fn header_and_prefix_mutations_match_the_rust_image_parser() {
    for record in [
        imports::encode(&imports::REQUIRED).to_vec(),
        imports::encode_brokered(&imports::REQUIRED).to_vec(),
        imports::encode_funded(&imports::REQUIRED, 4, 2).to_vec(),
        imports::encode_profile(&imports::REQUIRED, 4, 2, [80, 64, 8]).to_vec(),
    ] {
        let bytes = image(&record);
        for at in (0..120).chain(256..264) {
            for bit in 0..8 {
                let mut altered = bytes.clone();
                altered[at] ^= 1 << bit;
                executable(&altered);
            }
        }
    }
}
#[test]
fn table_extents_counts_and_entry_sizes_never_wrap_or_skip_validation() {
    let mut bytes = image(&imports::encode(&imports::REQUIRED));
    bytes.resize(2048, 0);
    for count in [0u16, 1, 2, 15, 16, 17, u16::MAX] {
        for size in [0u16, 55, 56, 57, u16::MAX] {
            for offset in [0, 1, 63, 64, 65, 1992, 2048, 2049, u64::MAX - 55, u64::MAX] {
                bytes[32..40].copy_from_slice(&offset.to_le_bytes());
                bytes[54..56].copy_from_slice(&size.to_le_bytes());
                bytes[56..58].copy_from_slice(&count.to_le_bytes());
                executable(&bytes);
            }
        }
    }
}
#[test]
fn only_read_only_loads_are_inspected_but_all_their_bounds_are_checked() {
    let initial = image(&imports::encode_profile(
        &imports::REQUIRED,
        4,
        2,
        [80, 64, 8],
    ));
    for kind in [0u32, 1, 2, u32::MAX] {
        for flags in [0u32, 1, 2, 3, 4, 5, 6, 7, u32::MAX] {
            for offset in [
                0,
                256,
                initial.len() as u64,
                initial.len() as u64 + 1,
                u64::MAX,
            ] {
                for length in [0u64, 5, 6, 7, 8, 143, 144, 167, 168, u64::MAX] {
                    let mut bytes = initial.clone();
                    bytes[64..68].copy_from_slice(&kind.to_le_bytes());
                    bytes[68..72].copy_from_slice(&flags.to_le_bytes());
                    bytes[72..80].copy_from_slice(&offset.to_le_bytes());
                    bytes[96..104].copy_from_slice(&length.to_le_bytes());
                    executable(&bytes);
                }
            }
        }
    }
}
#[test]
fn a_found_record_cannot_hide_later_invalid_segments_or_duplicate_declarations() {
    let mut bytes = image(&imports::encode(&imports::REQUIRED));
    bytes[56..58].copy_from_slice(&2u16.to_le_bytes());
    bytes.copy_within(64..120, 120);
    assert_eq!(policy::from_elf(&bytes), Err(Refusal::Malformed));
    // The second read-only segment's range must be checked even after a match.
    bytes[128..136].copy_from_slice(&u64::MAX.to_le_bytes());
    assert_eq!(policy::from_elf(&bytes), Err(Refusal::Malformed));
    // An irrelevant segment need not contain accessible file data.
    bytes[124..128].copy_from_slice(&5u32.to_le_bytes());
    executable(&bytes);
    assert!(policy::from_elf(&bytes).unwrap().is_some());
    // Put a valid declaration at the final allowed header, preceded by empty loads.
    bytes.resize(1536, 0);
    bytes[64..1024].fill(0);
    bytes[56..58].copy_from_slice(&16u16.to_le_bytes());
    for i in 0..16 {
        let at = 64 + 56 * i;
        bytes[at..at + 8].copy_from_slice(&[1, 0, 0, 0, 4, 0, 0, 0]);
        bytes[at + 8..at + 16].copy_from_slice(&1024u64.to_le_bytes());
    }
    bytes[936..944].copy_from_slice(&(imports::RECORD_BYTES as u64).to_le_bytes());
    bytes[1024..1024 + imports::RECORD_BYTES].copy_from_slice(&imports::encode(&imports::REQUIRED));
    executable(&bytes);
    assert!(policy::from_elf(&bytes).unwrap().is_some());
}
fn reply(p: &mut [u64; 128], bytes: &[u8]) -> u64 {
    p[1] = 13;
    p[2] = bytes.len() as u64;
    p[16..24].fill(0);
    for (i, byte) in bytes.iter().enumerate() {
        p[16 + i / 8] |= u64::from(*byte) << ((i % 8) * 8);
    }
    super::decide(p)
}
#[test]
fn scan_protocol_rejects_short_replies_and_handles_large_offsets_without_wrap() {
    let mut p = [0; 128];
    p[..3].copy_from_slice(&[11, 12, u64::MAX]);
    assert_eq!(super::decide(&mut p), 1);
    assert_eq!(&p[4..6], &[0, 64]);
    assert_eq!(reply(&mut p, &[0; 63]), 4);
    assert_eq!(&p[4..6], &[0, 0]);
    assert_eq!(reply(&mut p, &[0; 64]), 4); // A terminal scan cannot resume.
    p[1] = 12;
    p[2] = u64::MAX;
    assert_eq!(super::decide(&mut p), 1);
    let mut header = image(&[]);
    header[32..40].copy_from_slice(&(u64::MAX - 56).to_le_bytes());
    assert_eq!(reply(&mut p, &header[..64]), 1);
    assert_eq!(&p[4..6], &[u64::MAX - 56, 56]);
    assert_eq!(reply(&mut p, &[0; 56]), 2);
    assert_eq!(&p[4..6], &[0, 0]);
    p[1] = 12;
    p[2] = u64::MAX;
    assert_eq!(super::decide(&mut p), 1);
    header[56..58].copy_from_slice(&2u16.to_le_bytes());
    assert_eq!(reply(&mut p, &header[..64]), 4); // Entire table must fit.
}
