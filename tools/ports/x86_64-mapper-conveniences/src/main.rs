// SPDX-License-Identifier: MIT OR Apache-2.0
use x86_64::{PhysAddr,VirtAddr};
use x86_64::structures::paging::{Page,PageSize,PhysFrame,Size4KiB,Size2MiB,Size1GiB,PageTableFlags,FrameAllocator};
use x86_64::structures::paging::mapper::{Mapper,MapperFlush,MapperFlushAll,MapToError,UnmapError,FlagUpdateError,TranslateError,Translate,TranslateResult,MappedFrame};
#[derive(Default)]struct Recorder{seen:Option<(u64,u64,u64,u64,u64)>}
struct NoAlloc;
unsafe impl FrameAllocator<Size4KiB> for NoAlloc{fn allocate_frame(&mut self)->Option<PhysFrame>{panic!("default wrapper must only delegate")}}
impl<S:PageSize> Mapper<S> for Recorder{
 unsafe fn map_to_with_table_flags<A:FrameAllocator<Size4KiB>+?Sized>(&mut self,page:Page<S>,frame:PhysFrame<S>,flags:PageTableFlags,parent:PageTableFlags,_:&mut A)->Result<MapperFlush<S>,MapToError<S>>{
  self.seen=Some((page.start_address().as_u64(),frame.start_address().as_u64(),S::SIZE,flags.bits(),parent.bits()));Err(MapToError::FrameAllocationFailed)
 }
 fn unmap(&mut self,_:Page<S>)->Result<(PhysFrame<S>,MapperFlush<S>),UnmapError>{panic!("unexpected operation")}
 unsafe fn update_flags(&mut self,_:Page<S>,_:PageTableFlags)->Result<MapperFlush<S>,FlagUpdateError>{panic!("unexpected operation")}
 unsafe fn set_flags_p4_entry(&mut self,_:Page<S>,_:PageTableFlags)->Result<MapperFlushAll,FlagUpdateError>{panic!("unexpected operation")}
 unsafe fn set_flags_p3_entry(&mut self,_:Page<S>,_:PageTableFlags)->Result<MapperFlushAll,FlagUpdateError>{panic!("unexpected operation")}
 unsafe fn set_flags_p2_entry(&mut self,_:Page<S>,_:PageTableFlags)->Result<MapperFlushAll,FlagUpdateError>{panic!("unexpected operation")}
 fn translate_page(&self,_:Page<S>)->Result<PhysFrame<S>,TranslateError>{panic!("unexpected operation")}
}
fn defaults<S:PageSize>(){
 for flags in [0,1,2,4,7,0x80,0x100,0x12345007,0x8000000000000007]{
  let page=Page::<S>::from_start_address(VirtAddr::new(2*S::SIZE)).unwrap();let frame=PhysFrame::<S>::from_start_address(PhysAddr::new(3*S::SIZE)).unwrap();
  let mut recorder=Recorder::default();let mut supply=NoAlloc;
  assert!(matches!(unsafe{recorder.map_to(page,frame,PageTableFlags::from_bits_retain(flags),&mut supply)},Err(MapToError::FrameAllocationFailed)));
  assert_eq!(recorder.seen,Some((2*S::SIZE,3*S::SIZE,S::SIZE,flags,flags&7)));
  recorder.seen=None;
  assert!(matches!(unsafe{recorder.identity_map(frame,PageTableFlags::from_bits_retain(flags),&mut supply)},Err(MapToError::FrameAllocationFailed)));
  assert_eq!(recorder.seen,Some((3*S::SIZE,3*S::SIZE,S::SIZE,flags,flags&7)));
 }
 let mut recorder=Recorder::default();let mut supply=NoAlloc;
 let frame=PhysFrame::<S>::from_start_address(PhysAddr::new(0x800000000000)).unwrap();
 assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||unsafe{recorder.identity_map(frame,PageTableFlags::PRESENT,&mut supply)})).is_err());
 assert!(recorder.seen.is_none());
}
struct TranslationSource{frame:u64,size:u64,offset:u64,kind:u8}
fn frame(address:u64,size:u64)->MappedFrame{
 let address=PhysAddr::new(address);
 match size{4096=>MappedFrame::Size4KiB(PhysFrame::from_start_address(address).unwrap()),2097152=>MappedFrame::Size2MiB(PhysFrame::from_start_address(address).unwrap()),1073741824=>MappedFrame::Size1GiB(PhysFrame::from_start_address(address).unwrap()),_=>panic!("invalid size")}
}
impl Translate for TranslationSource{
 fn translate(&self,_:VirtAddr)->TranslateResult{
  match self.kind{0=>TranslateResult::Mapped{frame:frame(self.frame,self.size),offset:self.offset,flags:PageTableFlags::NO_EXECUTE},1=>TranslateResult::NotMapped,_=>TranslateResult::InvalidFrameAddress(PhysAddr::new(self.frame))}
 }
}
fn translated(address:u64,size:u64,offset:u64)->Option<u64>{TranslationSource{frame:address,size,offset,kind:0}.translate_addr(VirtAddr::zero()).map(|v|v.as_u64())}
mod route_reference;
fn main(){
 std::panic::set_hook(Box::new(|_|{}));defaults::<Size4KiB>();defaults::<Size2MiB>();defaults::<Size1GiB>();
 for size in [4096,2097152,1073741824]{
  let f=frame(size*2,size);assert_eq!(f.start_address().as_u64(),size*2);assert_eq!(f.size(),size);
  for offset in [0,size-1,size,size+1]{assert_eq!(translated(size*2,size,offset),Some(size*2+offset));}
 }
 for kind in [1,2]{assert!(TranslationSource{frame:4096,size:4096,offset:0,kind}.translate_addr(VirtAddr::zero()).is_none());}
 assert!(std::panic::catch_unwind(||translated(0xffffffffff000,4096,4096)).is_err());
 route_reference::verify();
 println!("PASS 54 actual Mapper default calls,3 identity pre-delegation panics,6 MappedFrame projections,15 Translate defaults and4 actual detached routes");
}
