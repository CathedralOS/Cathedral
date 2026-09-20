// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2017 Philipp Oppermann. See the README.md (upstream idt.rs).
// Exact option method bodies on detached mirror fields: no installed gate or CPU.
use x86_64::{VirtAddr,PrivilegeLevel,registers::{rflags::RFlags,segmentation::SegmentSelector},structures::idt::*};
use bit_field::BitField;
#[derive(Clone,Copy)]
struct OptionsMirror { cs:SegmentSelector,bits:u16 }
impl OptionsMirror {
    /// Creates a minimal options field with all the must-be-one bits set. This
    /// means the CS selector, IST, and DPL field are all 0.
    #[inline]
    const fn minimal() -> Self {
        OptionsMirror {
            cs: SegmentSelector(0),
            bits: 0b1110_0000_0000, // Default to a 64-bit Interrupt Gate
        }
    }

    /// Set the code segment that will be used by this interrupt.
    ///
    /// ## Safety
    /// This function is unsafe because the caller must ensure that the passed
    /// segment selector points to a valid, long-mode code segment.
    pub unsafe fn set_code_selector(&mut self, cs: SegmentSelector) -> &mut Self {
        self.cs = cs;
        self
    }

    /// Set or reset the preset bit.
    #[inline]
    pub fn set_present(&mut self, present: bool) -> &mut Self {
        self.bits.set_bit(15, present);
        self
    }

    fn present(&self) -> bool {
        self.bits.get_bit(15)
    }

    /// Let the CPU disable hardware interrupts when the handler is invoked. By default,
    /// interrupts are disabled on handler invocation.
    #[inline]
    pub fn disable_interrupts(&mut self, disable: bool) -> &mut Self {
        self.bits.set_bit(8, !disable);
        self
    }

    /// Set the required privilege level (DPL) for invoking the handler. The DPL can be 0, 1, 2,
    /// or 3, the default is 0. If CPL < DPL, a general protection fault occurs.
    #[inline]
    pub fn set_privilege_level(&mut self, dpl: PrivilegeLevel) -> &mut Self {
        self.bits.set_bits(13..15, dpl as u16);
        self
    }

    fn privilege_level(&self) -> PrivilegeLevel {
        PrivilegeLevel::from_u16(self.bits.get_bits(13..15))
    }

    /// Assigns a Interrupt Stack Table (IST) stack to this handler. The CPU will then always
    /// switch to the specified stack before the handler is invoked. This allows kernels to
    /// recover from corrupt stack pointers (e.g., on kernel stack overflow).
    ///
    /// An IST stack is specified by an IST index between 0 and 6 (inclusive). Using the same
    /// stack for multiple interrupts can be dangerous when nested interrupts are possible.
    ///
    /// This function panics if the index is not in the range 0..7.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the caller must ensure that the passed stack index is
    /// valid and not used by other interrupts. Otherwise, memory safety violations are possible.
    #[inline]
    pub unsafe fn set_stack_index(&mut self, index: u16) -> &mut Self {
        // The hardware IST index starts at 1, but our software IST index
        // starts at 0. Therefore we need to add 1 here.
        self.bits.set_bits(0..3, index + 1);
        self
    }

    fn stack_index(&self) -> Option<u16> {
        self.bits.get_bits(0..3).checked_sub(1)
    }
}
fn option_bytes(value:EntryOptions)->[u8;4] {unsafe{core::mem::transmute(value)}}
#[test]
fn options_actual_safe_methods_and_exact_detached_bodies() {
 let mut actual:EntryOptions=unsafe{core::mem::transmute([0u8,0,0,14])};
 actual.set_present(true).disable_interrupts(false).set_privilege_level(PrivilegeLevel::Ring3);
 assert_eq!(option_bytes(actual),[0,0,0,239]);
 for index in 0..7u16 {for dpl in 0..4u16 {for present in [false,true] {for disable in [false,true] {
  let mut value=OptionsMirror::minimal();
  // These are copied numeric bodies on a test-owned mirror, never original
  // live-handler lifetime/stack-authority operations.
  unsafe {value.set_code_selector(SegmentSelector(0xabcd));value.set_stack_index(index);}
  value.set_privilege_level(PrivilegeLevel::from_u16(dpl)).set_present(present).disable_interrupts(disable);
  let expected=0x0e00|(index+1)|(dpl<<13)|if present{0x8000}else{0}|if disable{0}else{0x100};
  assert_eq!(value.cs.0,0xabcd);assert_eq!(value.bits,expected);
  assert_eq!(value.stack_index(),Some(index));assert_eq!(value.privilege_level() as u16,dpl);assert_eq!(value.present(),present);
 }}}}
 assert_eq!(OptionsMirror::minimal().stack_index(),None);
 assert!(std::panic::catch_unwind(||unsafe{OptionsMirror::minimal().set_stack_index(7);}).is_err());
}
#[test]
fn all_missing_entries_and_detached_handler_projection() {
 let table=InterruptDescriptorTable::new();
 let bytes=unsafe{core::slice::from_raw_parts((&table as *const InterruptDescriptorTable).cast::<u8>(),4096)};
 for vector in 0..256 {for byte in 0..16 {assert_eq!(bytes[vector*16+byte],if byte==5{14}else{0});}}
 let raw:[u8;16]=[0x66,0x55,0xab,0xcd,7,0x8e,0x44,0x33,0x22,0x11,0,0,0,0,0,0];
 let entry:Entry<HandlerFunc>=unsafe{core::mem::transmute(raw)};
 assert_eq!(entry.handler_addr().as_u64(),0x112233445566);
}
#[test]
fn every_vector_identity_index_and_ranges() {
 let idt=InterruptDescriptorTable::new();
 for vector in 0..=255u8 {
  let known=vector<=8||(10..=14).contains(&vector)||(16..=21).contains(&vector)||(28..=30).contains(&vector);
  assert_eq!(ExceptionVector::try_from(vector).is_ok(),known);
  let available=matches!(vector,0..=7|9|16|19|20|28|32..=255);
  assert_eq!(std::panic::catch_unwind(||&idt[vector]).is_ok(),available);
 }
 assert_eq!(idt.slice(32..).len(),224);assert_eq!(idt.slice(255..=255).len(),1);
 assert_eq!(idt.slice((core::ops::Bound::Excluded(255),core::ops::Bound::Unbounded)).len(),0);
 assert!(std::panic::catch_unwind(||idt.slice(..)).is_err());
 assert!(std::panic::catch_unwind(||idt.slice(64..32)).is_err());
}
#[test]
fn frame_new_preserves_fields_and_zeroes_reserved_bytes() {
 let frame=InterruptStackFrameValue::new(VirtAddr::new(0x1000),SegmentSelector(8),RFlags::INTERRUPT_FLAG,VirtAddr::new(0x2000),SegmentSelector(16));
 assert_eq!(frame.instruction_pointer.as_u64(),0x1000);assert_eq!(frame.stack_pointer.as_u64(),0x2000);
 assert_eq!(frame.code_segment.0,8);assert_eq!(frame.stack_segment.0,16);assert_eq!(frame.cpu_flags.bits(),512);
 let bytes:[u8;40]=unsafe{core::mem::transmute(frame)};
 assert_eq!(&bytes[10..16],&[0;6]);assert_eq!(&bytes[34..40],&[0;6]);
}
