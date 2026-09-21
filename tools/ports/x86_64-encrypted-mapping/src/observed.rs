// SPDX-License-Identifier: MIT OR Apache-2.0
// Original safe observation adapters. Methods execute actual pinned PTE APIs;
// counters expose assignments and full-table clearing, including redundant writes.
use std::ops::{Index,IndexMut};
use std::cell::Cell;
use x86_64::{PhysAddr,structures::paging::{PageSize,Size4KiB,PhysFrame,PageTable,PageTableFlags,PageTableIndex,page_table::{PageTableEntry,FrameError}}};
pub struct ObservedEntry { pub value:PageTableEntry, pub writes:u32 }
impl ObservedEntry {
 pub fn new(word:u64)->Self {let mut value=PageTableEntry::new();value.set_addr(PhysAddr::zero(),PageTableFlags::from_bits_retain(word));Self{value,writes:0}}
 pub fn word(&self)->u64 {self.value.addr().as_u64()|self.value.flags().bits()}
 pub fn is_unused(&self)->bool {self.value.is_unused()}
 pub fn flags(&self)->PageTableFlags {self.value.flags()}
 pub fn addr(&self)->PhysAddr {self.value.addr()}
 pub fn frame(&self)->Result<PhysFrame,FrameError> {self.value.frame()}
 pub fn set_addr(&mut self,addr:PhysAddr,flags:PageTableFlags){self.value.set_addr(addr,flags);self.writes+=1;}
 pub fn set_frame(&mut self,frame:PhysFrame,flags:PageTableFlags){self.value.set_frame(frame,flags);self.writes+=1;}
 pub fn set_flags(&mut self,flags:PageTableFlags){self.value.set_flags(flags);self.writes+=1;}
 pub fn set_unused(&mut self){self.value.set_unused();self.writes+=1;}
}
pub struct ObservedTable {pub entries:[ObservedEntry;512],pub zeros:u32,pub touches:Cell<u32>}
impl ObservedTable {
 pub fn new(index:usize,word:u64)->Self {let mut entries=std::array::from_fn(|_|ObservedEntry::new(0));entries[index]=ObservedEntry::new(word);entries[if index==511{0}else{511}]=ObservedEntry::new(512);Self{entries,zeros:0,touches:Cell::new(0)}}
 pub fn zero(&mut self){let mut raw=PageTable::new();for (n,entry) in self.entries.iter().enumerate(){raw[n]=entry.value.clone();}raw.zero();for (n,entry) in self.entries.iter_mut().enumerate(){entry.value=raw[n].clone();}self.zeros+=1;}
}
impl Index<PageTableIndex> for ObservedTable {type Output=ObservedEntry;fn index(&self,index:PageTableIndex)->&ObservedEntry{self.touches.set(self.touches.get()+1);&self.entries[usize::from(index)]}}
impl IndexMut<PageTableIndex> for ObservedTable {fn index_mut(&mut self,index:PageTableIndex)->&mut ObservedEntry{self.touches.set(self.touches.get()+1);&mut self.entries[usize::from(index)]}}
// Observation input, deliberately not the unsafe upstream FrameAllocator trait.
// Its numeric returns are never dereferenced or presented as backing custody.
pub trait AllocationObservations<S:PageSize> {fn allocate_frame(&mut self)->Option<PhysFrame<S>>;}
pub struct Queue {pub frames:[Option<u64>;3],pub calls:usize}
impl AllocationObservations<Size4KiB> for Queue {
 fn allocate_frame(&mut self)->Option<PhysFrame>{let next=self.frames.get(self.calls).copied().flatten();self.calls+=1;next.map(|a|PhysFrame::from_start_address(PhysAddr::new(a)).unwrap())}
}
