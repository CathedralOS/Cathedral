// SPDX-License-Identifier: MIT OR Apache-2.0
use std::cell::UnsafeCell;
use x86_64::{PhysAddr,VirtAddr};
use x86_64::structures::paging::{PageTable,Page,Size4KiB,PhysFrame,PageTableFlags,FrameDeallocator};
use x86_64::structures::paging::mapper::{CleanUp,MappedPageTable,PageTableFrameMapping};
const MASK:u64=0x000ffffffffff000;
const IDS:[u64;4]=[4096,8192,12288,20480];
struct Captures(Vec<(u64,Box<UnsafeCell<PageTable>>)>);
// Stable detached owned tables, distinct IDs, acyclic generated links.
unsafe impl PageTableFrameMapping for Captures {
 fn frame_to_pointer(&self,frame:PhysFrame)->*mut PageTable {self.0.iter().find(|(id,_)|*id==frame.start_address().as_u64()).expect("missing captured frame").1.get()}
}
struct Retire(Vec<u64>);
// Keep allocations alive for final inspection; record the exact callback order.
impl FrameDeallocator<Size4KiB> for Retire {unsafe fn deallocate_frame(&mut self,frame:PhysFrame){self.0.push(frame.start_address().as_u64());}}
fn store(table:&mut PageTable,index:usize,word:u64){table[index].set_addr(PhysAddr::new(word&MASK),PageTableFlags::from_bits_retain(word&!MASK));}
fn observe(first:u64,last:u64,entries:&[(u64,usize,u64)])->(Vec<(u64,usize,u64)>,Vec<u64>){
 let mut root=Box::new(PageTable::new());let mut captures=Captures(IDS.into_iter().map(|id|(id,Box::new(UnsafeCell::new(PageTable::new())))).collect());
 for &(id,index,word)in entries{if id==16384{store(&mut root,index,word)}else{store(captures.0.iter_mut().find(|(i,_)|*i==id).unwrap().1.get_mut(),index,word)}}
 let mut mapper=unsafe{MappedPageTable::new(&mut root,captures)};let a=Page::<Size4KiB>::from_start_address(VirtAddr::new(first)).unwrap();let b=Page::<Size4KiB>::from_start_address(VirtAddr::new(last)).unwrap();
 let mut retired=Retire(vec![]);unsafe{mapper.clean_up_addr_range(Page::range_inclusive(a,b),&mut retired)};
 let mut result=vec![];
 for id in [4096,8192,12288,16384,20480]{let table=if id==16384{mapper.level_4_table()}else{unsafe{&*mapper.page_table_frame_mapping().0.iter().find(|(i,_)|*i==id).unwrap().1.get()}};for i in 0..512{let word=table[i].addr().as_u64()|table[i].flags().bits();if word!=0{result.push((id,i,word));}}}
 (result,retired.0)
}
fn main(){
 assert_eq!(observe(0,18446744073709547520,&[]),(vec![],vec![]),"empty-root");
assert_eq!(observe(0,18446744073709547520,&[(16384,0,4097),(4096,0,8193),(8192,0,12289)]),(vec![],vec![12288,8192,4096]),"empty-chain-all");
assert_eq!(observe(0,2097152,&[(16384,0,4097),(4096,0,8193),(8192,0,12289),(8192,1,20481)]),(vec![],vec![12288,20480,8192,4096]),"two-empty-leaves");
assert_eq!(observe(4096,8192,&[(16384,0,4097),(4096,0,8193),(8192,0,12289),(8192,1,20481)]),(vec![(4096,0,8193),(8192,1,20481),(16384,0,4097)],vec![12288]),"partial-first-leaf");
assert_eq!(observe(2097152,2097152,&[(16384,0,4097),(4096,0,8193),(8192,0,12289),(8192,1,20481)]),(vec![(4096,0,8193),(8192,0,12289),(16384,0,4097)],vec![20480]),"later-leaf-only");
assert_eq!(observe(0,0,&[(16384,0,4097),(4096,0,8193),(8192,0,12289),(12288,511,9223372036854775808)]),(vec![(4096,0,8193),(8192,0,12289),(12288,511,9223372036854775808),(16384,0,4097)],vec![]),"neighbor-nonpresent");
assert_eq!(observe(0,8192,&[(16384,0,4097),(4096,0,8193),(8192,0,12289),(12288,0,512)]),(vec![(4096,0,8193),(8192,0,12289),(12288,0,512),(16384,0,4097)],vec![]),"leaf-nonpresent");
assert_eq!(observe(0,2097152,&[(16384,0,4097),(4096,0,8193),(8192,0,129)]),(vec![(4096,0,8193),(8192,0,129),(16384,0,4097)],vec![]),"huge-parent");
assert_eq!(observe(0,1073741824,&[(16384,0,4097),(4096,0,8192)]),(vec![(4096,0,8192),(16384,0,4097)],vec![]),"nonpresent-parent");
assert_eq!(observe(18446603336221196288,18446603336223293440,&[(16384,256,4097),(4096,0,8193),(8192,0,12289),(8192,1,20481)]),(vec![],vec![12288,20480,8192,4096]),"high-half");
assert_eq!(observe(140737488351232,18446603336221196288,&[(16384,256,4097),(4096,0,8193),(8192,0,12289)]),(vec![],vec![12288,8192,4096]),"canonical-gap");
assert_eq!(observe(18446744073709547520,18446744073709547520,&[(16384,511,4097),(4096,511,8193),(8192,511,12289)]),(vec![],vec![12288,8192,4096]),"last-page");
assert_eq!(observe(4096,0,&[(16384,0,4097),(4096,0,8193),(8192,0,12289)]),(vec![(4096,0,8193),(8192,0,12289),(16384,0,4097)],vec![]),"reversed");
 println!("PASS actual pinned whole-range cleanup trees and retirement order");
}
