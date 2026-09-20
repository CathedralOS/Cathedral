// SPDX-License-Identifier: MIT OR Apache-2.0
// Actual public read-only MappedPageTable calls against stable owned snapshots.
use std::{panic::{catch_unwind,AssertUnwindSafe}, process::Command};
use x86_64::{PhysAddr,VirtAddr,structures::{mem_encrypt::{enable_memory_encryption,MemoryEncryptionConfiguration as Config},paging::{PageTable,PageTableFlags,PhysFrame,mapper::{MappedPageTable,PageTableFrameMapping,Translate,TranslateResult}}}};
struct Captures { ids:[u64;3], tables:[Box<PageTable>;3] }
// SAFETY: reachable IDs select distinct stable initialized allocations owned by
// the mapper for its complete lifetime. Only immutable translate is called.
unsafe impl PageTableFrameMapping for Captures {
 fn frame_to_pointer(&self,frame:PhysFrame)->*mut PageTable {
  let id=frame.start_address().as_u64(); let n=self.ids.iter().position(|v|*v==id).expect("unregistered child");
  &*self.tables[n] as *const PageTable as *mut PageTable
 }
}
fn store(table:&mut PageTable,index:usize,word:u64) {
 // Flags retain all bits so this constructs the exact raw word even for malformed
 // entries. The base is zero and configuration precedes all address/PTE values.
 table[index].set_addr(PhysAddr::zero(),PageTableFlags::from_bits_retain(word));
}
fn observe(va:u64,words:[u64;4],ids:[u64;3])->(u8,u64,u64,u64,u64) {
 let va=VirtAddr::new(va);let mut root=Box::new(PageTable::new());
 let mut captures=Captures{ids,tables:std::array::from_fn(|_|Box::new(PageTable::new()))};
 store(&mut root,usize::from(va.p4_index()),words[0]);
 store(&mut captures.tables[0],usize::from(va.p3_index()),words[1]);
 store(&mut captures.tables[1],usize::from(va.p2_index()),words[2]);
 store(&mut captures.tables[2],usize::from(va.p1_index()),words[3]);
 // SAFETY: owned root plus the registry above form initialized read-only tables;
 // no live CR3, table installation, mapping mutation or reference fabrication.
 let mapper=unsafe{MappedPageTable::new(&mut root,captures)};
 match mapper.translate(va) {
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
  let (kind,frame,size,offset,flags)=match result {Ok(value)=>value,Err(_)=>{assert!(words[0]&129==129,"unexpected panic {name}/{n}");(2,0,0,0,0)}};
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
