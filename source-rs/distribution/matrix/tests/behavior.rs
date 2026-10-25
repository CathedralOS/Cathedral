//! Execute the compiled component against the existing storage-format oracle.
use crate::policy;
use cathedral_desktop_matrix_contracts::matrix::{
    self,
    config::{self, Config},
};
fn sample() -> Config {
    Config {
        launcher: cathedral_platform_contracts::storage::metadata::field(b"Desktop").unwrap(),
        launcher_pages: 48,
        app_pages: 80,
        app_readers: 64,
        margin: 8,
        launcher_height: 80,
        gap: 8,
        app_surfaces: 9,
        keys: matrix::VALID_KEY_MASK,
    }
}
#[test]
fn configuration_matches_existing_format_and_legacy_normalization() {
    for version in [b'1', b'2'] {
        let mut bytes = sample().encode();
        bytes[7] = version;
        for offset in (40..104).step_by(8) {
            for value in [
                0,
                1,
                8,
                64,
                128,
                129,
                1024,
                1025,
                1536,
                1537,
                4096,
                4097,
                u64::MAX,
            ] {
                let mut candidate = bytes;
                candidate[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
                assert_eq!(
                    policy::decode(&candidate),
                    Config::decode(&candidate),
                    "offset={offset} value={value} version={version}"
                );
            }
        }
        assert_eq!(policy::decode(&bytes), Config::decode(&bytes));
    }
}
#[test]
fn every_single_byte_mutation_matches_canonical_name_and_header_validation() {
    let original = sample().encode();
    for i in 0..104 {
        for byte in 0..=255 {
            let mut bytes = original;
            bytes[i] = byte;
            assert_eq!(
                policy::decode(&bytes),
                Config::decode(&bytes),
                "at={i} byte={byte}"
            );
        }
    }
    for length in 0..104 {
        assert!(policy::decode(&original[..length]).is_none());
    }
    assert!(policy::decode(&[0; 105]).is_none());
    for name in [
        "\u{e9}",
        "\u{65e5}\u{672c}\u{8a9e}",
        "\u{1f600}",
        "./Desktop",
    ] {
        let mut c = sample();
        c.launcher =
            cathedral_platform_contracts::storage::metadata::field(name.as_bytes()).unwrap();
        assert_eq!(policy::decode(&c.encode()), Some(c));
    }
}
#[test]
fn placement_matches_checked_oracle_including_overflow_boundaries() {
    let values = [
        0,
        1,
        8,
        80,
        768,
        1024,
        4096,
        u64::MAX / 2,
        u64::MAX / 2 + 1,
        u64::MAX,
    ];
    for m in values {
        for b in values {
            for g in values {
                for w in values {
                    for h in values {
                        assert_eq!(
                            policy::rectangles(m, b, g, w, h),
                            config::rectangles(m, b, g, w, h),
                            "{m} {b} {g} {w} {h}"
                        );
                    }
                }
            }
        }
    }
}
