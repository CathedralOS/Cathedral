// SPDX-License-Identifier: MIT OR Apache-2.0
// Original host witnesses. Only public numeric APIs and bound private mirrors run.
use x86_64::{PhysAddr,VirtAddr};
use x86_64::structures::paging::{Page,PageSize,PageTableIndex,PhysFrame,Size4KiB,Size2MiB,Size1GiB};
use x86_64::structures::paging::page_table::{PageTableEntry,PageTableFlags,FrameError};
include!("private_reference.rs");
const MASK:u64=0x000f_ffff_ffff_f000;
fn entry(word:u64)->PageTableEntry {let mut e=PageTableEntry::new();e.set_addr(PhysAddr::new(word&MASK),PageTableFlags::from_bits_retain(word&!MASK));e}
fn main(){
 std::panic::set_hook(Box::new(|_|{}));
 for r in 0..512u16 {
  let rr=PageTableIndex::new(r);
  let (a,b,c,d)=match r%4 {0=>(0,0,0,0),1=>(255,511,511,511),2=>(256,1,511,1),_=>(511,511,0,511)};
  let i=|x|PageTableIndex::new(x);
  let p4:Page<Size4KiB>=Page::from_page_table_indices(i(a),i(b),i(c),i(d));
  let p2:Page<Size2MiB>=Page::from_page_table_indices_2mib(i(a),i(b),i(c));
  let p1:Page<Size1GiB>=Page::from_page_table_indices_1gib(i(a),i(b));
  println!("C|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",r,p4.start_address().as_u64(),p2.start_address().as_u64(),p1.start_address().as_u64(),p3_page(p4,rr).start_address().as_u64(),p3_page(p2,rr).start_address().as_u64(),p3_page(p1,rr).start_address().as_u64(),p2_page(p4,rr).start_address().as_u64(),p2_page(p2,rr).start_address().as_u64(),p1_page(p4,rr).start_address().as_u64());
 }
 for (offset,frame) in [(0,0),(0x1000,0x2000),(0xffff800000000000,0x1000),(0x7ffffffff000,0),(0x7ffffffff000,0x1000),(0xfffffffffffff000,0x1000),(0xffffffffffffefff,0x1000),(0x800000000000,0),(0,1),(0,0x10000000000000),(0,0xffffffffff000)] {
  let result=std::panic::catch_unwind(||{let offset=VirtAddr::new(offset);let frame:PhysFrame=PhysFrame::from_start_address(PhysAddr::new(frame)).unwrap();offset_numeric(offset,frame).as_u64()});
  match result {Ok(value)=>println!("O|{offset}|{frame}|1|{value}"),Err(_)=>println!("O|{offset}|{frame}|0|0")}
 }
 for r in [0u16,1,255,256,510,511] {
  let i=PageTableIndex::new(r);let address=Page::<Size4KiB>::from_page_table_indices(i,i,i,i).start_address().as_u64();
  for (observed,word) in [(0x2000,0x2003),(0x3000,0x2003),(0x2000,0x2080),(0x2000,0x2081),(0,1),(0xffffffffff000,0x800ffffffffff003)] {
   let value=entry(word);let observed=PhysFrame::from_start_address(PhysAddr::new(observed)).unwrap();let result=observed_constructor(VirtAddr::new(address),observed,&value);
   let outcome=match result {Ok(_)=>0,Err(ObservedError::NotRecursive)=>1,Err(ObservedError::NotActive)=>match value.frame(){Err(FrameError::FrameNotPresent)=>2,Err(FrameError::HugeFrame)=>3,Ok(_)=>4}};
   println!("N|{address}|{}|{word}|{outcome}|{r}",observed.start_address().as_u64());
  }
  let value=entry(0);let observed=PhysFrame::from_start_address(PhysAddr::new(0)).unwrap();let bad=address^(1<<21);
  assert!(matches!(observed_constructor(VirtAddr::new(bad),observed,&value),Err(ObservedError::NotRecursive)));
  println!("N|{bad}|0|0|1|{r}");
  let value=entry(0x2001);let observed=PhysFrame::from_start_address(PhysAddr::new(0x2000)).unwrap();
  assert!(observed_constructor(VirtAddr::new(address+15),observed,&value).is_ok());
  println!("N|{}|8192|8193|0|{r}",address+15);
 }
}
