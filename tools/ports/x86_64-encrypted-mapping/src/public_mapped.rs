// SPDX-License-Identifier: MIT OR Apache-2.0
use std::cell::UnsafeCell;
use x86_64::{PhysAddr,VirtAddr};
use x86_64::structures::paging::{PageTable,Page,PageSize,PhysFrame,Size4KiB,PageTableFlags,FrameAllocator};
use x86_64::structures::paging::mapper::{MappedPageTable,PageTableFrameMapping,Mapper,MapToError,UnmapError,FlagUpdateError,TranslateError};

pub struct Captures([Box<UnsafeCell<PageTable>>;3]);
// Test-only stable, distinct owned allocations; no active translation roots,
// concurrent references, cycles or aliases. UnsafeCell supports mapper writes.
unsafe impl PageTableFrameMapping for Captures {
 fn frame_to_pointer(&self,frame:PhysFrame)->*mut PageTable {
  let id=frame.start_address().as_u64();assert!([65536,131072,196608].contains(&id));self.0[(id/65536-1)as usize].get()
 }
}
struct Supply{frames:[u64;3],calls:usize}
// Generated fixtures supply only distinct, previously unlinked child tables.
unsafe impl FrameAllocator<Size4KiB> for Supply {
 fn allocate_frame(&mut self)->Option<PhysFrame>{let id=self.frames[self.calls];self.calls+=1;if id==0{None}else{Some(PhysFrame::from_start_address(PhysAddr::new(id)).unwrap())}}
}
fn store(table:&mut PageTable,index:usize,word:u64){table[index].set_addr(PhysAddr::zero(),PageTableFlags::from_bits_retain(word));}
fn raw(table:&PageTable,index:usize)->u64{table[index].addr().as_u64()|table[index].flags().bits()}
fn panic_text(v:Box<dyn std::any::Any+Send>)->String{if let Some(s)=v.downcast_ref::<&str>(){s.to_string()}else if let Some(s)=v.downcast_ref::<String>(){s.clone()}else{panic!("unexpected panic")}}
#[derive(Debug,PartialEq)]pub struct Observation{pub kind:u8,pub payload:u64,pub words:[u64;4],pub calls:u8,pub zero:u8}
pub fn observe<S:PageSize+std::fmt::Debug>(op:&str,level:u8,page:u64,words:[u64;4],frames:[u64;3],frame:u64,flags:u64,parent:u64)->Observation
where for<'a> MappedPageTable<'a,Captures>:Mapper<S>{
 let indices=[((page>>39)&511)as usize,((page>>30)&511)as usize,((page>>21)&511)as usize,((page>>12)&511)as usize];
 assert!(indices.iter().all(|&i|i!=511));
 let mut root=Box::new(PageTable::new());store(&mut root,indices[0],words[0]);
 let mut captures=Captures(std::array::from_fn(|_|Box::new(UnsafeCell::new(PageTable::new()))));
 for n in 0..3{store(captures.0[n].get_mut(),indices[n+1],words[n+1]);captures.0[n].get_mut()[511].set_flags(PageTableFlags::NO_EXECUTE);}
 let mut mapper=unsafe{MappedPageTable::new(&mut root,captures)};
 let page=Page::<S>::from_start_address(VirtAddr::new(page)).unwrap();let mut supply=Supply{frames,calls:0};
 let f=PageTableFlags::from_bits_retain(flags);let pf=PageTableFlags::from_bits_retain(parent);

 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||match op{
  "map"=>match unsafe{mapper.map_to_with_table_flags(page,PhysFrame::<S>::from_start_address(PhysAddr::new(frame)).unwrap(),f,pf,&mut supply)}{
   Ok(flush)=>{flush.ignore();(0,frame)},Err(MapToError::FrameAllocationFailed)=>(6,0),Err(MapToError::ParentEntryHugePage)=>(2,0),Err(MapToError::PageAlreadyMapped(f))=>(4,f.start_address().as_u64())},
  "unmap"=>match mapper.unmap(page){Ok((f,flush))=>{flush.ignore();(0,f.start_address().as_u64())},Err(UnmapError::PageNotMapped)=>(1,0),Err(UnmapError::ParentEntryHugePage)=>(2,0),Err(UnmapError::InvalidFrameAddress(a))=>(3,a.as_u64())},
  "translate"=>match mapper.translate_page(page){Ok(f)=>(0,f.start_address().as_u64()),Err(TranslateError::PageNotMapped)=>(1,0),Err(TranslateError::ParentEntryHugePage)=>(2,0),Err(TranslateError::InvalidFrameAddress(a))=>(3,a.as_u64())},
  "update"=>match unsafe{mapper.update_flags(page,f)}{Ok(flush)=>{flush.ignore();(0,0)},Err(FlagUpdateError::PageNotMapped)=>(1,0),Err(FlagUpdateError::ParentEntryHugePage)=>(2,0)},
  "parent"=>{let result=unsafe{match level{4=>mapper.set_flags_p4_entry(page,f),3=>mapper.set_flags_p3_entry(page,f),2=>mapper.set_flags_p2_entry(page,f),_=>panic!("bad level")}};match result{Ok(flush)=>{flush.ignore();(0,0)},Err(FlagUpdateError::PageNotMapped)=>(1,0),Err(FlagUpdateError::ParentEntryHugePage)=>(2,0)}},
  _=>panic!("bad operation")
 }));
 let (kind,payload)=match result{Ok(v)=>v,Err(v)=>{let s=panic_text(v);if s.contains("entry should be mapped"){(7,0)}else{assert!(s.contains("HUGE_PAGE"),"{s}");(5,0)}}};
 let mut observed=[raw(mapper.level_4_table(),indices[0]),0,0,0];let mut zero=0;
 for n in 0..3{let t=unsafe{&*mapper.page_table_frame_mapping().0[n].get()};observed[n+1]=raw(t,indices[n+1]);if t[511].is_unused(){zero|=1<<(n+1);}}
 Observation{kind,payload,words:observed,calls:supply.calls as u8,zero}
}
