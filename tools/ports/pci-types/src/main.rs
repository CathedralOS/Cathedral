// SPDX-License-Identifier: MIT OR Apache-2.0
// Test harness authored for Cathedral. No hardware, FFI, or address dereference.
use std::cell::RefCell;
use pci_types::{ConfigRegionAccess, PciAddress, PciHeader, EndpointHeader, CommandRegister};
use pci_types::device_type::DeviceType;
use pci_types::capability::{PciCapability, MultipleMessageSupport};

struct Snapshot {
    words: RefCell<[u32; 1024]>,
    writes: RefCell<Vec<(u16, u32)>>,
}
impl Snapshot {
    fn new() -> Self { Self { words: RefCell::new([0; 1024]), writes: RefCell::new(Vec::new()) } }
    fn set(&self, offset: usize, word: u32) { self.words.borrow_mut()[offset/4] = word; }
}
impl ConfigRegionAccess for Snapshot {
    unsafe fn read(&self, _address: PciAddress, offset: u16) -> u32 {
        assert_eq!(offset & 3, 0);
        let word = self.words.borrow()[usize::from(offset)/4];
        // Test device BAR response only; no all-ones access reaches hardware.
        if offset == 16 && word == u32::MAX { 0xffff_f008 } else { word }
    }
    unsafe fn write(&self, _address: PciAddress, offset: u16, value: u32) {
        assert_eq!(offset & 3, 0);
        self.words.borrow_mut()[usize::from(offset)/4] = value;
        self.writes.borrow_mut().push((offset, value));
    }
}
fn main() {
    let bdf = PciAddress::new(65535, 255, 31, 7);
    assert_eq!((bdf.segment(), bdf.bus(), bdf.device(), bdf.function()), (65535, 255, 31, 7));
    let memory = Snapshot::new();
    memory.set(0, 0x12348086);
    memory.set(4, 0x80100007);
    memory.set(8, 0x0c033009);
    memory.set(12, 0x00800000);
    memory.set(52, 64);
    memory.set(64, 0x01860005);
    let mut header = PciHeader::new(bdf);
    assert_eq!(header.id(&memory), (0x8086, 0x1234));
    assert_eq!(header.revision_and_class(&memory), (9, 12, 3, 48));
    assert!(header.has_multiple_functions(&memory));
    assert!(header.status(&memory).parity_error_detected());
    assert!(header.status(&memory).has_capability_list());
    header.update_command(&memory, |_| CommandRegister::from_bits_retain(5));
    assert_eq!(*memory.writes.borrow().last().unwrap(), (4, 0x80100005));
    println!("witness command: upstream echoes status 0x8010 in DWORD write; Omega plan writes 16-bit command only");
    let endpoint = EndpointHeader::from_header(header, &memory).unwrap();
    let capability = endpoint.capabilities(&memory).next().unwrap();
    let msi = match capability { PciCapability::Msi(value) => value, _ => panic!("expected MSI") };
    assert!(msi.is_64bit());
    assert!(msi.has_per_vector_masking());
    msi.set_multiple_message_enable(MultipleMessageSupport::Int4, &memory);
    assert_eq!(memory.words.borrow()[16], 0x01860025);
    assert_eq!(memory.words.borrow()[16] & 255, 0x25);
    println!("witness MME: upstream 0x01860005 -> 0x01860025 corrupts ID; corrected control 0x01a6 belongs at +2");
    memory.set(16, 0x80000008);
    let bar = endpoint.bar(0, &memory).unwrap();
    match bar { pci_types::Bar::Memory32 { address, size, prefetchable } => {
        assert_eq!((address, size, prefetchable), (0x80000000, 4096, true));
    }, _ => panic!("expected memory32") }
    assert_eq!(memory.words.borrow()[4], 0x80000000);
    println!("witness BAR: upstream restores 0x80000000 instead of original 0x80000008; Omega restores exact words");
    assert_eq!(DeviceType::from((2, 5)), DeviceType::Unknown);
    assert_eq!(DeviceType::from((4, 3)), DeviceType::OtherMultimediaDevice);
    assert_eq!(DeviceType::from((4, 128)), DeviceType::Unknown);
    assert_eq!(DeviceType::from((5, 2)), DeviceType::OtherMemoryController);
    assert_eq!(DeviceType::from((5, 128)), DeviceType::Unknown);
    assert_eq!(DeviceType::from((15, 0)), DeviceType::TvSatelliteCommunicationsController);
    assert_eq!(DeviceType::from((15, 4)), DeviceType::Unknown);
    println!("witness classes: omitted WorldFIP 0205, OtherMultimedia incorrectly0403, OtherMemory incorrectly0502, satellite subclasses shifted down by one");
    // A deliberately cyclic non-null list is sampled a finite number of times.
    // Do not invoke a null-only cycle: the upstream call itself would not end.
    memory.set(64, 0x00004009);
    let mut list = endpoint.capabilities(&memory);
    for _ in 0..4 { assert!(matches!(list.next(), Some(PciCapability::Vendor(_)))); }
    println!("witness list: pinned iterator returns four entries from same self-cycle; Omega rejects second visit");
    memory.set(64, 0x01060005); // 32-bit with per-vector masking.
    memory.set(76, 0x80000000);
    memory.set(80, 1);
    let msi = match endpoint.capabilities(&memory).next().unwrap() { PciCapability::Msi(value) => value, _ => panic!("expected MSI") };
    assert_eq!(msi.message_mask(&memory), 0);
    assert_eq!(msi.is_pending(&memory), 0);
    println!("witness masking: upstream ignores 32-bit mask/pending at +12/+16; Omega exposes both with PVM flag");
    println!("All pinned Rust witnesses passed using in-memory config provider.");
}
