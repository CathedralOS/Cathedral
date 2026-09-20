// SPDX-License-Identifier: MIT OR Apache-2.0
use x86_64::{PhysAddr,VirtAddr};
use x86_64::structures::paging::{PageTable,PhysFrame,PageTableFlags};
use x86_64::structures::paging::mapper::{MappedPageTable,Translate,TranslateResult,PageTableFrameMapping};
struct Captures { p3: Box<PageTable>, p2: Box<PageTable>, p1: Box<PageTable> }
// The registry owns three stable, aligned, initialized allocations for the
// entire mapper lifetime. Every reachable non-leaf ID is one of these IDs;
// this test calls only immutable translate, never live mapping mutation.
unsafe impl PageTableFrameMapping for Captures {
 fn frame_to_pointer(&self,frame:PhysFrame)->*mut PageTable {
  let table=&**match frame.start_address().as_u64() { 0x1000=>&self.p3,0x2000=>&self.p2,0x3000=>&self.p1,_=>panic!("unregistered captured table") };
  table as *const PageTable as *mut PageTable
 }
}
fn store(table:&mut PageTable,index:usize,word:u64) {
 table[index].set_addr(PhysAddr::new(word&0x000ffffffffff000),PageTableFlags::from_bits_retain(word&0xfff0000000000fff));
}
fn translate(va:u64,words:[u64;4])->Option<(u64,u64,u64,u64)> {
 let va=VirtAddr::new(va);let mut root=Box::new(PageTable::new());
 let mut captures=Captures{p3:Box::new(PageTable::new()),p2:Box::new(PageTable::new()),p1:Box::new(PageTable::new())};
 store(&mut root,usize::from(va.p4_index()),words[0]);store(&mut captures.p3,usize::from(va.p3_index()),words[1]);
 store(&mut captures.p2,usize::from(va.p2_index()),words[2]);store(&mut captures.p1,usize::from(va.p1_index()),words[3]);
 // Registry and root remain alive and unique throughout the read-only call.
 let mapper=unsafe{MappedPageTable::new(&mut root,captures)};
 match mapper.translate(va){TranslateResult::Mapped{frame,offset,flags}=>Some((frame.start_address().as_u64(),frame.size(),offset,flags.bits())),TranslateResult::NotMapped=>None,TranslateResult::InvalidFrameAddress(_)=>panic!("unexpected invalid masked frame")}
}
fn main(){std::panic::set_hook(Box::new(|_|{}));
 let mut table=PageTable::new();assert!(table.is_empty());
 store(&mut table,0,0x12345003);store(&mut table,511,0x8000000000000000);
 assert_eq!(table[0].addr().as_u64()|table[0].flags().bits(),0x12345003);
 assert_eq!(table[511].flags().bits(),0x8000000000000000);assert!(!table.is_empty());
 table[0].set_unused();assert!(!table.is_empty());
 for entry in table.iter_mut(){entry.set_flags(PageTableFlags::from_bits_retain(u64::MAX));}
 assert_eq!(table.iter().filter(|entry|!entry.is_unused()).count(),512);
 table.zero();assert!(table.is_empty());assert!(table.iter().all(|entry|entry.is_unused()));
assert_eq!(translate(0, [0, 0, 0, 0]),None,"case0");
assert_eq!(translate(0, [128, 0, 0, 0]),None,"case1");
assert!(std::panic::catch_unwind(||translate(0, [4225, 0, 0, 0])).is_err(),"case2");
assert_eq!(translate(0, [4097, 0, 0, 0]),None,"case3");
assert_eq!(translate(0, [4097, 128, 0, 0]),None,"case4");
assert_eq!(translate(0, [4097, 8193, 0, 0]),None,"case5");
assert_eq!(translate(0, [4097, 8193, 12289, 0]),None,"case6");
assert_eq!(translate(0, [4097, 8193, 12289, 512]),Some((0,4096,0,512)),"case7");
assert_eq!(translate(0, [4097, 8193, 12289, 3405643779]),Some((3405643776,4096,0,3)),"case8");
assert_eq!(translate(0, [4097, 8193, 12289, 3405643905]),Some((3405643776,4096,0,129)),"case9");
assert_eq!(translate(0, [4097, 1073746049, 0, 0]),Some((1073741824,1073741824,0,129)),"case10");
assert_eq!(translate(0, [4097, 8193, 1075843201, 0]),Some((1075838976,2097152,0,129)),"case11");
assert_eq!(translate(0, [4097, 18446744073709551615, 0, 0]),Some((4503598553628672,1073741824,0,18442240474082185215)),"case12");
assert_eq!(translate(0, [4097, 8193, 18446744073709551615, 0]),Some((4503599625273344,2097152,0,18442240474082185215)),"case13");
assert_eq!(translate(3735928559, [0, 0, 0, 0]),None,"case14");
assert_eq!(translate(3735928559, [128, 0, 0, 0]),None,"case15");
assert!(std::panic::catch_unwind(||translate(3735928559, [4225, 0, 0, 0])).is_err(),"case16");
assert_eq!(translate(3735928559, [4097, 0, 0, 0]),None,"case17");
assert_eq!(translate(3735928559, [4097, 128, 0, 0]),None,"case18");
assert_eq!(translate(3735928559, [4097, 8193, 0, 0]),None,"case19");
assert_eq!(translate(3735928559, [4097, 8193, 12289, 0]),None,"case20");
assert_eq!(translate(3735928559, [4097, 8193, 12289, 512]),Some((0,4096,3823,512)),"case21");
assert_eq!(translate(3735928559, [4097, 8193, 12289, 3405643779]),Some((3405643776,4096,3823,3)),"case22");
assert_eq!(translate(3735928559, [4097, 8193, 12289, 3405643905]),Some((3405643776,4096,3823,129)),"case23");
assert_eq!(translate(3735928559, [4097, 1073746049, 0, 0]),Some((1073741824,1073741824,514703087,129)),"case24");
assert_eq!(translate(3735928559, [4097, 8193, 1075843201, 0]),Some((1075838976,2097152,900847,129)),"case25");
assert_eq!(translate(3735928559, [4097, 18446744073709551615, 0, 0]),Some((4503598553628672,1073741824,514703087,18442240474082185215)),"case26");
assert_eq!(translate(3735928559, [4097, 8193, 18446744073709551615, 0]),Some((4503599625273344,2097152,900847,18442240474082185215)),"case27");
assert_eq!(translate(18446623352219540156, [0, 0, 0, 0]),None,"case28");
assert_eq!(translate(18446623352219540156, [128, 0, 0, 0]),None,"case29");
assert!(std::panic::catch_unwind(||translate(18446623352219540156, [4225, 0, 0, 0])).is_err(),"case30");
assert_eq!(translate(18446623352219540156, [4097, 0, 0, 0]),None,"case31");
assert_eq!(translate(18446623352219540156, [4097, 128, 0, 0]),None,"case32");
assert_eq!(translate(18446623352219540156, [4097, 8193, 0, 0]),None,"case33");
assert_eq!(translate(18446623352219540156, [4097, 8193, 12289, 0]),None,"case34");
assert_eq!(translate(18446623352219540156, [4097, 8193, 12289, 512]),Some((0,4096,2748,512)),"case35");
assert_eq!(translate(18446623352219540156, [4097, 8193, 12289, 3405643779]),Some((3405643776,4096,2748,3)),"case36");
assert_eq!(translate(18446623352219540156, [4097, 8193, 12289, 3405643905]),Some((3405643776,4096,2748,129)),"case37");
assert_eq!(translate(18446623352219540156, [4097, 1073746049, 0, 0]),Some((1073741824,1073741824,377002684,129)),"case38");
assert_eq!(translate(18446623352219540156, [4097, 8193, 1075843201, 0]),Some((1075838976,2097152,1612476,129)),"case39");
assert_eq!(translate(18446623352219540156, [4097, 18446744073709551615, 0, 0]),Some((4503598553628672,1073741824,377002684,18442240474082185215)),"case40");
assert_eq!(translate(18446623352219540156, [4097, 8193, 18446744073709551615, 0]),Some((4503599625273344,2097152,1612476,18442240474082185215)),"case41");
println!("42 actual pinned translations and complete512entry table operations passed");
}
