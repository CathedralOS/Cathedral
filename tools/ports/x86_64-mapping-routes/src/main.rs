// SPDX-License-Identifier: MIT OR Apache-2.0
use std::cell::UnsafeCell;
use x86_64::{PhysAddr,VirtAddr};
use x86_64::structures::paging::{PageTable,Page,PageSize,PhysFrame,Size4KiB,Size2MiB,Size1GiB,PageTableFlags,FrameAllocator};
use x86_64::structures::paging::mapper::{MappedPageTable,PageTableFrameMapping,Mapper,MapToError,UnmapError,FlagUpdateError,TranslateError};
const MASK:u64=0x000ffffffffff000;
struct Captures([Box<UnsafeCell<PageTable>>;3]);
// Test-only stable, distinct owned allocations; no active translation roots,
// concurrent references, cycles or aliases. UnsafeCell supports mapper writes.
unsafe impl PageTableFrameMapping for Captures {
 fn frame_to_pointer(&self,frame:PhysFrame)->*mut PageTable {
  let id=frame.start_address().as_u64();assert!([4096,8192,12288].contains(&id));self.0[(id/4096-1)as usize].get()
 }
}
struct Supply{frames:[u64;3],calls:usize}
// Generated fixtures supply only distinct, previously unlinked child tables.
unsafe impl FrameAllocator<Size4KiB> for Supply {
 fn allocate_frame(&mut self)->Option<PhysFrame>{let id=self.frames[self.calls];self.calls+=1;if id==0{None}else{Some(PhysFrame::from_start_address(PhysAddr::new(id)).unwrap())}}
}
fn store(table:&mut PageTable,index:usize,word:u64){table[index].set_addr(PhysAddr::new(word&MASK),PageTableFlags::from_bits_retain(word&!MASK));}
fn raw(table:&PageTable,index:usize)->u64{table[index].addr().as_u64()|table[index].flags().bits()}
fn panic_text(v:Box<dyn std::any::Any+Send>)->String{if let Some(s)=v.downcast_ref::<&str>(){s.to_string()}else if let Some(s)=v.downcast_ref::<String>(){s.clone()}else{panic!("unexpected panic")}}
#[derive(Debug,PartialEq)]struct Observation{kind:u8,payload:u64,words:[u64;4],calls:u8,zero:u8}
fn observe<S:PageSize+std::fmt::Debug>(op:&str,level:u8,page:u64,words:[u64;4],frames:[u64;3],frame:u64,flags:u64,parent:u64)->Observation
where for<'a> MappedPageTable<'a,Captures>:Mapper<S>{
 let indices=[((page>>39)&511)as usize,((page>>30)&511)as usize,((page>>21)&511)as usize,((page>>12)&511)as usize];
 assert!(indices.iter().all(|&i|i!=511));
 let mut root=Box::new(PageTable::new());store(&mut root,indices[0],words[0]);
 let mut captures=Captures(std::array::from_fn(|_|Box::new(UnsafeCell::new(PageTable::new()))));
 for n in 0..3{store(captures.0[n].get_mut(),indices[n+1],words[n+1]);captures.0[n].get_mut()[511].set_flags(PageTableFlags::NO_EXECUTE);}
 let mut mapper=unsafe{MappedPageTable::new(&mut root,captures)};
 let page=Page::<S>::from_start_address(VirtAddr::new(page)).unwrap();let mut supply=Supply{frames,calls:0};
 let f=PageTableFlags::from_bits_retain(flags);let pf=PageTableFlags::from_bits_retain(parent);
 let leaf=match S::SIZE{1073741824=>1,2097152=>2,_=>3};
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||match op{
  "map"=>match unsafe{mapper.map_to_with_table_flags(page,PhysFrame::<S>::from_start_address(PhysAddr::new(frame)).unwrap(),f,pf,&mut supply)}{
   Ok(flush)=>{flush.ignore();(0,frame)},Err(MapToError::FrameAllocationFailed)=>(6,0),Err(MapToError::ParentEntryHugePage)=>(2,0),Err(MapToError::PageAlreadyMapped(f))=>(4,f.start_address().as_u64())},
  "unmap"=>match mapper.unmap(page){Ok((f,flush))=>{flush.ignore();(0,f.start_address().as_u64())},Err(UnmapError::PageNotMapped)=>(1,0),Err(UnmapError::ParentEntryHugePage)=>(2,0),Err(UnmapError::InvalidFrameAddress(a))=>(3,a.as_u64())},
  "translate"=>match mapper.translate_page(page){Ok(f)=>(0,f.start_address().as_u64()),Err(TranslateError::PageNotMapped)=>(1,0),Err(TranslateError::ParentEntryHugePage)=>(2,0),Err(TranslateError::InvalidFrameAddress(a))=>(3,a.as_u64())},
  "update"=>match unsafe{mapper.update_flags(page,f)}{Ok(flush)=>{flush.ignore();(0,words[leaf]&MASK)},Err(FlagUpdateError::PageNotMapped)=>(1,0),Err(FlagUpdateError::ParentEntryHugePage)=>(2,0)},
  "parent"=>{let result=unsafe{match level{4=>mapper.set_flags_p4_entry(page,f),3=>mapper.set_flags_p3_entry(page,f),2=>mapper.set_flags_p2_entry(page,f),_=>panic!("bad level")}};match result{Ok(flush)=>{flush.ignore();(0,words[(4-level)as usize]&MASK)},Err(FlagUpdateError::PageNotMapped)=>(1,0),Err(FlagUpdateError::ParentEntryHugePage)=>(2,0)}},
  _=>panic!("bad operation")
 }));
 let (kind,payload)=match result{Ok(v)=>v,Err(v)=>{let s=panic_text(v);if s.contains("entry should be mapped"){(7,0)}else{assert!(s.contains("HUGE_PAGE"),"{s}");(5,0)}}};
 let mut observed=[raw(mapper.level_4_table(),indices[0]),0,0,0];let mut zero=0;
 for n in 0..3{let t=unsafe{&*mapper.page_table_frame_mapping().0[n].get()};observed[n+1]=raw(t,indices[n+1]);if t[511].is_unused(){zero|=1<<(n+1);}}
 Observation{kind,payload,words:observed,calls:supply.calls as u8,zero}
}
fn main(){std::panic::set_hook(Box::new(|_|{}));
 assert_eq!(observe::<Size1GiB>("map",4,18446623141389139968,[4097,0,12289,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:3221225472,words:[4099,3221225603,12289,0],calls:0,zero:0},"case 0: ordinary");
assert_eq!(observe::<Size1GiB>("map",4,18446623141389139968,[0,0,12289,0],[0,0,0],3221225472,3,3),Observation{kind:6,payload:0,words:[0,0,12289,0],calls:1,zero:0},"case 1: parent-0-0");
assert_eq!(observe::<Size1GiB>("map",4,18446623141389139968,[128,0,12289,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[131,0,12289,0],calls:0,zero:0},"case 2: parent-0-128");
assert_eq!(observe::<Size1GiB>("map",4,18446623141389139968,[4096,0,12289,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:3221225472,words:[4099,3221225603,12289,0],calls:0,zero:0},"case 3: parent-0-4096");
assert_eq!(observe::<Size1GiB>("map",4,18446623141389139968,[4225,0,12289,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4227,0,12289,0],calls:0,zero:0},"case 4: parent-0-4225");
assert_eq!(observe::<Size1GiB>("map",4,18446623141389139968,[4097,0,12289,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:3221225472,words:[4099,3221225603,12289,0],calls:0,zero:0},"case 5: leaf-0");
assert_eq!(observe::<Size1GiB>("map",4,18446623141389139968,[4097,512,12289,0],[0,0,0],3221225472,3,3),Observation{kind:4,payload:3221225472,words:[4099,512,12289,0],calls:0,zero:0},"case 6: leaf-512");
assert_eq!(observe::<Size1GiB>("map",4,18446623141389139968,[4097,2147487745,12289,0],[0,0,0],3221225472,3,3),Observation{kind:4,payload:3221225472,words:[4099,2147487745,12289,0],calls:0,zero:0},"case 7: leaf-2147487745");
assert_eq!(observe::<Size1GiB>("map",4,18446623141389139968,[4097,2147487873,12289,0],[0,0,0],3221225472,3,3),Observation{kind:4,payload:3221225472,words:[4099,2147487873,12289,0],calls:0,zero:0},"case 8: leaf-2147487873");
assert_eq!(observe::<Size1GiB>("unmap",4,18446623141389139968,[4097,2147483777,12289,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:2147483648,words:[4097,0,12289,0],calls:0,zero:0},"case 9: ordinary");
assert_eq!(observe::<Size1GiB>("unmap",4,18446623141389139968,[0,2147483777,12289,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[0,2147483777,12289,0],calls:0,zero:0},"case 10: parent-0-0");
assert_eq!(observe::<Size1GiB>("unmap",4,18446623141389139968,[128,2147483777,12289,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[128,2147483777,12289,0],calls:0,zero:0},"case 11: parent-0-128");
assert_eq!(observe::<Size1GiB>("unmap",4,18446623141389139968,[4096,2147483777,12289,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4096,2147483777,12289,0],calls:0,zero:0},"case 12: parent-0-4096");
assert_eq!(observe::<Size1GiB>("unmap",4,18446623141389139968,[4225,2147483777,12289,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4225,2147483777,12289,0],calls:0,zero:0},"case 13: parent-0-4225");
assert_eq!(observe::<Size1GiB>("unmap",4,18446623141389139968,[4097,0,12289,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,0,12289,0],calls:0,zero:0},"case 14: leaf-0");
assert_eq!(observe::<Size1GiB>("unmap",4,18446623141389139968,[4097,512,12289,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,512,12289,0],calls:0,zero:0},"case 15: leaf-512");
assert_eq!(observe::<Size1GiB>("unmap",4,18446623141389139968,[4097,2147487745,12289,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4097,2147487745,12289,0],calls:0,zero:0},"case 16: leaf-2147487745");
assert_eq!(observe::<Size1GiB>("unmap",4,18446623141389139968,[4097,2147487873,12289,0],[0,0,0],3221225472,3,3),Observation{kind:3,payload:2147487744,words:[4097,2147487873,12289,0],calls:0,zero:0},"case 17: leaf-2147487873");
assert_eq!(observe::<Size1GiB>("update",4,18446623141389139968,[4097,2147483777,12289,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:2147483648,words:[4097,2147483779,12289,0],calls:0,zero:0},"case 18: ordinary");
assert_eq!(observe::<Size1GiB>("update",4,18446623141389139968,[0,2147483777,12289,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[0,2147483777,12289,0],calls:0,zero:0},"case 19: parent-0-0");
assert_eq!(observe::<Size1GiB>("update",4,18446623141389139968,[128,2147483777,12289,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[128,2147483777,12289,0],calls:0,zero:0},"case 20: parent-0-128");
assert_eq!(observe::<Size1GiB>("update",4,18446623141389139968,[4096,2147483777,12289,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4096,2147483777,12289,0],calls:0,zero:0},"case 21: parent-0-4096");
assert_eq!(observe::<Size1GiB>("update",4,18446623141389139968,[4225,2147483777,12289,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4225,2147483777,12289,0],calls:0,zero:0},"case 22: parent-0-4225");
assert_eq!(observe::<Size1GiB>("update",4,18446623141389139968,[4097,0,12289,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,0,12289,0],calls:0,zero:0},"case 23: leaf-0");
assert_eq!(observe::<Size1GiB>("update",4,18446623141389139968,[4097,512,12289,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:0,words:[4097,131,12289,0],calls:0,zero:0},"case 24: leaf-512");
assert_eq!(observe::<Size1GiB>("update",4,18446623141389139968,[4097,2147487745,12289,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:2147487744,words:[4097,2147487875,12289,0],calls:0,zero:0},"case 25: leaf-2147487745");
assert_eq!(observe::<Size1GiB>("update",4,18446623141389139968,[4097,2147487873,12289,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:2147487744,words:[4097,2147487875,12289,0],calls:0,zero:0},"case 26: leaf-2147487873");
assert_eq!(observe::<Size1GiB>("translate",4,18446623141389139968,[4097,2147483777,12289,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:2147483648,words:[4097,2147483777,12289,0],calls:0,zero:0},"case 27: ordinary");
assert_eq!(observe::<Size1GiB>("translate",4,18446623141389139968,[0,2147483777,12289,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[0,2147483777,12289,0],calls:0,zero:0},"case 28: parent-0-0");
assert_eq!(observe::<Size1GiB>("translate",4,18446623141389139968,[128,2147483777,12289,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[128,2147483777,12289,0],calls:0,zero:0},"case 29: parent-0-128");
assert_eq!(observe::<Size1GiB>("translate",4,18446623141389139968,[4096,2147483777,12289,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4096,2147483777,12289,0],calls:0,zero:0},"case 30: parent-0-4096");
assert_eq!(observe::<Size1GiB>("translate",4,18446623141389139968,[4225,2147483777,12289,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4225,2147483777,12289,0],calls:0,zero:0},"case 31: parent-0-4225");
assert_eq!(observe::<Size1GiB>("translate",4,18446623141389139968,[4097,0,12289,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,0,12289,0],calls:0,zero:0},"case 32: leaf-0");
assert_eq!(observe::<Size1GiB>("translate",4,18446623141389139968,[4097,512,12289,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:0,words:[4097,512,12289,0],calls:0,zero:0},"case 33: leaf-512");
assert_eq!(observe::<Size1GiB>("translate",4,18446623141389139968,[4097,2147487745,12289,0],[0,0,0],3221225472,3,3),Observation{kind:3,payload:2147487744,words:[4097,2147487745,12289,0],calls:0,zero:0},"case 34: leaf-2147487745");
assert_eq!(observe::<Size1GiB>("translate",4,18446623141389139968,[4097,2147487873,12289,0],[0,0,0],3221225472,3,3),Observation{kind:3,payload:2147487744,words:[4097,2147487873,12289,0],calls:0,zero:0},"case 35: leaf-2147487873");
assert_eq!(observe::<Size1GiB>("map",4,18446623141389139968,[0,8193,12289,512],[0,0,0],3221225472,3,3),Observation{kind:6,payload:0,words:[0,8193,12289,512],calls:1,zero:0},"case 36: new-tables-0");
assert_eq!(observe::<Size1GiB>("map",4,18446623141389139968,[0,8193,12289,512],[4096,0,0],3221225472,3,3),Observation{kind:0,payload:3221225472,words:[4099,3221225603,12289,512],calls:1,zero:2},"case 37: new-tables-1");
assert_eq!(observe::<Size1GiB>("map",4,18446623141389139968,[0,8193,12289,512],[4096,8192,12288],3221225472,3,0),Observation{kind:7,payload:0,words:[4096,8193,12289,512],calls:1,zero:0},"case 38: new-parent-flags-0");
assert_eq!(observe::<Size1GiB>("map",4,18446623141389139968,[0,8193,12289,512],[4096,8192,12288],3221225472,3,128),Observation{kind:5,payload:0,words:[0,8193,12289,512],calls:1,zero:0},"case 39: new-parent-flags-128");
assert_eq!(observe::<Size1GiB>("map",4,18446623141389139968,[0,8193,12289,512],[4096,8192,12288],3221225472,3,129),Observation{kind:5,payload:0,words:[0,8193,12289,512],calls:1,zero:0},"case 40: new-parent-flags-129");
assert_eq!(observe::<Size1GiB>("map",4,18446623141389139968,[0,8193,12289,512],[4096,8192,12288],3221225472,3,7),Observation{kind:0,payload:3221225472,words:[4103,3221225603,12289,512],calls:1,zero:2},"case 41: new-parent-flags-7");
assert_eq!(observe::<Size1GiB>("map",4,18446623141389139968,[4097,0,12289,0],[0,0,0],3221225472,128,3),Observation{kind:0,payload:3221225472,words:[4099,3221225600,12289,0],calls:0,zero:0},"case 42: leaf-huge-flag");
assert_eq!(observe::<Size1GiB>("update",4,18446623141389139968,[4097,2147483777,12289,0],[0,0,0],3221225472,18446744073709551615,3),Observation{kind:0,payload:2147483648,words:[4097,18446744073709551615,12289,0],calls:0,zero:0},"case 43: all-leaf-flags");
assert_eq!(observe::<Size1GiB>("parent",2,18446623141389139968,[4097,2147483777,12289,0],[0,0,0],3221225472,0,3),Observation{kind:2,payload:0,words:[4097,2147483777,12289,0],calls:0,zero:0},"case 44: parent-clear-2");
assert_eq!(observe::<Size1GiB>("parent",2,18446623141389139968,[4097,8193,0,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4097,8193,0,0],calls:0,zero:0},"case 45: parent-unused-2");
assert_eq!(observe::<Size1GiB>("parent",3,18446623141389139968,[4097,2147483777,12289,0],[0,0,0],3221225472,0,3),Observation{kind:2,payload:0,words:[4097,2147483777,12289,0],calls:0,zero:0},"case 46: parent-clear-3");
assert_eq!(observe::<Size1GiB>("parent",3,18446623141389139968,[4097,0,12289,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4097,0,12289,0],calls:0,zero:0},"case 47: parent-unused-3");
assert_eq!(observe::<Size1GiB>("parent",4,18446623141389139968,[4097,2147483777,12289,0],[0,0,0],3221225472,0,3),Observation{kind:0,payload:4096,words:[4096,2147483777,12289,0],calls:0,zero:0},"case 48: parent-clear-4");
assert_eq!(observe::<Size1GiB>("parent",4,18446623141389139968,[0,8193,12289,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[0,8193,12289,0],calls:0,zero:0},"case 49: parent-unused-4");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[4097,8193,0,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:3221225472,words:[4099,8195,3221225603,0],calls:0,zero:0},"case 50: ordinary");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[0,8193,0,0],[0,0,0],3221225472,3,3),Observation{kind:6,payload:0,words:[0,8193,0,0],calls:1,zero:0},"case 51: parent-0-0");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[128,8193,0,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[131,8193,0,0],calls:0,zero:0},"case 52: parent-0-128");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[4096,8193,0,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:3221225472,words:[4099,8195,3221225603,0],calls:0,zero:0},"case 53: parent-0-4096");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[4225,8193,0,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4227,8193,0,0],calls:0,zero:0},"case 54: parent-0-4225");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[4097,0,0,0],[0,0,0],3221225472,3,3),Observation{kind:6,payload:0,words:[4099,0,0,0],calls:1,zero:0},"case 55: parent-1-0");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[4097,128,0,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4099,131,0,0],calls:0,zero:0},"case 56: parent-1-128");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[4097,8192,0,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:3221225472,words:[4099,8195,3221225603,0],calls:0,zero:0},"case 57: parent-1-8192");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[4097,8321,0,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4099,8323,0,0],calls:0,zero:0},"case 58: parent-1-8321");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[4097,8193,0,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:3221225472,words:[4099,8195,3221225603,0],calls:0,zero:0},"case 59: leaf-0");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[4097,8193,512,0],[0,0,0],3221225472,3,3),Observation{kind:4,payload:3221225472,words:[4099,8195,512,0],calls:0,zero:0},"case 60: leaf-512");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[4097,8193,2147487745,0],[0,0,0],3221225472,3,3),Observation{kind:4,payload:3221225472,words:[4099,8195,2147487745,0],calls:0,zero:0},"case 61: leaf-2147487745");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[4097,8193,2147487873,0],[0,0,0],3221225472,3,3),Observation{kind:4,payload:3221225472,words:[4099,8195,2147487873,0],calls:0,zero:0},"case 62: leaf-2147487873");
assert_eq!(observe::<Size2MiB>("unmap",4,18446623141389139968,[4097,8193,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:2147483648,words:[4097,8193,0,0],calls:0,zero:0},"case 63: ordinary");
assert_eq!(observe::<Size2MiB>("unmap",4,18446623141389139968,[0,8193,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[0,8193,2147483777,0],calls:0,zero:0},"case 64: parent-0-0");
assert_eq!(observe::<Size2MiB>("unmap",4,18446623141389139968,[128,8193,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[128,8193,2147483777,0],calls:0,zero:0},"case 65: parent-0-128");
assert_eq!(observe::<Size2MiB>("unmap",4,18446623141389139968,[4096,8193,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4096,8193,2147483777,0],calls:0,zero:0},"case 66: parent-0-4096");
assert_eq!(observe::<Size2MiB>("unmap",4,18446623141389139968,[4225,8193,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4225,8193,2147483777,0],calls:0,zero:0},"case 67: parent-0-4225");
assert_eq!(observe::<Size2MiB>("unmap",4,18446623141389139968,[4097,0,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,0,2147483777,0],calls:0,zero:0},"case 68: parent-1-0");
assert_eq!(observe::<Size2MiB>("unmap",4,18446623141389139968,[4097,128,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,128,2147483777,0],calls:0,zero:0},"case 69: parent-1-128");
assert_eq!(observe::<Size2MiB>("unmap",4,18446623141389139968,[4097,8192,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8192,2147483777,0],calls:0,zero:0},"case 70: parent-1-8192");
assert_eq!(observe::<Size2MiB>("unmap",4,18446623141389139968,[4097,8321,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4097,8321,2147483777,0],calls:0,zero:0},"case 71: parent-1-8321");
assert_eq!(observe::<Size2MiB>("unmap",4,18446623141389139968,[4097,8193,0,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8193,0,0],calls:0,zero:0},"case 72: leaf-0");
assert_eq!(observe::<Size2MiB>("unmap",4,18446623141389139968,[4097,8193,512,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8193,512,0],calls:0,zero:0},"case 73: leaf-512");
assert_eq!(observe::<Size2MiB>("unmap",4,18446623141389139968,[4097,8193,2147487745,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4097,8193,2147487745,0],calls:0,zero:0},"case 74: leaf-2147487745");
assert_eq!(observe::<Size2MiB>("unmap",4,18446623141389139968,[4097,8193,2147487873,0],[0,0,0],3221225472,3,3),Observation{kind:3,payload:2147487744,words:[4097,8193,2147487873,0],calls:0,zero:0},"case 75: leaf-2147487873");
assert_eq!(observe::<Size2MiB>("update",4,18446623141389139968,[4097,8193,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:2147483648,words:[4097,8193,2147483779,0],calls:0,zero:0},"case 76: ordinary");
assert_eq!(observe::<Size2MiB>("update",4,18446623141389139968,[0,8193,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[0,8193,2147483777,0],calls:0,zero:0},"case 77: parent-0-0");
assert_eq!(observe::<Size2MiB>("update",4,18446623141389139968,[128,8193,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[128,8193,2147483777,0],calls:0,zero:0},"case 78: parent-0-128");
assert_eq!(observe::<Size2MiB>("update",4,18446623141389139968,[4096,8193,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4096,8193,2147483777,0],calls:0,zero:0},"case 79: parent-0-4096");
assert_eq!(observe::<Size2MiB>("update",4,18446623141389139968,[4225,8193,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4225,8193,2147483777,0],calls:0,zero:0},"case 80: parent-0-4225");
assert_eq!(observe::<Size2MiB>("update",4,18446623141389139968,[4097,0,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,0,2147483777,0],calls:0,zero:0},"case 81: parent-1-0");
assert_eq!(observe::<Size2MiB>("update",4,18446623141389139968,[4097,128,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,128,2147483777,0],calls:0,zero:0},"case 82: parent-1-128");
assert_eq!(observe::<Size2MiB>("update",4,18446623141389139968,[4097,8192,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8192,2147483777,0],calls:0,zero:0},"case 83: parent-1-8192");
assert_eq!(observe::<Size2MiB>("update",4,18446623141389139968,[4097,8321,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4097,8321,2147483777,0],calls:0,zero:0},"case 84: parent-1-8321");
assert_eq!(observe::<Size2MiB>("update",4,18446623141389139968,[4097,8193,0,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8193,0,0],calls:0,zero:0},"case 85: leaf-0");
assert_eq!(observe::<Size2MiB>("update",4,18446623141389139968,[4097,8193,512,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:0,words:[4097,8193,131,0],calls:0,zero:0},"case 86: leaf-512");
assert_eq!(observe::<Size2MiB>("update",4,18446623141389139968,[4097,8193,2147487745,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:2147487744,words:[4097,8193,2147487875,0],calls:0,zero:0},"case 87: leaf-2147487745");
assert_eq!(observe::<Size2MiB>("update",4,18446623141389139968,[4097,8193,2147487873,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:2147487744,words:[4097,8193,2147487875,0],calls:0,zero:0},"case 88: leaf-2147487873");
assert_eq!(observe::<Size2MiB>("translate",4,18446623141389139968,[4097,8193,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:2147483648,words:[4097,8193,2147483777,0],calls:0,zero:0},"case 89: ordinary");
assert_eq!(observe::<Size2MiB>("translate",4,18446623141389139968,[0,8193,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[0,8193,2147483777,0],calls:0,zero:0},"case 90: parent-0-0");
assert_eq!(observe::<Size2MiB>("translate",4,18446623141389139968,[128,8193,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[128,8193,2147483777,0],calls:0,zero:0},"case 91: parent-0-128");
assert_eq!(observe::<Size2MiB>("translate",4,18446623141389139968,[4096,8193,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4096,8193,2147483777,0],calls:0,zero:0},"case 92: parent-0-4096");
assert_eq!(observe::<Size2MiB>("translate",4,18446623141389139968,[4225,8193,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4225,8193,2147483777,0],calls:0,zero:0},"case 93: parent-0-4225");
assert_eq!(observe::<Size2MiB>("translate",4,18446623141389139968,[4097,0,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,0,2147483777,0],calls:0,zero:0},"case 94: parent-1-0");
assert_eq!(observe::<Size2MiB>("translate",4,18446623141389139968,[4097,128,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,128,2147483777,0],calls:0,zero:0},"case 95: parent-1-128");
assert_eq!(observe::<Size2MiB>("translate",4,18446623141389139968,[4097,8192,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8192,2147483777,0],calls:0,zero:0},"case 96: parent-1-8192");
assert_eq!(observe::<Size2MiB>("translate",4,18446623141389139968,[4097,8321,2147483777,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4097,8321,2147483777,0],calls:0,zero:0},"case 97: parent-1-8321");
assert_eq!(observe::<Size2MiB>("translate",4,18446623141389139968,[4097,8193,0,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8193,0,0],calls:0,zero:0},"case 98: leaf-0");
assert_eq!(observe::<Size2MiB>("translate",4,18446623141389139968,[4097,8193,512,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:0,words:[4097,8193,512,0],calls:0,zero:0},"case 99: leaf-512");
assert_eq!(observe::<Size2MiB>("translate",4,18446623141389139968,[4097,8193,2147487745,0],[0,0,0],3221225472,3,3),Observation{kind:3,payload:2147487744,words:[4097,8193,2147487745,0],calls:0,zero:0},"case 100: leaf-2147487745");
assert_eq!(observe::<Size2MiB>("translate",4,18446623141389139968,[4097,8193,2147487873,0],[0,0,0],3221225472,3,3),Observation{kind:3,payload:2147487744,words:[4097,8193,2147487873,0],calls:0,zero:0},"case 101: leaf-2147487873");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[0,8193,12289,512],[0,0,0],3221225472,3,3),Observation{kind:6,payload:0,words:[0,8193,12289,512],calls:1,zero:0},"case 102: new-tables-0");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[0,8193,12289,512],[4096,0,0],3221225472,3,3),Observation{kind:6,payload:0,words:[4099,0,12289,512],calls:2,zero:2},"case 103: new-tables-1");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[0,8193,12289,512],[4096,8192,0],3221225472,3,3),Observation{kind:0,payload:3221225472,words:[4099,8195,3221225603,512],calls:2,zero:6},"case 104: new-tables-2");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[0,8193,12289,512],[4096,8192,12288],3221225472,3,0),Observation{kind:7,payload:0,words:[4096,8193,12289,512],calls:1,zero:0},"case 105: new-parent-flags-0");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[0,8193,12289,512],[4096,8192,12288],3221225472,3,128),Observation{kind:5,payload:0,words:[0,8193,12289,512],calls:1,zero:0},"case 106: new-parent-flags-128");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[0,8193,12289,512],[4096,8192,12288],3221225472,3,129),Observation{kind:5,payload:0,words:[0,8193,12289,512],calls:1,zero:0},"case 107: new-parent-flags-129");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[0,8193,12289,512],[4096,8192,12288],3221225472,3,7),Observation{kind:0,payload:3221225472,words:[4103,8199,3221225603,512],calls:2,zero:6},"case 108: new-parent-flags-7");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[4097,0,12289,512],[0,0,0],3221225472,3,3),Observation{kind:6,payload:0,words:[4099,0,12289,512],calls:1,zero:0},"case 109: new-suffix-1-0");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[4097,0,12289,512],[8192,0,0],3221225472,3,3),Observation{kind:0,payload:3221225472,words:[4099,8195,3221225603,512],calls:1,zero:4},"case 110: new-suffix-1-1");
assert_eq!(observe::<Size2MiB>("map",4,18446623141389139968,[4097,8193,0,0],[0,0,0],3221225472,128,3),Observation{kind:0,payload:3221225472,words:[4099,8195,3221225600,0],calls:0,zero:0},"case 111: leaf-huge-flag");
assert_eq!(observe::<Size2MiB>("update",4,18446623141389139968,[4097,8193,2147483777,0],[0,0,0],3221225472,18446744073709551615,3),Observation{kind:0,payload:2147483648,words:[4097,8193,18446744073709551615,0],calls:0,zero:0},"case 112: all-leaf-flags");
assert_eq!(observe::<Size2MiB>("parent",2,18446623141389139968,[4097,8193,2147483777,0],[0,0,0],3221225472,0,3),Observation{kind:2,payload:0,words:[4097,8193,2147483777,0],calls:0,zero:0},"case 113: parent-clear-2");
assert_eq!(observe::<Size2MiB>("parent",2,18446623141389139968,[4097,8193,0,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4097,8193,0,0],calls:0,zero:0},"case 114: parent-unused-2");
assert_eq!(observe::<Size2MiB>("parent",3,18446623141389139968,[4097,8193,2147483777,0],[0,0,0],3221225472,0,3),Observation{kind:0,payload:8192,words:[4097,8192,2147483777,0],calls:0,zero:0},"case 115: parent-clear-3");
assert_eq!(observe::<Size2MiB>("parent",3,18446623141389139968,[4097,0,12289,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,0,12289,0],calls:0,zero:0},"case 116: parent-unused-3");
assert_eq!(observe::<Size2MiB>("parent",4,18446623141389139968,[4097,8193,2147483777,0],[0,0,0],3221225472,0,3),Observation{kind:0,payload:4096,words:[4096,8193,2147483777,0],calls:0,zero:0},"case 117: parent-clear-4");
assert_eq!(observe::<Size2MiB>("parent",4,18446623141389139968,[0,8193,12289,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[0,8193,12289,0],calls:0,zero:0},"case 118: parent-unused-4");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[4097,8193,12289,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:3221225472,words:[4099,8195,12291,3221225475],calls:0,zero:0},"case 119: ordinary");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[0,8193,12289,0],[0,0,0],3221225472,3,3),Observation{kind:6,payload:0,words:[0,8193,12289,0],calls:1,zero:0},"case 120: parent-0-0");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[128,8193,12289,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[131,8193,12289,0],calls:0,zero:0},"case 121: parent-0-128");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[4096,8193,12289,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:3221225472,words:[4099,8195,12291,3221225475],calls:0,zero:0},"case 122: parent-0-4096");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[4225,8193,12289,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4227,8193,12289,0],calls:0,zero:0},"case 123: parent-0-4225");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[4097,0,12289,0],[0,0,0],3221225472,3,3),Observation{kind:6,payload:0,words:[4099,0,12289,0],calls:1,zero:0},"case 124: parent-1-0");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[4097,128,12289,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4099,131,12289,0],calls:0,zero:0},"case 125: parent-1-128");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[4097,8192,12289,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:3221225472,words:[4099,8195,12291,3221225475],calls:0,zero:0},"case 126: parent-1-8192");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[4097,8321,12289,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4099,8323,12289,0],calls:0,zero:0},"case 127: parent-1-8321");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[4097,8193,0,0],[0,0,0],3221225472,3,3),Observation{kind:6,payload:0,words:[4099,8195,0,0],calls:1,zero:0},"case 128: parent-2-0");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[4097,8193,128,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4099,8195,131,0],calls:0,zero:0},"case 129: parent-2-128");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[4097,8193,12288,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:3221225472,words:[4099,8195,12291,3221225475],calls:0,zero:0},"case 130: parent-2-12288");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[4097,8193,12417,0],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4099,8195,12419,0],calls:0,zero:0},"case 131: parent-2-12417");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[4097,8193,12289,0],[0,0,0],3221225472,3,3),Observation{kind:0,payload:3221225472,words:[4099,8195,12291,3221225475],calls:0,zero:0},"case 132: leaf-0");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[4097,8193,12289,512],[0,0,0],3221225472,3,3),Observation{kind:4,payload:3221225472,words:[4099,8195,12291,512],calls:0,zero:0},"case 133: leaf-512");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[4097,8193,12289,2147487745],[0,0,0],3221225472,3,3),Observation{kind:4,payload:3221225472,words:[4099,8195,12291,2147487745],calls:0,zero:0},"case 134: leaf-2147487745");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[4097,8193,12289,2147487873],[0,0,0],3221225472,3,3),Observation{kind:4,payload:3221225472,words:[4099,8195,12291,2147487873],calls:0,zero:0},"case 135: leaf-2147487873");
assert_eq!(observe::<Size4KiB>("unmap",4,18446623141389139968,[4097,8193,12289,2147483649],[0,0,0],3221225472,3,3),Observation{kind:0,payload:2147483648,words:[4097,8193,12289,0],calls:0,zero:0},"case 136: ordinary");
assert_eq!(observe::<Size4KiB>("unmap",4,18446623141389139968,[0,8193,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[0,8193,12289,2147483777],calls:0,zero:0},"case 137: parent-0-0");
assert_eq!(observe::<Size4KiB>("unmap",4,18446623141389139968,[128,8193,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[128,8193,12289,2147483777],calls:0,zero:0},"case 138: parent-0-128");
assert_eq!(observe::<Size4KiB>("unmap",4,18446623141389139968,[4096,8193,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4096,8193,12289,2147483777],calls:0,zero:0},"case 139: parent-0-4096");
assert_eq!(observe::<Size4KiB>("unmap",4,18446623141389139968,[4225,8193,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4225,8193,12289,2147483777],calls:0,zero:0},"case 140: parent-0-4225");
assert_eq!(observe::<Size4KiB>("unmap",4,18446623141389139968,[4097,0,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,0,12289,2147483777],calls:0,zero:0},"case 141: parent-1-0");
assert_eq!(observe::<Size4KiB>("unmap",4,18446623141389139968,[4097,128,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,128,12289,2147483777],calls:0,zero:0},"case 142: parent-1-128");
assert_eq!(observe::<Size4KiB>("unmap",4,18446623141389139968,[4097,8192,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8192,12289,2147483777],calls:0,zero:0},"case 143: parent-1-8192");
assert_eq!(observe::<Size4KiB>("unmap",4,18446623141389139968,[4097,8321,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4097,8321,12289,2147483777],calls:0,zero:0},"case 144: parent-1-8321");
assert_eq!(observe::<Size4KiB>("unmap",4,18446623141389139968,[4097,8193,0,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8193,0,2147483777],calls:0,zero:0},"case 145: parent-2-0");
assert_eq!(observe::<Size4KiB>("unmap",4,18446623141389139968,[4097,8193,128,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8193,128,2147483777],calls:0,zero:0},"case 146: parent-2-128");
assert_eq!(observe::<Size4KiB>("unmap",4,18446623141389139968,[4097,8193,12288,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8193,12288,2147483777],calls:0,zero:0},"case 147: parent-2-12288");
assert_eq!(observe::<Size4KiB>("unmap",4,18446623141389139968,[4097,8193,12417,2147483777],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4097,8193,12417,2147483777],calls:0,zero:0},"case 148: parent-2-12417");
assert_eq!(observe::<Size4KiB>("unmap",4,18446623141389139968,[4097,8193,12289,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8193,12289,0],calls:0,zero:0},"case 149: leaf-0");
assert_eq!(observe::<Size4KiB>("unmap",4,18446623141389139968,[4097,8193,12289,512],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8193,12289,512],calls:0,zero:0},"case 150: leaf-512");
assert_eq!(observe::<Size4KiB>("unmap",4,18446623141389139968,[4097,8193,12289,2147487745],[0,0,0],3221225472,3,3),Observation{kind:0,payload:2147487744,words:[4097,8193,12289,0],calls:0,zero:0},"case 151: leaf-2147487745");
assert_eq!(observe::<Size4KiB>("unmap",4,18446623141389139968,[4097,8193,12289,2147487873],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4097,8193,12289,2147487873],calls:0,zero:0},"case 152: leaf-2147487873");
assert_eq!(observe::<Size4KiB>("update",4,18446623141389139968,[4097,8193,12289,2147483649],[0,0,0],3221225472,3,3),Observation{kind:0,payload:2147483648,words:[4097,8193,12289,2147483651],calls:0,zero:0},"case 153: ordinary");
assert_eq!(observe::<Size4KiB>("update",4,18446623141389139968,[0,8193,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[0,8193,12289,2147483777],calls:0,zero:0},"case 154: parent-0-0");
assert_eq!(observe::<Size4KiB>("update",4,18446623141389139968,[128,8193,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[128,8193,12289,2147483777],calls:0,zero:0},"case 155: parent-0-128");
assert_eq!(observe::<Size4KiB>("update",4,18446623141389139968,[4096,8193,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4096,8193,12289,2147483777],calls:0,zero:0},"case 156: parent-0-4096");
assert_eq!(observe::<Size4KiB>("update",4,18446623141389139968,[4225,8193,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4225,8193,12289,2147483777],calls:0,zero:0},"case 157: parent-0-4225");
assert_eq!(observe::<Size4KiB>("update",4,18446623141389139968,[4097,0,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,0,12289,2147483777],calls:0,zero:0},"case 158: parent-1-0");
assert_eq!(observe::<Size4KiB>("update",4,18446623141389139968,[4097,128,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,128,12289,2147483777],calls:0,zero:0},"case 159: parent-1-128");
assert_eq!(observe::<Size4KiB>("update",4,18446623141389139968,[4097,8192,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8192,12289,2147483777],calls:0,zero:0},"case 160: parent-1-8192");
assert_eq!(observe::<Size4KiB>("update",4,18446623141389139968,[4097,8321,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4097,8321,12289,2147483777],calls:0,zero:0},"case 161: parent-1-8321");
assert_eq!(observe::<Size4KiB>("update",4,18446623141389139968,[4097,8193,0,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8193,0,2147483777],calls:0,zero:0},"case 162: parent-2-0");
assert_eq!(observe::<Size4KiB>("update",4,18446623141389139968,[4097,8193,128,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8193,128,2147483777],calls:0,zero:0},"case 163: parent-2-128");
assert_eq!(observe::<Size4KiB>("update",4,18446623141389139968,[4097,8193,12288,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8193,12288,2147483777],calls:0,zero:0},"case 164: parent-2-12288");
assert_eq!(observe::<Size4KiB>("update",4,18446623141389139968,[4097,8193,12417,2147483777],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4097,8193,12417,2147483777],calls:0,zero:0},"case 165: parent-2-12417");
assert_eq!(observe::<Size4KiB>("update",4,18446623141389139968,[4097,8193,12289,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8193,12289,0],calls:0,zero:0},"case 166: leaf-0");
assert_eq!(observe::<Size4KiB>("update",4,18446623141389139968,[4097,8193,12289,512],[0,0,0],3221225472,3,3),Observation{kind:0,payload:0,words:[4097,8193,12289,3],calls:0,zero:0},"case 167: leaf-512");
assert_eq!(observe::<Size4KiB>("update",4,18446623141389139968,[4097,8193,12289,2147487745],[0,0,0],3221225472,3,3),Observation{kind:0,payload:2147487744,words:[4097,8193,12289,2147487747],calls:0,zero:0},"case 168: leaf-2147487745");
assert_eq!(observe::<Size4KiB>("update",4,18446623141389139968,[4097,8193,12289,2147487873],[0,0,0],3221225472,3,3),Observation{kind:0,payload:2147487744,words:[4097,8193,12289,2147487747],calls:0,zero:0},"case 169: leaf-2147487873");
assert_eq!(observe::<Size4KiB>("translate",4,18446623141389139968,[4097,8193,12289,2147483649],[0,0,0],3221225472,3,3),Observation{kind:0,payload:2147483648,words:[4097,8193,12289,2147483649],calls:0,zero:0},"case 170: ordinary");
assert_eq!(observe::<Size4KiB>("translate",4,18446623141389139968,[0,8193,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[0,8193,12289,2147483777],calls:0,zero:0},"case 171: parent-0-0");
assert_eq!(observe::<Size4KiB>("translate",4,18446623141389139968,[128,8193,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[128,8193,12289,2147483777],calls:0,zero:0},"case 172: parent-0-128");
assert_eq!(observe::<Size4KiB>("translate",4,18446623141389139968,[4096,8193,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4096,8193,12289,2147483777],calls:0,zero:0},"case 173: parent-0-4096");
assert_eq!(observe::<Size4KiB>("translate",4,18446623141389139968,[4225,8193,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4225,8193,12289,2147483777],calls:0,zero:0},"case 174: parent-0-4225");
assert_eq!(observe::<Size4KiB>("translate",4,18446623141389139968,[4097,0,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,0,12289,2147483777],calls:0,zero:0},"case 175: parent-1-0");
assert_eq!(observe::<Size4KiB>("translate",4,18446623141389139968,[4097,128,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,128,12289,2147483777],calls:0,zero:0},"case 176: parent-1-128");
assert_eq!(observe::<Size4KiB>("translate",4,18446623141389139968,[4097,8192,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8192,12289,2147483777],calls:0,zero:0},"case 177: parent-1-8192");
assert_eq!(observe::<Size4KiB>("translate",4,18446623141389139968,[4097,8321,12289,2147483777],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4097,8321,12289,2147483777],calls:0,zero:0},"case 178: parent-1-8321");
assert_eq!(observe::<Size4KiB>("translate",4,18446623141389139968,[4097,8193,0,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8193,0,2147483777],calls:0,zero:0},"case 179: parent-2-0");
assert_eq!(observe::<Size4KiB>("translate",4,18446623141389139968,[4097,8193,128,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8193,128,2147483777],calls:0,zero:0},"case 180: parent-2-128");
assert_eq!(observe::<Size4KiB>("translate",4,18446623141389139968,[4097,8193,12288,2147483777],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8193,12288,2147483777],calls:0,zero:0},"case 181: parent-2-12288");
assert_eq!(observe::<Size4KiB>("translate",4,18446623141389139968,[4097,8193,12417,2147483777],[0,0,0],3221225472,3,3),Observation{kind:2,payload:0,words:[4097,8193,12417,2147483777],calls:0,zero:0},"case 182: parent-2-12417");
assert_eq!(observe::<Size4KiB>("translate",4,18446623141389139968,[4097,8193,12289,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8193,12289,0],calls:0,zero:0},"case 183: leaf-0");
assert_eq!(observe::<Size4KiB>("translate",4,18446623141389139968,[4097,8193,12289,512],[0,0,0],3221225472,3,3),Observation{kind:0,payload:0,words:[4097,8193,12289,512],calls:0,zero:0},"case 184: leaf-512");
assert_eq!(observe::<Size4KiB>("translate",4,18446623141389139968,[4097,8193,12289,2147487745],[0,0,0],3221225472,3,3),Observation{kind:0,payload:2147487744,words:[4097,8193,12289,2147487745],calls:0,zero:0},"case 185: leaf-2147487745");
assert_eq!(observe::<Size4KiB>("translate",4,18446623141389139968,[4097,8193,12289,2147487873],[0,0,0],3221225472,3,3),Observation{kind:0,payload:2147487744,words:[4097,8193,12289,2147487873],calls:0,zero:0},"case 186: leaf-2147487873");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[0,8193,12289,512],[0,0,0],3221225472,3,3),Observation{kind:6,payload:0,words:[0,8193,12289,512],calls:1,zero:0},"case 187: new-tables-0");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[0,8193,12289,512],[4096,0,0],3221225472,3,3),Observation{kind:6,payload:0,words:[4099,0,12289,512],calls:2,zero:2},"case 188: new-tables-1");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[0,8193,12289,512],[4096,8192,0],3221225472,3,3),Observation{kind:6,payload:0,words:[4099,8195,0,512],calls:3,zero:6},"case 189: new-tables-2");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[0,8193,12289,512],[4096,8192,12288],3221225472,3,3),Observation{kind:0,payload:3221225472,words:[4099,8195,12291,3221225475],calls:3,zero:14},"case 190: new-tables-3");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[0,8193,12289,512],[4096,8192,12288],3221225472,3,0),Observation{kind:7,payload:0,words:[4096,8193,12289,512],calls:1,zero:0},"case 191: new-parent-flags-0");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[0,8193,12289,512],[4096,8192,12288],3221225472,3,128),Observation{kind:5,payload:0,words:[0,8193,12289,512],calls:1,zero:0},"case 192: new-parent-flags-128");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[0,8193,12289,512],[4096,8192,12288],3221225472,3,129),Observation{kind:5,payload:0,words:[0,8193,12289,512],calls:1,zero:0},"case 193: new-parent-flags-129");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[0,8193,12289,512],[4096,8192,12288],3221225472,3,7),Observation{kind:0,payload:3221225472,words:[4103,8199,12295,3221225475],calls:3,zero:14},"case 194: new-parent-flags-7");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[4097,0,12289,512],[0,0,0],3221225472,3,3),Observation{kind:6,payload:0,words:[4099,0,12289,512],calls:1,zero:0},"case 195: new-suffix-1-0");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[4097,0,12289,512],[8192,0,0],3221225472,3,3),Observation{kind:6,payload:0,words:[4099,8195,0,512],calls:2,zero:4},"case 196: new-suffix-1-1");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[4097,0,12289,512],[8192,12288,0],3221225472,3,3),Observation{kind:0,payload:3221225472,words:[4099,8195,12291,3221225475],calls:2,zero:12},"case 197: new-suffix-1-2");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[4097,8193,0,512],[0,0,0],3221225472,3,3),Observation{kind:6,payload:0,words:[4099,8195,0,512],calls:1,zero:0},"case 198: new-suffix-2-0");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[4097,8193,0,512],[12288,0,0],3221225472,3,3),Observation{kind:0,payload:3221225472,words:[4099,8195,12291,3221225475],calls:1,zero:8},"case 199: new-suffix-2-1");
assert_eq!(observe::<Size4KiB>("map",4,18446623141389139968,[4097,8193,12289,0],[0,0,0],3221225472,128,3),Observation{kind:5,payload:0,words:[4099,8195,12291,0],calls:0,zero:0},"case 200: leaf-huge-flag");
assert_eq!(observe::<Size4KiB>("update",4,18446623141389139968,[4097,8193,12289,2147483649],[0,0,0],3221225472,18446744073709551615,3),Observation{kind:0,payload:2147483648,words:[4097,8193,12289,18446744073709551615],calls:0,zero:0},"case 201: all-leaf-flags");
assert_eq!(observe::<Size4KiB>("parent",2,18446623141389139968,[4097,8193,12289,2147483649],[0,0,0],3221225472,0,3),Observation{kind:0,payload:12288,words:[4097,8193,12288,2147483649],calls:0,zero:0},"case 202: parent-clear-2");
assert_eq!(observe::<Size4KiB>("parent",2,18446623141389139968,[4097,8193,0,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,8193,0,0],calls:0,zero:0},"case 203: parent-unused-2");
assert_eq!(observe::<Size4KiB>("parent",3,18446623141389139968,[4097,8193,12289,2147483649],[0,0,0],3221225472,0,3),Observation{kind:0,payload:8192,words:[4097,8192,12289,2147483649],calls:0,zero:0},"case 204: parent-clear-3");
assert_eq!(observe::<Size4KiB>("parent",3,18446623141389139968,[4097,0,12289,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[4097,0,12289,0],calls:0,zero:0},"case 205: parent-unused-3");
assert_eq!(observe::<Size4KiB>("parent",4,18446623141389139968,[4097,8193,12289,2147483649],[0,0,0],3221225472,0,3),Observation{kind:0,payload:4096,words:[4096,8193,12289,2147483649],calls:0,zero:0},"case 206: parent-clear-4");
assert_eq!(observe::<Size4KiB>("parent",4,18446623141389139968,[0,8193,12289,0],[0,0,0],3221225472,3,3),Observation{kind:1,payload:0,words:[0,8193,12289,0],calls:0,zero:0},"case 207: parent-unused-4");
 println!("PASS actual pinned full mapper routes on detached owned tables; no hardware or flush execution");
}
