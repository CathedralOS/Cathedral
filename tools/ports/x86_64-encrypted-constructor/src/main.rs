// SPDX-License-Identifier: MIT OR Apache-2.0
use std::{panic::catch_unwind,process::Command};
use x86_64::{PhysAddr,VirtAddr,structures::{mem_encrypt::{enable_memory_encryption,MemoryEncryptionConfiguration as Config},paging::{PhysFrame,Page,Size4KiB,PageTableIndex,PageTableFlags,page_table::{PageTableEntry,FrameError}}}};
include!("pinned.rs");
fn observed_frame(raw:u64)->PhysFrame {PhysFrame::from_start_address(PhysAddr::new(raw)).unwrap()}
fn profile(name:&str) {
 let mut bit=0;
 if name!="disabled" {for part in name.split('_') {let pos:u8=part[1..].parse().unwrap();bit=1u64<<pos;
 // SAFETY: isolated process, no prior physical/PTE values or live mappings.
 unsafe {enable_memory_encryption(if part.starts_with('e'){Config::EncryptedBit(pos)}else{Config::SharedBit(pos)})};
 }}
 let mut cases=Vec::new();
 for r in [0,255,256,511] {let i=PageTableIndex::new(r);let base=Page::<Size4KiB>::from_page_table_indices(i,i,i,i).start_address().as_u64();
 for (observed,word) in [(0,1),(0,bit|1),(bit&0xffffffffff000,bit|1),(0,0),(0,128),(0,129),(4096,4097),(8192,4097),(1,0),(1,129),(1<<48,(1<<48)|1),(0,u64::MAX)] {cases.push((base,observed,word));}
 cases.push((base|15,0,1));cases.push((base^(1<<21),u64::MAX,0));cases.push((base,1<<52,1));
 }
 cases.push((1<<47,u64::MAX,0));
 for (address,observed,word) in cases {
 let mut entry=PageTableEntry::new();entry.set_addr(PhysAddr::zero(),PageTableFlags::from_bits_retain(word));
 let (kind,index,frame)=match VirtAddr::try_new(address) {Err(_)=>(5,0,0),Ok(va)=>match catch_unwind(||observed_constructor(va,observed,&entry)) {
  Err(_)=>(6,0,0),Ok(Ok(i))=>(0,u16::from(i),entry.addr().as_u64()),Ok(Err(ObservedError::NotRecursive))=>(1,0,0),Ok(Err(ObservedError::NotActive))=>match entry.frame(){Err(FrameError::FrameNotPresent)=>(2,0,0),Err(FrameError::HugeFrame)=>(3,0,0),Ok(_)=>(4,0,0)}
 }};
 println!("{{\"profile\":{name:?},\"address\":{address},\"observed\":{observed},\"word\":{word},\"kind\":{kind},\"index\":{index},\"frame\":{frame}}}");
 }
}
fn main(){std::panic::set_hook(Box::new(|_|{}));if let Some(name)=std::env::args().nth(1){profile(&name);return;}
 for name in ["disabled","e0","e7","e12","e21","e47","s47","e51","s63","e47_s48_e47"] {let output=Command::new(std::env::current_exe().unwrap()).arg(name).output().unwrap();assert!(output.status.success());print!("{}",String::from_utf8(output.stdout).unwrap());}
}
