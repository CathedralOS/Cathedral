// SPDX-License-Identifier: MIT OR Apache-2.0
// Generated exact pure fragments from pinned instructions/tlb.rs. No instruction execution.
use bit_field::BitField;
use core::{cmp,fmt};
use std::iter::Step;
use x86_64::{VirtAddr,structures::paging::{Page,PageSize,Size2MiB,page::NotGiantPageSize}};
#[derive(Debug)]
pub enum InvPcidCommand {
    /// The logical processor invalidates mappings—except global translations—for the linear address and PCID specified.
    Address(VirtAddr, Pcid),

    /// The logical processor invalidates all mappings—except global translations—associated with the PCID.
    Single(Pcid),

    /// The logical processor invalidates all mappings—including global translations—associated with any PCID.
    All,

    /// The logical processor invalidates all mappings—except global translations—associated with any PCID.
    AllExceptGlobal,
}

/// The INVPCID descriptor comprises 128 bits and consists of a PCID and a linear address.
/// For INVPCID type 0, the processor uses the full 64 bits of the linear address even outside 64-bit mode; the linear address is not used for other INVPCID types.
#[repr(C)]
#[derive(Debug)]
pub struct InvpcidDescriptor {
    pcid: u64,
    address: u64,
}

/// Structure of a PCID. A PCID has to be <= 4096 for x86_64.
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


pub fn prepare_invpcid(command: InvPcidCommand)->(u64,InvpcidDescriptor) {
    let mut desc = InvpcidDescriptor {
        pcid: 0,
        address: 0,
    };

    let kind: u64;
    match command {
        InvPcidCommand::Address(addr, pcid) => {
            kind = 0;
            desc.pcid = pcid.value().into();
            desc.address = addr.as_u64()
        }
        InvPcidCommand::Single(pcid) => {
            kind = 1;
            desc.pcid = pcid.0.into()
        }
        InvPcidCommand::All => kind = 2,
        InvPcidCommand::AllExceptGlobal => kind = 3,
    }

    (kind,desc)
}
pub fn encode_broadcast<S:NotGiantPageSize>(va_and_count:Option<(Page<S>,u16)>,pcid:Option<Pcid>,asid:Option<u16>,include_global:bool,final_translation_only:bool,include_nested_translations:bool)->(u64,u32,u32) {
    let mut rax = 0;
    let mut ecx = 0;
    let mut edx = 0;

    if let Some((va, count)) = va_and_count {
        rax.set_bit(0, true);
        rax.set_bits(12.., va.start_address().as_u64().get_bits(12..));

        ecx.set_bits(0..=15, u32::from(count));
        ecx.set_bit(31, S::SIZE == Size2MiB::SIZE);
    }

    if let Some(pcid) = pcid {
        rax.set_bit(1, true);
        edx.set_bits(16..=27, u32::from(pcid.value()));
    }

    if let Some(asid) = asid {
        rax.set_bit(2, true);
        edx.set_bits(0..=15, u32::from(asid));
    }

    rax.set_bit(3, include_global);
    rax.set_bit(4, final_translation_only);
    rax.set_bit(5, include_nested_translations);

    (rax,ecx,edx)
}
pub fn pinned_chunk<S:NotGiantPageSize>(start:Page<S>,end:Page<S>,count_max:u16)->Option<(u16,u16,u64)> {
    let pages=Page::range(start,end);
    if pages.is_empty(){return None;}
                // Calculate out how many pages we still need to flush.
                let count = <Page<S> as Step>::steps_between(&pages.start, &pages.end).0;

                // Make sure that we never jump the gap in the address space when flushing.
                let second_half_start =
                    Page::<S>::containing_address(VirtAddr::new(0xffff_8000_0000_0000));
                let count = if pages.start < second_half_start {
                    let count_to_second_half =
                        <Page<S> as Step>::steps_between(&pages.start, &second_half_start).0;
                    cmp::min(count, count_to_second_half)
                } else {
                    count
                };

                // We can flush at most u16::MAX pages at once.
                let count = u16::try_from(count).unwrap_or(u16::MAX);

                // Cap the count by the maximum supported count of the processor.
                let count = cmp::min(count, count_max);

    let inc_count=cmp::max(count,1);
    let next=<Page<S> as Step>::forward_checked(pages.start,usize::from(inc_count)).unwrap();
    Some((count,inc_count,next.start_address().as_u64()))
}
pub fn descriptor_offsets()->(usize,usize){(core::mem::offset_of!(InvpcidDescriptor,pcid),core::mem::offset_of!(InvpcidDescriptor,address))}
