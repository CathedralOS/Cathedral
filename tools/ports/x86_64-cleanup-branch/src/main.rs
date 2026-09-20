// SPDX-License-Identifier: MIT OR Apache-2.0
use std::cell::UnsafeCell;
use x86_64::{PhysAddr,VirtAddr};
use x86_64::structures::paging::{PageTable,Page,Size4KiB,PhysFrame,PageTableFlags,FrameDeallocator};
use x86_64::structures::paging::mapper::{CleanUp,MappedPageTable,PageTableFrameMapping};
const MASK:u64=0x000ffffffffff000;
struct Captures([Box<UnsafeCell<PageTable>>;3]);
// Distinct stable owned tables; generated links are acyclic and unaliased.
unsafe impl PageTableFrameMapping for Captures {
 fn frame_to_pointer(&self,frame:PhysFrame)->*mut PageTable {let id=frame.start_address().as_u64();assert!([4096,8192,12288].contains(&id));self.0[(id/4096-1)as usize].get()}
}
struct Retire(Vec<u64>);
// Detached fixture keeps backing alive to inspect cleared links after the call.
// The callback records order; none of these frames belongs to live hardware.
impl FrameDeallocator<Size4KiB> for Retire {unsafe fn deallocate_frame(&mut self,frame:PhysFrame){self.0.push(frame.start_address().as_u64());}}
fn store(table:&mut PageTable,index:usize,word:u64){table[index].set_addr(PhysAddr::new(word&MASK),PageTableFlags::from_bits_retain(word&!MASK));}
fn raw(table:&PageTable,index:usize)->u64{table[index].addr().as_u64()|table[index].flags().bits()}
fn observe(words:[u64;4],others:u8)->([u64;4],Vec<u64>){
 let mut root=Box::new(PageTable::new());store(&mut root,0,words[0]);if others&1!=0{root[511].set_flags(PageTableFlags::NO_EXECUTE);}
 let mut captures=Captures(std::array::from_fn(|_|Box::new(UnsafeCell::new(PageTable::new()))));
 for n in 0..3{store(captures.0[n].get_mut(),0,words[n+1]);if others&(1<<(n+1))!=0{captures.0[n].get_mut()[511].set_flags(PageTableFlags::NO_EXECUTE);}}
 let mut mapper=unsafe{MappedPageTable::new(&mut root,captures)};let page=Page::<Size4KiB>::from_start_address(VirtAddr::new(0)).unwrap();
 let mut retired=Retire(vec![]);unsafe{mapper.clean_up_addr_range(Page::range_inclusive(page,page),&mut retired)};
 let mut result=[raw(mapper.level_4_table(),0),0,0,0];
 for n in 0..3{let table=unsafe{&*mapper.page_table_frame_mapping().0[n].get()};result[n+1]=raw(table,0);assert_eq!(table[511].is_unused(),others&(1<<(n+1))==0);}
 (result,retired.0)
}
fn main(){
 assert_eq!(observe([0,8193,12289,0],0),([0,8193,12289,0],vec![]),"case 0");
assert_eq!(observe([0,8193,12289,0],2),([0,8193,12289,0],vec![]),"case 1");
assert_eq!(observe([0,8193,12289,0],4),([0,8193,12289,0],vec![]),"case 2");
assert_eq!(observe([0,8193,12289,0],8),([0,8193,12289,0],vec![]),"case 3");
assert_eq!(observe([512,8193,12289,0],0),([512,8193,12289,0],vec![]),"case 4");
assert_eq!(observe([512,8193,12289,0],2),([512,8193,12289,0],vec![]),"case 5");
assert_eq!(observe([512,8193,12289,0],4),([512,8193,12289,0],vec![]),"case 6");
assert_eq!(observe([512,8193,12289,0],8),([512,8193,12289,0],vec![]),"case 7");
assert_eq!(observe([128,8193,12289,0],0),([128,8193,12289,0],vec![]),"case 8");
assert_eq!(observe([128,8193,12289,0],2),([128,8193,12289,0],vec![]),"case 9");
assert_eq!(observe([128,8193,12289,0],4),([128,8193,12289,0],vec![]),"case 10");
assert_eq!(observe([128,8193,12289,0],8),([128,8193,12289,0],vec![]),"case 11");
assert_eq!(observe([4225,8193,12289,0],0),([4225,8193,12289,0],vec![]),"case 12");
assert_eq!(observe([4225,8193,12289,0],2),([4225,8193,12289,0],vec![]),"case 13");
assert_eq!(observe([4225,8193,12289,0],4),([4225,8193,12289,0],vec![]),"case 14");
assert_eq!(observe([4225,8193,12289,0],8),([4225,8193,12289,0],vec![]),"case 15");
assert_eq!(observe([4097,0,12289,0],0),([0,0,12289,0],vec![4096]),"case 16");
assert_eq!(observe([4097,0,12289,0],2),([4097,0,12289,0],vec![]),"case 17");
assert_eq!(observe([4097,0,12289,0],4),([0,0,12289,0],vec![4096]),"case 18");
assert_eq!(observe([4097,0,12289,0],8),([0,0,12289,0],vec![4096]),"case 19");
assert_eq!(observe([4097,512,12289,0],0),([4097,512,12289,0],vec![]),"case 20");
assert_eq!(observe([4097,512,12289,0],2),([4097,512,12289,0],vec![]),"case 21");
assert_eq!(observe([4097,512,12289,0],4),([4097,512,12289,0],vec![]),"case 22");
assert_eq!(observe([4097,512,12289,0],8),([4097,512,12289,0],vec![]),"case 23");
assert_eq!(observe([4097,128,12289,0],0),([4097,128,12289,0],vec![]),"case 24");
assert_eq!(observe([4097,128,12289,0],2),([4097,128,12289,0],vec![]),"case 25");
assert_eq!(observe([4097,128,12289,0],4),([4097,128,12289,0],vec![]),"case 26");
assert_eq!(observe([4097,128,12289,0],8),([4097,128,12289,0],vec![]),"case 27");
assert_eq!(observe([4097,8321,12289,0],0),([4097,8321,12289,0],vec![]),"case 28");
assert_eq!(observe([4097,8321,12289,0],2),([4097,8321,12289,0],vec![]),"case 29");
assert_eq!(observe([4097,8321,12289,0],4),([4097,8321,12289,0],vec![]),"case 30");
assert_eq!(observe([4097,8321,12289,0],8),([4097,8321,12289,0],vec![]),"case 31");
assert_eq!(observe([4097,8193,0,0],0),([0,0,0,0],vec![8192,4096]),"case 32");
assert_eq!(observe([4097,8193,0,0],2),([4097,0,0,0],vec![8192]),"case 33");
assert_eq!(observe([4097,8193,0,0],4),([4097,8193,0,0],vec![]),"case 34");
assert_eq!(observe([4097,8193,0,0],8),([0,0,0,0],vec![8192,4096]),"case 35");
assert_eq!(observe([4097,8193,512,0],0),([4097,8193,512,0],vec![]),"case 36");
assert_eq!(observe([4097,8193,512,0],2),([4097,8193,512,0],vec![]),"case 37");
assert_eq!(observe([4097,8193,512,0],4),([4097,8193,512,0],vec![]),"case 38");
assert_eq!(observe([4097,8193,512,0],8),([4097,8193,512,0],vec![]),"case 39");
assert_eq!(observe([4097,8193,128,0],0),([4097,8193,128,0],vec![]),"case 40");
assert_eq!(observe([4097,8193,128,0],2),([4097,8193,128,0],vec![]),"case 41");
assert_eq!(observe([4097,8193,128,0],4),([4097,8193,128,0],vec![]),"case 42");
assert_eq!(observe([4097,8193,128,0],8),([4097,8193,128,0],vec![]),"case 43");
assert_eq!(observe([4097,8193,12417,0],0),([4097,8193,12417,0],vec![]),"case 44");
assert_eq!(observe([4097,8193,12417,0],2),([4097,8193,12417,0],vec![]),"case 45");
assert_eq!(observe([4097,8193,12417,0],4),([4097,8193,12417,0],vec![]),"case 46");
assert_eq!(observe([4097,8193,12417,0],8),([4097,8193,12417,0],vec![]),"case 47");
assert_eq!(observe([4097,8193,12289,0],0),([0,0,0,0],vec![12288,8192,4096]),"case 48");
assert_eq!(observe([4097,8193,12289,0],1),([0,0,0,0],vec![12288,8192,4096]),"case 49");
assert_eq!(observe([4097,8193,12289,0],2),([4097,0,0,0],vec![12288,8192]),"case 50");
assert_eq!(observe([4097,8193,12289,0],3),([4097,0,0,0],vec![12288,8192]),"case 51");
assert_eq!(observe([4097,8193,12289,0],4),([4097,8193,0,0],vec![12288]),"case 52");
assert_eq!(observe([4097,8193,12289,0],5),([4097,8193,0,0],vec![12288]),"case 53");
assert_eq!(observe([4097,8193,12289,0],6),([4097,8193,0,0],vec![12288]),"case 54");
assert_eq!(observe([4097,8193,12289,0],7),([4097,8193,0,0],vec![12288]),"case 55");
assert_eq!(observe([4097,8193,12289,0],8),([4097,8193,12289,0],vec![]),"case 56");
assert_eq!(observe([4097,8193,12289,0],9),([4097,8193,12289,0],vec![]),"case 57");
assert_eq!(observe([4097,8193,12289,0],10),([4097,8193,12289,0],vec![]),"case 58");
assert_eq!(observe([4097,8193,12289,0],11),([4097,8193,12289,0],vec![]),"case 59");
assert_eq!(observe([4097,8193,12289,0],12),([4097,8193,12289,0],vec![]),"case 60");
assert_eq!(observe([4097,8193,12289,0],13),([4097,8193,12289,0],vec![]),"case 61");
assert_eq!(observe([4097,8193,12289,0],14),([4097,8193,12289,0],vec![]),"case 62");
assert_eq!(observe([4097,8193,12289,0],15),([4097,8193,12289,0],vec![]),"case 63");
assert_eq!(observe([4097,8193,12289,512],0),([4097,8193,12289,512],vec![]),"case 64");
assert_eq!(observe([4097,8193,12289,512],1),([4097,8193,12289,512],vec![]),"case 65");
assert_eq!(observe([4097,8193,12289,512],2),([4097,8193,12289,512],vec![]),"case 66");
assert_eq!(observe([4097,8193,12289,512],3),([4097,8193,12289,512],vec![]),"case 67");
assert_eq!(observe([4097,8193,12289,512],4),([4097,8193,12289,512],vec![]),"case 68");
assert_eq!(observe([4097,8193,12289,512],5),([4097,8193,12289,512],vec![]),"case 69");
assert_eq!(observe([4097,8193,12289,512],6),([4097,8193,12289,512],vec![]),"case 70");
assert_eq!(observe([4097,8193,12289,512],7),([4097,8193,12289,512],vec![]),"case 71");
assert_eq!(observe([4097,8193,12289,512],8),([4097,8193,12289,512],vec![]),"case 72");
assert_eq!(observe([4097,8193,12289,512],9),([4097,8193,12289,512],vec![]),"case 73");
assert_eq!(observe([4097,8193,12289,512],10),([4097,8193,12289,512],vec![]),"case 74");
assert_eq!(observe([4097,8193,12289,512],11),([4097,8193,12289,512],vec![]),"case 75");
assert_eq!(observe([4097,8193,12289,512],12),([4097,8193,12289,512],vec![]),"case 76");
assert_eq!(observe([4097,8193,12289,512],13),([4097,8193,12289,512],vec![]),"case 77");
assert_eq!(observe([4097,8193,12289,512],14),([4097,8193,12289,512],vec![]),"case 78");
assert_eq!(observe([4097,8193,12289,512],15),([4097,8193,12289,512],vec![]),"case 79");
assert_eq!(observe([4097,8193,12289,128],0),([4097,8193,12289,128],vec![]),"case 80");
assert_eq!(observe([4097,8193,12289,128],1),([4097,8193,12289,128],vec![]),"case 81");
assert_eq!(observe([4097,8193,12289,128],2),([4097,8193,12289,128],vec![]),"case 82");
assert_eq!(observe([4097,8193,12289,128],3),([4097,8193,12289,128],vec![]),"case 83");
assert_eq!(observe([4097,8193,12289,128],4),([4097,8193,12289,128],vec![]),"case 84");
assert_eq!(observe([4097,8193,12289,128],5),([4097,8193,12289,128],vec![]),"case 85");
assert_eq!(observe([4097,8193,12289,128],6),([4097,8193,12289,128],vec![]),"case 86");
assert_eq!(observe([4097,8193,12289,128],7),([4097,8193,12289,128],vec![]),"case 87");
assert_eq!(observe([4097,8193,12289,128],8),([4097,8193,12289,128],vec![]),"case 88");
assert_eq!(observe([4097,8193,12289,128],9),([4097,8193,12289,128],vec![]),"case 89");
assert_eq!(observe([4097,8193,12289,128],10),([4097,8193,12289,128],vec![]),"case 90");
assert_eq!(observe([4097,8193,12289,128],11),([4097,8193,12289,128],vec![]),"case 91");
assert_eq!(observe([4097,8193,12289,128],12),([4097,8193,12289,128],vec![]),"case 92");
assert_eq!(observe([4097,8193,12289,128],13),([4097,8193,12289,128],vec![]),"case 93");
assert_eq!(observe([4097,8193,12289,128],14),([4097,8193,12289,128],vec![]),"case 94");
assert_eq!(observe([4097,8193,12289,128],15),([4097,8193,12289,128],vec![]),"case 95");
assert_eq!(observe([4097,8193,12289,1],0),([4097,8193,12289,1],vec![]),"case 96");
assert_eq!(observe([4097,8193,12289,1],1),([4097,8193,12289,1],vec![]),"case 97");
assert_eq!(observe([4097,8193,12289,1],2),([4097,8193,12289,1],vec![]),"case 98");
assert_eq!(observe([4097,8193,12289,1],3),([4097,8193,12289,1],vec![]),"case 99");
assert_eq!(observe([4097,8193,12289,1],4),([4097,8193,12289,1],vec![]),"case 100");
assert_eq!(observe([4097,8193,12289,1],5),([4097,8193,12289,1],vec![]),"case 101");
assert_eq!(observe([4097,8193,12289,1],6),([4097,8193,12289,1],vec![]),"case 102");
assert_eq!(observe([4097,8193,12289,1],7),([4097,8193,12289,1],vec![]),"case 103");
assert_eq!(observe([4097,8193,12289,1],8),([4097,8193,12289,1],vec![]),"case 104");
assert_eq!(observe([4097,8193,12289,1],9),([4097,8193,12289,1],vec![]),"case 105");
assert_eq!(observe([4097,8193,12289,1],10),([4097,8193,12289,1],vec![]),"case 106");
assert_eq!(observe([4097,8193,12289,1],11),([4097,8193,12289,1],vec![]),"case 107");
assert_eq!(observe([4097,8193,12289,1],12),([4097,8193,12289,1],vec![]),"case 108");
assert_eq!(observe([4097,8193,12289,1],13),([4097,8193,12289,1],vec![]),"case 109");
assert_eq!(observe([4097,8193,12289,1],14),([4097,8193,12289,1],vec![]),"case 110");
assert_eq!(observe([4097,8193,12289,1],15),([4097,8193,12289,1],vec![]),"case 111");
 println!("PASS actual pinned cleanup branches and bottom-up callback order on detached tables");
}
