// SPDX-License-Identifier: MIT OR Apache-2.0
// Actual public Translate::translate_addr defaults, using supplied value results.
use std::{panic::catch_unwind,process::Command};
use x86_64::{PhysAddr,VirtAddr,structures::{mem_encrypt::{enable_memory_encryption,MemoryEncryptionConfiguration as Config},paging::{PhysFrame,PageTableFlags,mapper::{Translate,TranslateResult,MappedFrame}}}};
struct Source {frame:u64,size:u64,offset:u64,kind:u8}
impl Translate for Source {
 fn translate(&self,_:VirtAddr)->TranslateResult {
  if self.kind==1{return TranslateResult::NotMapped;}
  if self.kind==2{return TranslateResult::InvalidFrameAddress(PhysAddr::zero());}
  let address=PhysAddr::new(self.frame);
  let frame=match self.size {4096=>MappedFrame::Size4KiB(PhysFrame::from_start_address(address).unwrap()),2097152=>MappedFrame::Size2MiB(PhysFrame::from_start_address(address).unwrap()),1073741824=>MappedFrame::Size1GiB(PhysFrame::from_start_address(address).unwrap()),_=>panic!("unsupported size")};
  TranslateResult::Mapped{frame,offset:self.offset,flags:PageTableFlags::NO_EXECUTE}
 }
}
fn profile(name:&str) {
 let mut bit=0;
 if name!="disabled"{for part in name.split('_'){
  let pos:u8=part[1..].parse().unwrap();bit=1u64<<pos;
  let config=if part.starts_with('e'){Config::EncryptedBit(pos)}else{Config::SharedBit(pos)};
  // SAFETY: isolated process, no prior address or PTE values and no live mapping.
  unsafe{enable_memory_encryption(config)};
 }}
 let mut cases=Vec::new();
 for size in [4096,2097152,1073741824] {
  for offset in [0,1,size-1,size,size+1,bit] {cases.push(Source{frame:0,size,offset,kind:0});}
  cases.push(Source{frame:bit&!(size-1),size,offset:0,kind:0});
  cases.push(Source{frame:1<<48,size,offset:4096,kind:0});
 }
 cases.extend([Source{frame:0xffffffffff000,size:4096,offset:4096,kind:0},Source{frame:4096,size:4096,offset:u64::MAX,kind:0},Source{frame:4097,size:4096,offset:0,kind:0},Source{frame:0,size:8192,offset:0,kind:0},Source{frame:0,size:4096,offset:0,kind:1},Source{frame:0,size:4096,offset:0,kind:2}]);
 for source in cases {
  let outcome=catch_unwind(||source.translate_addr(VirtAddr::zero()));
  let (kind,value)=match outcome {Ok(Some(v))=>(0,v.as_u64()),Ok(None)=>(1,0),Err(_)=>(2,0)};
  println!("{{\"profile\":{name:?},\"frame\":{},\"size\":{},\"offset\":{},\"input_kind\":{},\"outcome\":{kind},\"value\":{value}}}",source.frame,source.size,source.offset,source.kind);
 }
}
fn main(){
 std::panic::set_hook(Box::new(|_|{}));
 if let Some(name)=std::env::args().nth(1){profile(&name);return;}
 for name in ["disabled","e0","e12","e21","e47","s47","s63","e47_s48_e47"] {
  let output=Command::new(std::env::current_exe().unwrap()).arg(name).output().unwrap();assert!(output.status.success());print!("{}",String::from_utf8(output.stdout).unwrap());
 }
}
