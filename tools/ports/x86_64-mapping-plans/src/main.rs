// SPDX-License-Identifier: MIT OR Apache-2.0
use std::cell::UnsafeCell;
use x86_64::{PhysAddr,VirtAddr};
use x86_64::structures::paging::{PageTable,Page,PageSize,PhysFrame,Size4KiB,Size2MiB,Size1GiB,PageTableFlags,FrameAllocator};
use x86_64::structures::paging::mapper::{MappedPageTable,PageTableFrameMapping,Mapper,MapToError,UnmapError,FlagUpdateError,TranslateError};
const MASK:u64=0x000ffffffffff000;
struct Captures { p3:Box<UnsafeCell<PageTable>>,p2:Box<UnsafeCell<PageTable>>,p1:Box<UnsafeCell<PageTable>> }
impl Captures {
 fn new()->Self { Self{p3:Box::new(UnsafeCell::new(PageTable::new())),p2:Box::new(UnsafeCell::new(PageTable::new())),p1:Box::new(UnsafeCell::new(PageTable::new()))} }
 fn slot(&self,size:u64)->&PageTable { unsafe{&*match size {4096=>self.p1.get(),2097152=>self.p2.get(),_=>self.p3.get()}} }
}
// Owned stable allocations, distinct registered IDs, no cycles or simultaneous
// table references. UnsafeCell provides the interior mutation the mapper requires.
unsafe impl PageTableFrameMapping for Captures {
 fn frame_to_pointer(&self,frame:PhysFrame)->*mut PageTable {match frame.start_address().as_u64(){4096=>self.p3.get(),8192=>self.p2.get(),12288=>self.p1.get(),_=>panic!("unregistered frame")}}
}
struct Supply(Option<u64>);
// At most one previously unlinked captured frame is supplied in these fixtures.
unsafe impl FrameAllocator<Size4KiB> for Supply { fn allocate_frame(&mut self)->Option<PhysFrame>{self.0.take().map(|v|PhysFrame::from_start_address(PhysAddr::new(v)).unwrap())} }
fn store(table:&mut PageTable,word:u64){table[0].set_addr(PhysAddr::new(word&MASK),PageTableFlags::from_bits_retain(word&!MASK));}
fn raw(table:&PageTable)->u64{table[0].addr().as_u64()|table[0].flags().bits()}
fn panic_text(value:Box<dyn std::any::Any+Send>)->String{if let Some(s)=value.downcast_ref::<&str>(){s.to_string()}else if let Some(s)=value.downcast_ref::<String>(){s.clone()}else{panic!("unexpected panic payload")}}
fn leaf<S:PageSize+std::fmt::Debug>(op:&str,word:u64,frame:u64,flags:u64)->(u8,u64,u64)
where for<'a> MappedPageTable<'a,Captures>:Mapper<S> {
 let mut root=Box::new(PageTable::new());store(&mut root,4097);let mut captures=Captures::new();
 store(captures.p3.get_mut(),if S::SIZE==1073741824{word}else{8193});
 store(captures.p2.get_mut(),if S::SIZE==2097152{word}else{12289});store(captures.p1.get_mut(),word);
 let mut mapper=unsafe{MappedPageTable::new(&mut root,captures)};
 let page=Page::<S>::from_start_address(VirtAddr::new(0)).unwrap();let flags=PageTableFlags::from_bits_retain(flags);
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||match op {
  "map"=>match unsafe{mapper.map_to_with_table_flags(page,PhysFrame::<S>::from_start_address(PhysAddr::new(frame)).unwrap(),flags,PageTableFlags::empty(),&mut Supply(None))}{Ok(flush)=>{flush.ignore();(0,frame)},Err(MapToError::PageAlreadyMapped(frame))=>(4,frame.start_address().as_u64()),other=>panic!("unexpected map result {other:?}")},
  "unmap"=>match mapper.unmap(page){Ok((frame,flush))=>{flush.ignore();(0,frame.start_address().as_u64())},Err(UnmapError::PageNotMapped)=>(1,0),Err(UnmapError::ParentEntryHugePage)=>(2,0),Err(UnmapError::InvalidFrameAddress(a))=>(3,a.as_u64())},
  "update"=>match unsafe{mapper.update_flags(page,flags)}{Ok(flush)=>{flush.ignore();(0,word&MASK)},Err(FlagUpdateError::PageNotMapped)=>(1,0),Err(FlagUpdateError::ParentEntryHugePage)=>(2,0)},
  "translate"=>match mapper.translate_page(page){Ok(frame)=>(0,frame.start_address().as_u64()),Err(TranslateError::PageNotMapped)=>(1,0),Err(TranslateError::ParentEntryHugePage)=>(2,0),Err(TranslateError::InvalidFrameAddress(a))=>(3,a.as_u64())},_=>panic!("unknown op")
 }));
 let (kind,payload)=match result{Ok(value)=>value,Err(error)=>{assert!(panic_text(error).contains("HUGE_PAGE"));(5,0)}};
 (kind,raw(mapper.page_table_frame_mapping().slot(S::SIZE)),payload)
}
fn child(word:u64,flags:u64,supply:Option<u64>)->(u8,u64,bool){
 let mut root=Box::new(PageTable::new());store(&mut root,word);let mut captures=Captures::new();
 // A nonzero last slot proves a newly supplied child was actually cleared.
 captures.p3.get_mut()[511].set_flags(PageTableFlags::NO_EXECUTE);
 let mut mapper=unsafe{MappedPageTable::new(&mut root,captures)};
 let page=Page::<Size1GiB>::from_start_address(VirtAddr::new(0)).unwrap();let frame=PhysFrame::<Size1GiB>::from_start_address(PhysAddr::new(0x80000000)).unwrap();
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||unsafe{mapper.map_to_with_table_flags(page,frame,PageTableFlags::PRESENT,PageTableFlags::from_bits_retain(flags),&mut Supply(supply))}));
 let kind=match result{Ok(Ok(flush))=>{flush.ignore();0},Ok(Err(MapToError::FrameAllocationFailed))=>1,Ok(Err(MapToError::ParentEntryHugePage))=>3,Ok(Err(other))=>panic!("unexpected child {other:?}"),Err(error)=>{let text=panic_text(error);if text.contains("entry should be mapped"){2}else{assert!(text.contains("HUGE_PAGE"));4}}};
 let cleared=kind==0 && word==0 && unsafe{(&*mapper.page_table_frame_mapping().p3.get())[511].is_unused()};
 (kind,raw(mapper.level_4_table()),cleared)
}
fn main(){std::panic::set_hook(Box::new(|_|{}));
 assert_eq!(leaf::<Size4KiB>("unmap",0,2147483648,9223372036854775811),(1,0,0));
assert_eq!(leaf::<Size4KiB>("update",0,2147483648,9223372036854775811),(1,0,0));
assert_eq!(leaf::<Size4KiB>("translate",0,2147483648,9223372036854775811),(1,0,0));
assert_eq!(leaf::<Size4KiB>("unmap",128,2147483648,9223372036854775811),(1,128,0));
assert_eq!(leaf::<Size4KiB>("update",128,2147483648,9223372036854775811),(0,9223372036854775811,0));
assert_eq!(leaf::<Size4KiB>("translate",128,2147483648,9223372036854775811),(0,128,0));
assert_eq!(leaf::<Size4KiB>("unmap",512,2147483648,9223372036854775811),(1,512,0));
assert_eq!(leaf::<Size4KiB>("update",512,2147483648,9223372036854775811),(0,9223372036854775811,0));
assert_eq!(leaf::<Size4KiB>("translate",512,2147483648,9223372036854775811),(0,512,0));
assert_eq!(leaf::<Size4KiB>("unmap",1,2147483648,9223372036854775811),(0,0,0));
assert_eq!(leaf::<Size4KiB>("update",1,2147483648,9223372036854775811),(0,9223372036854775811,0));
assert_eq!(leaf::<Size4KiB>("translate",1,2147483648,9223372036854775811),(0,1,0));
assert_eq!(leaf::<Size4KiB>("unmap",129,2147483648,9223372036854775811),(2,129,0));
assert_eq!(leaf::<Size4KiB>("update",129,2147483648,9223372036854775811),(0,9223372036854775811,0));
assert_eq!(leaf::<Size4KiB>("translate",129,2147483648,9223372036854775811),(0,129,0));
assert_eq!(leaf::<Size4KiB>("unmap",1073741825,2147483648,9223372036854775811),(0,0,1073741824));
assert_eq!(leaf::<Size4KiB>("update",1073741825,2147483648,9223372036854775811),(0,9223372037928517635,1073741824));
assert_eq!(leaf::<Size4KiB>("translate",1073741825,2147483648,9223372036854775811),(0,1073741825,1073741824));
assert_eq!(leaf::<Size4KiB>("unmap",1073741953,2147483648,9223372036854775811),(2,1073741953,0));
assert_eq!(leaf::<Size4KiB>("update",1073741953,2147483648,9223372036854775811),(0,9223372037928517635,1073741824));
assert_eq!(leaf::<Size4KiB>("translate",1073741953,2147483648,9223372036854775811),(0,1073741953,1073741824));
assert_eq!(leaf::<Size4KiB>("unmap",1073746049,2147483648,9223372036854775811),(2,1073746049,0));
assert_eq!(leaf::<Size4KiB>("update",1073746049,2147483648,9223372036854775811),(0,9223372037928521731,1073745920));
assert_eq!(leaf::<Size4KiB>("translate",1073746049,2147483648,9223372036854775811),(0,1073746049,1073745920));
assert_eq!(leaf::<Size4KiB>("map",0,2147483648,3),(0,2147483651,2147483648));
assert_eq!(leaf::<Size4KiB>("map",0,2147483648,128),(5,0,0));
assert_eq!(leaf::<Size4KiB>("map",0,2147483648,0),(0,2147483648,2147483648));
assert_eq!(leaf::<Size4KiB>("map",512,2147483648,3),(4,512,2147483648));
assert_eq!(leaf::<Size4KiB>("map",1,2147483648,3),(4,1,2147483648));
assert_eq!(leaf::<Size4KiB>("map",1,2147483648,128),(4,1,2147483648));
assert_eq!(leaf::<Size4KiB>("update",1073741825,2147483648,0),(0,1073741824,1073741824));
assert_eq!(leaf::<Size4KiB>("update",1073741825,2147483648,8195),(0,1073750019,1073741824));
assert_eq!(leaf::<Size4KiB>("update",1073741825,2147483648,18446744073709551615),(0,18446744073709551615,1073741824));
assert_eq!(leaf::<Size2MiB>("unmap",0,2147483648,9223372036854775811),(1,0,0));
assert_eq!(leaf::<Size2MiB>("update",0,2147483648,9223372036854775811),(1,0,0));
assert_eq!(leaf::<Size2MiB>("translate",0,2147483648,9223372036854775811),(1,0,0));
assert_eq!(leaf::<Size2MiB>("unmap",128,2147483648,9223372036854775811),(1,128,0));
assert_eq!(leaf::<Size2MiB>("update",128,2147483648,9223372036854775811),(0,9223372036854775939,0));
assert_eq!(leaf::<Size2MiB>("translate",128,2147483648,9223372036854775811),(0,128,0));
assert_eq!(leaf::<Size2MiB>("unmap",512,2147483648,9223372036854775811),(1,512,0));
assert_eq!(leaf::<Size2MiB>("update",512,2147483648,9223372036854775811),(0,9223372036854775939,0));
assert_eq!(leaf::<Size2MiB>("translate",512,2147483648,9223372036854775811),(0,512,0));
assert_eq!(leaf::<Size2MiB>("unmap",1,2147483648,9223372036854775811),(2,1,0));
assert_eq!(leaf::<Size2MiB>("update",1,2147483648,9223372036854775811),(0,9223372036854775939,0));
assert_eq!(leaf::<Size2MiB>("translate",1,2147483648,9223372036854775811),(0,1,0));
assert_eq!(leaf::<Size2MiB>("unmap",129,2147483648,9223372036854775811),(0,0,0));
assert_eq!(leaf::<Size2MiB>("update",129,2147483648,9223372036854775811),(0,9223372036854775939,0));
assert_eq!(leaf::<Size2MiB>("translate",129,2147483648,9223372036854775811),(0,129,0));
assert_eq!(leaf::<Size2MiB>("unmap",1073741825,2147483648,9223372036854775811),(2,1073741825,0));
assert_eq!(leaf::<Size2MiB>("update",1073741825,2147483648,9223372036854775811),(0,9223372037928517763,1073741824));
assert_eq!(leaf::<Size2MiB>("translate",1073741825,2147483648,9223372036854775811),(0,1073741825,1073741824));
assert_eq!(leaf::<Size2MiB>("unmap",1073741953,2147483648,9223372036854775811),(0,0,1073741824));
assert_eq!(leaf::<Size2MiB>("update",1073741953,2147483648,9223372036854775811),(0,9223372037928517763,1073741824));
assert_eq!(leaf::<Size2MiB>("translate",1073741953,2147483648,9223372036854775811),(0,1073741953,1073741824));
assert_eq!(leaf::<Size2MiB>("unmap",1073746049,2147483648,9223372036854775811),(3,1073746049,1073745920));
assert_eq!(leaf::<Size2MiB>("update",1073746049,2147483648,9223372036854775811),(0,9223372037928521859,1073745920));
assert_eq!(leaf::<Size2MiB>("translate",1073746049,2147483648,9223372036854775811),(3,1073746049,1073745920));
assert_eq!(leaf::<Size2MiB>("map",0,2147483648,3),(0,2147483779,2147483648));
assert_eq!(leaf::<Size2MiB>("map",0,2147483648,128),(0,2147483776,2147483648));
assert_eq!(leaf::<Size2MiB>("map",0,2147483648,0),(0,2147483776,2147483648));
assert_eq!(leaf::<Size2MiB>("map",512,2147483648,3),(4,512,2147483648));
assert_eq!(leaf::<Size2MiB>("map",1,2147483648,3),(4,1,2147483648));
assert_eq!(leaf::<Size2MiB>("map",1,2147483648,128),(4,1,2147483648));
assert_eq!(leaf::<Size2MiB>("update",1073741825,2147483648,0),(0,1073741952,1073741824));
assert_eq!(leaf::<Size2MiB>("update",1073741825,2147483648,8195),(0,1073750147,1073741824));
assert_eq!(leaf::<Size2MiB>("update",1073741825,2147483648,18446744073709551615),(0,18446744073709551615,1073741824));
assert_eq!(leaf::<Size1GiB>("unmap",0,2147483648,9223372036854775811),(1,0,0));
assert_eq!(leaf::<Size1GiB>("update",0,2147483648,9223372036854775811),(1,0,0));
assert_eq!(leaf::<Size1GiB>("translate",0,2147483648,9223372036854775811),(1,0,0));
assert_eq!(leaf::<Size1GiB>("unmap",128,2147483648,9223372036854775811),(1,128,0));
assert_eq!(leaf::<Size1GiB>("update",128,2147483648,9223372036854775811),(0,9223372036854775939,0));
assert_eq!(leaf::<Size1GiB>("translate",128,2147483648,9223372036854775811),(0,128,0));
assert_eq!(leaf::<Size1GiB>("unmap",512,2147483648,9223372036854775811),(1,512,0));
assert_eq!(leaf::<Size1GiB>("update",512,2147483648,9223372036854775811),(0,9223372036854775939,0));
assert_eq!(leaf::<Size1GiB>("translate",512,2147483648,9223372036854775811),(0,512,0));
assert_eq!(leaf::<Size1GiB>("unmap",1,2147483648,9223372036854775811),(2,1,0));
assert_eq!(leaf::<Size1GiB>("update",1,2147483648,9223372036854775811),(0,9223372036854775939,0));
assert_eq!(leaf::<Size1GiB>("translate",1,2147483648,9223372036854775811),(0,1,0));
assert_eq!(leaf::<Size1GiB>("unmap",129,2147483648,9223372036854775811),(0,0,0));
assert_eq!(leaf::<Size1GiB>("update",129,2147483648,9223372036854775811),(0,9223372036854775939,0));
assert_eq!(leaf::<Size1GiB>("translate",129,2147483648,9223372036854775811),(0,129,0));
assert_eq!(leaf::<Size1GiB>("unmap",1073741825,2147483648,9223372036854775811),(2,1073741825,0));
assert_eq!(leaf::<Size1GiB>("update",1073741825,2147483648,9223372036854775811),(0,9223372037928517763,1073741824));
assert_eq!(leaf::<Size1GiB>("translate",1073741825,2147483648,9223372036854775811),(0,1073741825,1073741824));
assert_eq!(leaf::<Size1GiB>("unmap",1073741953,2147483648,9223372036854775811),(0,0,1073741824));
assert_eq!(leaf::<Size1GiB>("update",1073741953,2147483648,9223372036854775811),(0,9223372037928517763,1073741824));
assert_eq!(leaf::<Size1GiB>("translate",1073741953,2147483648,9223372036854775811),(0,1073741953,1073741824));
assert_eq!(leaf::<Size1GiB>("unmap",1073746049,2147483648,9223372036854775811),(3,1073746049,1073745920));
assert_eq!(leaf::<Size1GiB>("update",1073746049,2147483648,9223372036854775811),(0,9223372037928521859,1073745920));
assert_eq!(leaf::<Size1GiB>("translate",1073746049,2147483648,9223372036854775811),(3,1073746049,1073745920));
assert_eq!(leaf::<Size1GiB>("map",0,2147483648,3),(0,2147483779,2147483648));
assert_eq!(leaf::<Size1GiB>("map",0,2147483648,128),(0,2147483776,2147483648));
assert_eq!(leaf::<Size1GiB>("map",0,2147483648,0),(0,2147483776,2147483648));
assert_eq!(leaf::<Size1GiB>("map",512,2147483648,3),(4,512,2147483648));
assert_eq!(leaf::<Size1GiB>("map",1,2147483648,3),(4,1,2147483648));
assert_eq!(leaf::<Size1GiB>("map",1,2147483648,128),(4,1,2147483648));
assert_eq!(leaf::<Size1GiB>("update",1073741825,2147483648,0),(0,1073741952,1073741824));
assert_eq!(leaf::<Size1GiB>("update",1073741825,2147483648,8195),(0,1073750147,1073741824));
assert_eq!(leaf::<Size1GiB>("update",1073741825,2147483648,18446744073709551615),(0,18446744073709551615,1073741824));
assert_eq!(child(0,3,Some(4096)),(0,4099,true));
assert_eq!(child(0,3,None),(1,0,false));
assert_eq!(child(0,0,Some(4096)),(2,4096,false));
assert_eq!(child(0,128,Some(4096)),(4,0,false));
assert_eq!(child(4097,0,None),(0,4097,false));
assert_eq!(child(4097,3,None),(0,4099,false));
assert_eq!(child(4097,129,None),(3,4225,false));
assert_eq!(child(4096,0,None),(2,4096,false));
assert_eq!(child(4096,1,None),(0,4097,false));
assert_eq!(child(4225,2,None),(3,4227,false));
assert_eq!(child(4097,8192,None),(0,12289,false));
assert_eq!(child(0,128,None),(1,0,false));
assert_eq!(child(4097,0,Some(1)),(0,4097,false));
 println!("Actual pinned leaf operations and parent-creation effects passed; no live mappings or flush instructions");
}
