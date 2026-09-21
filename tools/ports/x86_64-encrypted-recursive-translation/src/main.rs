// SPDX-License-Identifier: MIT OR Apache-2.0
// Exact recursive pure-body mirror over disjoint initialized borrowed tables.
use std::{panic::{catch_unwind,AssertUnwindSafe}, process::Command};
use x86_64::{PhysAddr,VirtAddr,structures::{mem_encrypt::{enable_memory_encryption,MemoryEncryptionConfiguration as Config},paging::{PageTable,PageTableFlags,mapper::TranslateResult}}};
mod pinned;
fn store(table:&mut PageTable,index:usize,word:u64) {
 table[index].set_addr(PhysAddr::zero(),PageTableFlags::from_bits_retain(word));
}
fn observe(va:u64,words:[u64;4],_ids:[u64;3])->(u8,u64,u64,u64,u64) {
 let va=VirtAddr::new(va);
 let mut tables:[Box<PageTable>;4]=std::array::from_fn(|_|Box::new(PageTable::new()));
 for (n,index) in [usize::from(va.p4_index()),usize::from(va.p3_index()),usize::from(va.p2_index()),usize::from(va.p1_index())].into_iter().enumerate() {store(&mut tables[n],index,words[n]);}
 match pinned::translate(&tables[0],&tables[1],&tables[2],&tables[3],va) {
  TranslateResult::Mapped{frame,offset,flags}=>(1,frame.start_address().as_u64(),frame.size(),offset,flags.bits()),
  TranslateResult::NotMapped=>(0,0,0,0,0),
  TranslateResult::InvalidFrameAddress(_)=>panic!("masked alignment is unreachable"),
 }
}
fn profile(name:&str) {
 let mut mask=0x000ffffffffff000u64;let mut bits=0;
 if name!="disabled" {for part in name.split('_') {
  let pos:u8=part[1..].parse().unwrap();bits|=1u64<<pos;mask&=!(1u64<<pos);
  let configuration=if part.starts_with('e'){Config::EncryptedBit(pos)}else{Config::SharedBit(pos)};
  // SAFETY: no physical address/PTE exists yet; arithmetic profile only.
  unsafe{enable_memory_encryption(configuration)};
 }}
 let ids:Vec<u64>=(12..52).map(|n|1u64<<n).filter(|bit|bit&mask!=0).take(3).collect();
 let ids:[u64;3]=ids.try_into().unwrap();let parents=[ids[0]|1,ids[1]|1,ids[2]|1];
 let leaf=0x00012345abcdef03|bits;
 let base=[parents[0]|bits,parents[1]|bits,parents[2]|bits,leaf];
 let mut cases=vec![base,[parents[0],parents[1],parents[2],leaf],
  [parents[0],leaf|129,0,0],[parents[0],parents[1],leaf|129,0],
  [parents[0],parents[1],parents[2],512|bits],
  [parents[0],parents[1],parents[2],128],
  [parents[0],u64::MAX,0,0],[parents[0],parents[1],u64::MAX,0],
  [parents[0],parents[1],parents[2],u64::MAX]];
 for depth in 0..4 {let mut absent=[parents[0],parents[1],parents[2],leaf];absent[depth]=0;cases.push(absent);}
 cases.push([parents[0]|128,0,0,0]);
 cases.push([parents[0]&!1,parents[1],parents[2],leaf]);
 for va in [0xdeadbeefu64,0xffff920345678abc] {for (n,words) in cases.iter().enumerate() {
  let result=catch_unwind(AssertUnwindSafe(||observe(va,*words,ids)));
  let (kind,frame,size,offset,flags)=match result {Ok(value)=>value,Err(error)=>{
   let message=error.downcast_ref::<&str>().copied().or_else(||error.downcast_ref::<String>().map(String::as_str)).expect("unrecognized panic payload");
   let kind=match message {"level 4 entry has huge page bit set"=>2,"level 1 entry has huge page bit set"=>3,_=>panic!("unexpected panic {name}/{n}: {message}")};
   (kind,0,0,0,0)
  }};
  println!("{{\"profile\":{name:?},\"case\":{n},\"va\":{va},\"words\":{words:?},\"ids\":{ids:?},\"kind\":{kind},\"frame\":{frame},\"size\":{size},\"offset\":{offset},\"flags\":{flags}}}");
 }}
}
fn main(){
 std::panic::set_hook(Box::new(|_|{}));
 if let Some(name)=std::env::args().nth(1){profile(&name);return;}
 for name in ["disabled","e0","e7","e12","e21","e30","e47","s47","e51","s63","e47_s48_e47"] {
  let output=Command::new(std::env::current_exe().unwrap()).arg(name).output().unwrap();
  assert!(output.status.success(),"{name}: {:?}",output.stderr);
  print!("{}",String::from_utf8(output.stdout).unwrap());
 }
}
