use super::*;
use alloc::vec;
use alloc::vec::Vec;
use cathedral_arch::{USER_CODE, USER_IMAGE_END, USER_STACK};

fn put16(b: &mut [u8], at: usize, value: u16) {
    b[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn put32(b: &mut [u8], at: usize, value: u32) {
    b[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
fn put64(b: &mut [u8], at: usize, value: u64) {
    b[at..at + 8].copy_from_slice(&value.to_le_bytes());
}
fn fixture() -> Vec<u8> {
    let mut bytes = vec![0; 8193];
    bytes[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
    put16(&mut bytes, 16, 2);
    put16(&mut bytes, 18, 62);
    put32(&mut bytes, 20, 1);
    put64(&mut bytes, 24, USER_CODE);
    put64(&mut bytes, 32, 64);
    put16(&mut bytes, 52, 64);
    put16(&mut bytes, 54, 56);
    put16(&mut bytes, 56, 2);
    for (index, (flags, file_size, memory_size)) in
        [(5, 1, 1), (6, 1, 8193)].into_iter().enumerate()
    {
        let header = 64 + index * 56;
        put32(&mut bytes, header, 1);
        put32(&mut bytes, header + 4, flags);
        put64(&mut bytes, header + 8, 4096 * (index as u64 + 1));
        put64(&mut bytes, header + 16, USER_CODE + 4096 * index as u64);
        put64(&mut bytes, header + 32, file_size);
        put64(&mut bytes, header + 40, memory_size);
        put64(&mut bytes, header + 48, 4096);
    }
    bytes[4096] = 0xc3;
    bytes[8192] = 0x42;
    bytes
}

#[test]
fn accepts_static_image_and_retains_file_bytes_separately_from_bss() {
    let bytes = fixture();
    let image = parse(&bytes).unwrap();
    assert_eq!(image.entry, USER_CODE);
    assert_eq!(image.segments().len(), 2);
    assert_eq!(image.segments()[1].bytes, &[0x42]);
    assert_eq!(image.segments()[1].memory_size, 8193);
    assert!(image.segments()[0].executable && !image.segments()[0].writable);
    assert!(image.segments()[1].writable && !image.segments()[1].executable);
}

#[test]
fn rejects_truncation_overflow_and_oversized_header_tables() {
    let bytes = fixture();
    for length in 0..bytes.len() {
        assert!(parse(&bytes[..length]).is_err());
    }
    for (at, value) in [
        (32, u64::MAX),
        (64 + 8, u64::MAX - 4095),
        (64 + 32, u64::MAX),
        (64 + 40, u64::MAX),
    ] {
        let mut b = fixture();
        put64(&mut b, at, value);
        assert!(parse(&b).is_err());
    }
    let mut b = fixture();
    put16(&mut b, 56, 17);
    assert!(parse(&b).is_err());
}

#[test]
fn rejects_wrong_format_dynamic_tls_and_executable_stack() {
    for (at, value) in [
        (0, 0),
        (4, 1),
        (5, 2),
        (6, 0),
        (7, 3),
        (8, 1),
        (16, 3),
        (18, 3),
        (20, 0),
        (48, 1),
        (52, 63),
        (54, 55),
    ] {
        let mut b = fixture();
        b[at] = value;
        assert!(parse(&b).is_err());
    }
    for kind in [2, 3, 7, 0x6474_e551, 99] {
        let mut b = fixture();
        put32(&mut b, 64, kind);
        assert!(parse(&b).is_err());
    }
}

#[test]
fn accepts_only_a_non_executable_stack_and_rejects_dynamic_requirements() {
    let mut bytes = fixture();
    put16(&mut bytes, 56, 3);
    put32(&mut bytes, 176, 0x6474_e551);
    put32(&mut bytes, 180, 6);
    assert!(parse(&bytes).is_ok());
    put32(&mut bytes, 180, 7);
    assert!(matches!(parse(&bytes), Err(Error::Unsupported)));
    for kind in [2, 3, 7] {
        put32(&mut bytes, 176, kind);
        assert!(matches!(parse(&bytes), Err(Error::Unsupported)));
    }
}

#[test]
fn rejects_file_bounds_alignment_and_entry_in_zero_fill() {
    for (at, value) in [(64 + 8, 4097), (64 + 48, 1), (64 + 32, 2)] {
        let mut bytes = fixture();
        put64(&mut bytes, at, value);
        assert!(parse(&bytes).is_err());
    }
    let mut bytes = fixture();
    put64(&mut bytes, 64 + 40, 2);
    put64(&mut bytes, 24, USER_CODE + 1);
    assert!(matches!(parse(&bytes), Err(Error::Entry)));
    bytes.resize(MAX_FILE + 1, 0);
    assert!(matches!(parse(&bytes), Err(Error::Bounds)));
}

#[test]
fn rejects_aliases_wx_invalid_entry_and_excessive_memory() {
    for flags in [0, 1, 3, 7, 8] {
        let mut b = fixture();
        put32(&mut b, 68, flags);
        assert!(parse(&b).is_err());
    }
    for address in [
        USER_CODE,
        USER_CODE + 1,
        USER_IMAGE_END,
        USER_STACK,
        0xffff_9000_0000_1000,
        u64::MAX - 4095,
    ] {
        let mut b = fixture();
        put64(&mut b, 120 + 16, address);
        assert!(parse(&b).is_err());
    }
    for entry in [0, USER_CODE + 1, USER_CODE + 4096, USER_STACK, u64::MAX] {
        let mut b = fixture();
        put64(&mut b, 24, entry);
        assert!(matches!(parse(&b), Err(Error::Entry)));
    }
    for size in [0, 64 * 4096, u64::MAX] {
        let mut b = fixture();
        put64(&mut b, 120 + 40, size);
        assert!(parse(&b).is_err());
    }
    let mut b = fixture();
    put64(&mut b, 64 + 40, 4097);
    assert!(parse(&b).is_err(), "overlap after page rounding");
}
