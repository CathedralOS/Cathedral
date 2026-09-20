// SPDX-License-Identifier: MIT OR Apache-2.0
// Pinned source bodies: pointer and byte observation replaced with explicit inputs.
// No validation comparison or bit composition is changed.
use x86_64::structures::{gdt::{Descriptor,DescriptorFlags,GlobalDescriptorTable},tss::{TaskStateSegment,InvalidIoMap},idt::{SelectorErrorCode,DescriptorTable}};
use bit_field::BitField;
use core::{cmp,mem};
fn encode_raw(base:u64,iomap_base:u16,iomap_size:u16)->Descriptor {
        use self::DescriptorFlags as Flags;

        let ptr = base;

        let mut low = Flags::PRESENT.bits();
        // base
        low.set_bits(16..40, ptr.get_bits(0..24));
        low.set_bits(56..64, ptr.get_bits(24..32));
        // limit (the `-1` is needed since the bound is inclusive)
        let iomap_limit = u64::from(iomap_base) + u64::from(iomap_size);
        low.set_bits(
            0..16,
            cmp::max(mem::size_of::<TaskStateSegment>() as u64, iomap_limit) - 1,
        );
        // type (0b1001 = available 64-bit tss)
        low.set_bits(40..44, 0b1001);

        let mut high = 0;
        high.set_bits(0..32, ptr.get_bits(32..64));

        Descriptor::SystemSegment(low, high)
    }
fn validate(tss_addr:usize,iomap_addr:usize,length:usize,last_byte:u8,iomap_base:u16)->Result<Descriptor,InvalidIoMap> {
        if length > 8193 {
            return Err(InvalidIoMap::TooLong { len: length });
        }




        if tss_addr > iomap_addr {
            return Err(InvalidIoMap::IoMapBeforeTss);
        }

        let base = iomap_addr - tss_addr;
        if base > 0xdfff {
            return Err(InvalidIoMap::TooFarFromTss { distance: base });
        }

        let last_byte = if length == 0 { 0xff } else { last_byte };
        if last_byte != 0xff {
            return Err(InvalidIoMap::InvalidTerminatingByte { byte: last_byte });
        }

        if iomap_base != base as u16 {
            return Err(InvalidIoMap::InvalidBase {
                expected: base as u16,
                got: iomap_base,
            });
        }

        // SAFETY: all invariants checked above
        Ok(encode_raw(tss_addr as u64, iomap_base, length as u16))
    }
fn words(value:Descriptor)->(u64,u64) {match value {Descriptor::SystemSegment(a,b)=>(a,b),_=>panic!("system expected")}}
#[test]
fn pointer_tss_defaults_and_descriptors() {
 let tss=TaskStateSegment::new();assert_eq!(tss.iomap_base,104);
 let rsp=tss.privilege_stack_table;let ist=tss.interrupt_stack_table;
 assert!(rsp.iter().chain(ist.iter()).all(|p|p.as_u64()==0));
 for base in [0,1,0x1122334455667788,u64::MAX] {for (iomap,size) in [(104,0),(104,1),(0xdfff,8193),(0,0)] {
  let (low,high)=words(encode_raw(base,iomap,size));
  let limit=cmp::max(104u64,iomap as u64+size as u64)-1;
  assert_eq!(low,(limit&65535)|((base&0xffffff)<<16)|(((base>>24)&255)<<56)|0x890000000000);
  assert_eq!(high,base>>32);
 }}
}
#[test]
fn iomap_original_check_order_and_boundaries() {
 assert!(matches!(validate(2,1,8194,0,0),Err(InvalidIoMap::TooLong{len:8194})));
 assert!(matches!(validate(2,1,1,0,0),Err(InvalidIoMap::IoMapBeforeTss)));
 assert!(matches!(validate(0,0xe000,1,0,0),Err(InvalidIoMap::TooFarFromTss{distance:0xe000})));
 assert!(matches!(validate(0,104,1,0,0),Err(InvalidIoMap::InvalidTerminatingByte{byte:0})));
 assert!(matches!(validate(0,104,1,255,0),Err(InvalidIoMap::InvalidBase{expected:104,got:0})));
 assert_eq!(words(validate(0,0xdfff,8193,255,0xdfff).unwrap()).0&65535,65535);
 assert_eq!(words(validate(0,0,0,0,0).unwrap()).0&65535,103);
 assert_eq!(words(validate(0,104,0,0,104).unwrap()).0&65535,103);
 // The original checks permit a numeric base inside the fixed TSS. No valid
 // borrowed overlapping Rust object is fabricated; this is validation-only.
 assert_eq!(words(validate(0,1,1,255,1).unwrap()).0&65535,103);
}
#[test]
fn gdt_append_and_full_boundaries() {
 let mut table=GlobalDescriptorTable::<8>::new();assert_eq!(table.limit(),7);
 for _ in 0..5 {table.append(Descriptor::kernel_code_segment());}
 let selector=table.append(Descriptor::SystemSegment(0x890000000067,0));
 assert_eq!(selector.0,48);assert_eq!(table.entries().len(),8);assert_eq!(table.limit(),63);
 assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||table.append(Descriptor::user_data_segment()))).is_err());
 let mut table=GlobalDescriptorTable::<2>::empty();
 assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||table.append(Descriptor::SystemSegment(0,0)))).is_err());
 let table=GlobalDescriptorTable::<3>::from_raw_entries(&[0,DescriptorFlags::KERNEL_CODE64.bits(),DescriptorFlags::KERNEL_DATA.bits()]);
 assert_eq!(table.entries().len(),3);assert_eq!(table.limit(),23);
 assert_eq!(Descriptor::user_code_segment().dpl() as u8,3);
}
#[test]
fn selector_errors_exhaustive() {
 for raw in 0..=65535u64 {
  let code=SelectorErrorCode::new(raw).unwrap();assert_eq!(code.external(),raw&1!=0);assert_eq!(code.index(),raw>>3);assert_eq!(code.is_null(),raw==0);
  assert_eq!(code.descriptor_table(),match(raw>>1)&3 {0=>DescriptorTable::Gdt,2=>DescriptorTable::Ldt,_=>DescriptorTable::Idt});
 }
 assert!(SelectorErrorCode::new(65536).is_none());assert_eq!(SelectorErrorCode::new_truncate(u64::MAX).index(),8191);
}
