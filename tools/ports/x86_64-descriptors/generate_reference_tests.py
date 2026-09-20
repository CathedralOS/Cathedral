#!/usr/bin/env python3
"""Extract pinned pure bit/validation work with explicit non-pointer inputs."""
from pathlib import Path
import sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
s=(ROOT/'reference_code/rust-osdev/x86_64/src/structures/gdt.rs').read_text()
def body(marker):
 start=s.index(marker);first=s.index('{',start);i=first+1;depth=1
 while depth:depth+=(s[i]=='{')-(s[i]=='}');i+=1
 return s[first:i]
raw=body('unsafe fn tss_segment_raw(')
replacements={'let ptr = tss as u64;':'let ptr = base;','u64::from(unsafe { (*tss).iomap_base })':'u64::from(iomap_base)'}
for a,b in replacements.items():assert raw.count(a)==1;raw=raw.replace(a,b)
validation=body('pub fn tss_segment_with_iomap(')
for a,b in [('iomap.len()','length'),('let iomap_addr = iomap.as_ptr() as usize;',''),('let tss_addr = tss as *const _ as usize;',''),('let last_byte = *iomap.last().unwrap_or(&0xff);','let last_byte = if length == 0 { 0xff } else { last_byte };'),('tss.iomap_base','iomap_base'),('Ok(unsafe { Self::tss_segment_raw(tss, length as u16) })','Ok(encode_raw(tss_addr as u64, iomap_base, length as u16))')]:
 assert a in validation,a;validation=validation.replace(a,b)
tests='''// SPDX-License-Identifier: MIT OR Apache-2.0
// Pinned source bodies: pointer and byte observation replaced with explicit inputs.
// No validation comparison or bit composition is changed.
use x86_64::structures::{gdt::{Descriptor,DescriptorFlags,GlobalDescriptorTable},tss::{TaskStateSegment,InvalidIoMap},idt::{SelectorErrorCode,DescriptorTable}};
use bit_field::BitField;
use core::{cmp,mem};
fn encode_raw(base:u64,iomap_base:u16,iomap_size:u16)->Descriptor '''+raw+'''
fn validate(tss_addr:usize,iomap_addr:usize,length:usize,last_byte:u8,iomap_base:u16)->Result<Descriptor,InvalidIoMap> '''+validation+'''
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
'''
tests='\n'.join(line.rstrip() for line in tests.splitlines())+'\n'
p=HERE/'src/tests.rs'
if '--check' in sys.argv:
 if not p.exists() or p.read_text()!=tests:raise SystemExit('reference extraction differs')
else:p.write_text(tests)
print('Pinned encoder and bitmap comparisons extracted; pointer/byte observations explicit')
