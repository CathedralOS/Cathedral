// SPDX-License-Identifier: MIT OR Apache-2.0
// Generated exact pinned body fragments. Explicit old observations replace reads;
// returned operands replace instruction writes. No privileged API is called.
use core::fmt;
use x86_64::{PhysAddr,VirtAddr};
use x86_64::structures::paging::{Page,PhysFrame,Size4KiB};
use x86_64::registers::{control::{Cr0Flags,Cr3Flags,Cr4Flags,PriorityClass},model_specific::{EferFlags,CetFlags,ApicBaseFlags},rflags::RFlags,xcontrol::XCr0Flags,segmentation::SegmentSelector};
pub fn cr0(old_value:u64, flags:Cr0Flags)->u64 {
let reserved = old_value & !(Cr0Flags::all().bits());
            let new_value = reserved | flags.bits();
new_value
}
pub fn cr4(old_value:u64, flags:Cr4Flags)->u64 {
let reserved = old_value & !(Cr4Flags::all().bits());
            let new_value = reserved | flags.bits();
new_value
}
pub fn efer(old_value:u64, flags:EferFlags)->u64 {
let reserved = old_value & !(EferFlags::all().bits());
            let new_value = reserved | flags.bits();
new_value
}
pub fn rflags(old_value:u64, flags:RFlags)->u64 {
let reserved = old_value & !(RFlags::all().bits());
        let new_value = reserved | flags.bits();
new_value
}
pub fn xcr0_merge(old_value:u64,flags:XCr0Flags)->u64 {
            let reserved = old_value & !(XCr0Flags::all().bits());
            let new_value = reserved | flags.bits();
new_value
}
pub fn xcr0(old_value:u64,flags:XCr0Flags)->u64 {
            let reserved = old_value & !(XCr0Flags::all().bits());
            let new_value = reserved | flags.bits();

            assert!(flags.contains(XCr0Flags::X87), "The X87 flag must be set");
            if flags.contains(XCr0Flags::AVX) {
                assert!(
                    flags.contains(XCr0Flags::SSE),
                    "AVX cannot be enabled without enabling SSE"
                );
            }
            let mpx = XCr0Flags::BNDREG | XCr0Flags::BNDCSR;
            if flags.intersects(mpx) {
                assert!(
                    flags.contains(mpx),
                    "MPX flags XCr0.BNDREG and XCr0.BNDCSR must be set and unset together"
                );
            }
            let avx512 = XCr0Flags::OPMASK | XCr0Flags::ZMM_HI256 | XCr0Flags::HI16_ZMM;
            if flags.intersects(avx512) {
                assert!(
                    flags.contains(XCr0Flags::AVX),
                    "AVX-512 cannot be enabled without enabling AVX"
                );
                assert!(
                    flags.contains(avx512),
                    "AVX-512 flags XCR0.opmask, XCR0.ZMM_Hi256, and XCR0.Hi16_ZMM must be set and unset together"
                );
            }
new_value
}
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pcid(u16);

impl Pcid {
    /// Create a new PCID. Will result in a failure if the value of
    /// PCID is out of expected bounds.
    pub const fn new(pcid: u16) -> Result<Pcid, PcidTooBig> {
        if pcid >= 4096 {
            Err(PcidTooBig(pcid))
        } else {
            Ok(Pcid(pcid))
        }
    }

    /// Get the value of the current PCID.
    pub const fn value(&self) -> u16 {
        self.0
    }
}

/// A passed `u16` was not a valid PCID.
///
/// A PCID has to be <= 4096 for x86_64.
#[derive(Debug)]
pub struct PcidTooBig(u16);

impl fmt::Display for PcidTooBig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PCID should be < 4096, got {}", self.0)
    }
}

pub fn cr3_observed(value:u64)->(PhysFrame,u16) {
            let addr = PhysAddr::new(value & 0x_000f_ffff_ffff_f000);
            let frame = PhysFrame::containing_address(addr);
            (frame, (value & 0xFFF) as u16)
}
pub fn cr3_raw(top_bit:bool,frame:PhysFrame,val:u16)->u64 {
            let addr = frame.start_address();
            let value = ((top_bit as u64) << 63) | addr.as_u64() | val as u64;
value
}
pub fn cr3_flags(frame:PhysFrame,flags:Cr3Flags)->u64 { cr3_raw(false,frame,flags.bits() as u16) }
pub fn cr3_pcid(top_bit:bool,frame:PhysFrame,pcid:Pcid)->u64 { cr3_raw(top_bit,frame,pcid.value()) }
pub fn cr8_observed(raw:u64)->Option<PriorityClass> { PriorityClass::new(raw as u8) }
pub fn cr8_operand(priority_class:Option<PriorityClass>)->u64 {
            let value = priority_class.map_or(0, |pc| pc as u64);
value
}
pub fn star(raw:(u16,u16))->(SegmentSelector,SegmentSelector,SegmentSelector,SegmentSelector) {
            (
                SegmentSelector(raw.0 + 16),
                SegmentSelector(raw.0 + 8),
                SegmentSelector(raw.1),
                SegmentSelector(raw.1 + 8),
            )
}
pub fn cet_observed(value:u64)->(CetFlags,Page) {
            let cet_flags = CetFlags::from_bits_truncate(value);
            let legacy_bitmap =
                Page::from_start_address(VirtAddr::new(value & !(Page::<Size4KiB>::SIZE - 1)))
                    .unwrap();

            (cet_flags, legacy_bitmap)
}
pub fn cet_operand(flags:CetFlags,legacy_bitmap:Page)->u64 { flags.bits() | legacy_bitmap.start_address().as_u64() }
pub fn apic_observed(raw:u64)->(PhysFrame,u64) {
            let addr = PhysAddr::new_truncate(raw);
            let frame = PhysFrame::containing_address(addr);
            (frame, raw)
}
pub fn apic_preserving(old_flags:u64,frame:PhysFrame,flags:ApicBaseFlags)->u64 {
            let reserved = old_flags & !(ApicBaseFlags::all().bits());
            let new_flags = reserved | flags.bits();
apic_raw(frame,new_flags)
}
pub fn apic_raw(frame:PhysFrame,flags:u64)->u64 { let addr=frame.start_address(); flags | addr.as_u64() }
