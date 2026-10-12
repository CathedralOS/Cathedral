//! Standalone host test root for firmware parsing, without compiling the UEFI executable.
//! Uses the real architecture permission types; see arch/memory/README.md for the command.
extern crate self as cathedral_arch;
#[path = "../../../arch/memory/mod.rs"]
pub mod memory;
pub use memory::ImageRange;
mod image;

const BASE: u64 = 0x100000;
const OPTIONAL: usize = 0x98;
const SECTIONS: usize = OPTIONAL + 0xf0;
fn put16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}
fn put32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
fn fixture() -> Vec<u8> {
    let mut bytes = vec![0; 0x5000];
    put16(&mut bytes, 0, 0x5a4d);
    put32(&mut bytes, 0x3c, 0x80);
    put32(&mut bytes, 0x80, 0x4550);
    put16(&mut bytes, 0x86, 3);
    put16(&mut bytes, 0x94, 0xf0);
    put16(&mut bytes, OPTIONAL, 0x20b);
    put32(&mut bytes, OPTIONAL + 32, 4096);
    put32(&mut bytes, OPTIONAL + 56, 0x5000);
    put32(&mut bytes, OPTIONAL + 60, 0x400);
    for (index, flags) in [0x6000_0020, 0x4000_0040, 0xc000_0080]
        .into_iter()
        .enumerate()
    {
        put32(&mut bytes, SECTIONS + index * 40 + 8, 37);
        put32(
            &mut bytes,
            SECTIONS + index * 40 + 12,
            (index as u32 + 1) * 4096,
        );
        put32(&mut bytes, SECTIONS + index * 40 + 36, flags);
    }
    bytes
}
#[test]
fn relocated_sections_include_bss_and_round_up_without_granting_padding() {
    let bytes = fixture();
    let loaded = image::parse(&bytes, BASE).unwrap();
    let protection = loaded.protection();
    for (offset, writable, executable) in [
        (0, false, false),
        (0x1fff, false, true),
        (0x2000, false, false),
        (0x3000, true, false),
        (0x4000, false, false),
    ] {
        let p = protection.permissions(BASE + offset).unwrap();
        assert_eq!(p.writable, writable);
        assert_eq!(p.executable, executable);
    }
}
#[test]
fn truncated_and_overflowing_headers_fail_without_panicking() {
    let mut bytes = fixture();
    for size in 0..bytes.len() {
        assert!(image::parse(&bytes[..size], BASE).is_err());
    }
    put32(&mut bytes, 0x3c, u32::MAX);
    assert!(image::parse(&bytes, BASE).is_err());
    assert!(image::parse(&fixture(), u64::MAX - 4095).is_err());
}
#[test]
fn permissions_and_geometry_fail_closed() {
    for (offset, value) in [
        (OPTIONAL + 32, 512),         // Subpage section alignment.
        (OPTIONAL + 60, 0x100),       // Section table outside headers.
        (SECTIONS + 36, 0xe000_0020), // Writable executable section.
        (SECTIONS + 8, 0x2001),       // Rounded text overlaps the next section.
        (SECTIONS + 12, 0),           // Section overlaps headers.
        (SECTIONS + 12, 0x5000),      // Section beyond loaded image.
        (SECTIONS + 36, 0x4000_0040), // No executable code.
    ] {
        let mut bytes = fixture();
        put32(&mut bytes, offset, value);
        assert!(
            image::parse(&bytes, BASE).is_err(),
            "accepted field {offset:#x}={value:#x}"
        );
    }
    let mut bytes = fixture();
    put16(&mut bytes, 0x86, 17);
    assert!(image::parse(&bytes, BASE).is_err());
}
