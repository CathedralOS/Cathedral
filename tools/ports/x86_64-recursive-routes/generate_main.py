#!/usr/bin/env python3
from pathlib import Path
import json,sys
HERE=Path(__file__).resolve().parent
rows=json.loads((HERE/'cases.json').read_text());translations=json.loads((HERE/'translation-cases.json').read_text())
s='''// SPDX-License-Identifier: MIT OR Apache-2.0
// Original safe adapters around copied source bodies, not a RecursivePageTable.
mod observed;mod pinned;
pub use observed::{ObservedTable,ObservedEntry,AllocationObservations};
use observed::Queue;
use x86_64::{VirtAddr,PhysAddr,structures::paging::{Page,PhysFrame,PageTableFlags,Size4KiB,Size2MiB,Size1GiB,mapper::{MapToError,UnmapError,FlagUpdateError,TranslateError,TranslateResult}}};
use std::panic::{catch_unwind,AssertUnwindSafe};
#[derive(Clone,Copy)]struct Case {size:u64,op:u8,words:[u64;4],frames:[Option<u64>;3],flags:u64,parent:u64,level:u8,page:u64,frame:u64}
fn map_error<S:x86_64::structures::paging::PageSize>(e:MapToError<S>)->(u64,u64){match e{MapToError::FrameAllocationFailed=>(6,0),MapToError::ParentEntryHugePage=>(2,0),MapToError::PageAlreadyMapped(f)=>(4,f.start_address().as_u64())}}
fn unmap_error(e:UnmapError)->(u64,u64){match e{UnmapError::PageNotMapped=>(1,0),UnmapError::ParentEntryHugePage=>(2,0),UnmapError::InvalidFrameAddress(a)=>(3,a.as_u64())}}
fn flag_error(e:FlagUpdateError)->(u64,u64){match e{FlagUpdateError::PageNotMapped=>(1,0),FlagUpdateError::ParentEntryHugePage=>(2,0)}}
fn translate_error(e:TranslateError)->(u64,u64){match e{TranslateError::PageNotMapped=>(1,0),TranslateError::ParentEntryHugePage=>(2,0),TranslateError::InvalidFrameAddress(a)=>(3,a.as_u64())}}
fn execute(r:Case,tables:&mut[ObservedTable;4],queue:&mut Queue)->(u64,u64){
 let [p4,p3,p2,p1]=tables;
 let flags=PageTableFlags::from_bits_retain(r.flags);let parent=PageTableFlags::from_bits_retain(r.parent);
 match r.size {
'''
for size,num in [('Size1GiB',1<<30),('Size2MiB',1<<21),('Size4KiB',4096)]:
 s+=f'''{num}=>{{let page=Page::<{size}>::from_start_address(VirtAddr::new(r.page)).unwrap();
 match r.op {{
 0=>{{let frame=PhysFrame::<{size}>::from_start_address(PhysAddr::new(r.frame)).unwrap();match pinned::map_to_with_table_flags_{size}(p4,p3,p2,p1,page,frame,flags,parent,queue){{Ok(_)=>(0,r.frame),Err(e)=>map_error(e)}}}},
 1=>match pinned::unmap_{size}(p4,p3,p2,p1,page){{Ok((f,_))=>(0,f.start_address().as_u64()),Err(e)=>unmap_error(e)}},
 2=>match pinned::update_flags_{size}(p4,p3,p2,p1,page,flags){{Ok(_)=>(0,0),Err(e)=>flag_error(e)}},
 3=>match pinned::translate_page_{size}(p4,p3,p2,p1,page){{Ok(f)=>(0,f.start_address().as_u64()),Err(e)=>translate_error(e)}},
 4=>{{let result=match r.level {{4=>pinned::set_flags_p4_entry_{size}(p4,p3,p2,p1,page,flags),3=>pinned::set_flags_p3_entry_{size}(p4,p3,p2,p1,page,flags),2=>pinned::set_flags_p2_entry_{size}(p4,p3,p2,p1,page,flags),_=>unreachable!()}};match result{{Ok(_)=>(0,0),Err(e)=>flag_error(e)}}}},
 _=>unreachable!()}}}},
'''
s+='''_=>unreachable!()}}
fn main(){std::panic::set_hook(Box::new(|_|{}));let cases:&[Case]=&[
'''
for r in rows:
 frames='['+','.join('None' if n is None else f'Some({n})' for n in r['frames'])+']'
 s+='Case{'+f'size:{r["size"]},op:{["map","unmap","update","translate","parent"].index(r["op"])},words:{r["words"]},frames:{frames},flags:{r["flags"]},parent:{r["parent"]},level:{r["level"]},page:{r["page"]},frame:{r["frame"]}'+'},\n'
s+=''' ];println!("{{\\\"routes\\\":[");for (n,r) in cases.iter().copied().enumerate(){
 let indices=[((r.page>>39)&511)as usize,((r.page>>30)&511)as usize,((r.page>>21)&511)as usize,((r.page>>12)&511)as usize];
 let mut tables=std::array::from_fn(|j|ObservedTable::new(indices[j],r.words[j]));let mut q=Queue{frames:r.frames,calls:0};
 let result=catch_unwind(AssertUnwindSafe(||execute(r,&mut tables,&mut q)));
 let (code,payload)=match result{Ok(v)=>v,Err(e)=>{let msg=e.downcast_ref::<&str>().copied().or_else(||e.downcast_ref::<String>().map(String::as_str)).unwrap();assert!(msg.contains("HUGE_PAGE"),"unexpected panic {msg}");(5,0)}};
 let words:[u64;4]=std::array::from_fn(|j|tables[j].entries[indices[j]].word());
 let mut write=0;let mut zero=0;let mut visited=0;
 for j in 0..4 {if tables[j].entries[indices[j]].writes>0{write|=1<<j;}if tables[j].zeros>0{zero|=1<<j;assert!(tables[j].entries[if indices[j]==511{0}else{511}].is_unused());}if tables[j].touches.get()>0{visited=j+1;}}
 println!("{{\\\"words\\\":{:?},\\\"write\\\":{},\\\"zero\\\":{},\\\"calls\\\":{},\\\"visited\\\":{},\\\"code\\\":{},\\\"payload\\\":{}}}{}",words,write,zero,q.calls,visited,code,payload,if n+1==cases.len(){""}else{","});
 }println!("],\\\"translations\\\":[");let translations:&[(u64,[u64;4])]=&[
'''
for r in translations:s+=f'({r["va"]},{r["words"]}),\n'
s+=''' ];for (n,(va,words)) in translations.iter().enumerate(){
 let indices=[((va>>39)&511)as usize,((va>>30)&511)as usize,((va>>21)&511)as usize,((va>>12)&511)as usize];let t:[ObservedTable;4]=std::array::from_fn(|j|ObservedTable::new(indices[j],words[j]));
 let result=catch_unwind(AssertUnwindSafe(||pinned::translate(&t[0],&t[1],&t[2],&t[3],VirtAddr::new(*va))));
 let values=match result{Ok(TranslateResult::NotMapped)=>[1,0,0,0,0],Ok(TranslateResult::InvalidFrameAddress(a))=>[4,a.as_u64(),0,0,0],Ok(TranslateResult::Mapped{frame,offset,flags})=>[0,frame.start_address().as_u64(),frame.size(),offset,flags.bits()],Err(e)=>{let msg=e.downcast_ref::<&str>().copied().or_else(||e.downcast_ref::<String>().map(String::as_str)).unwrap();if msg.contains("level 4"){[2,0,0,0,0]}else if msg.contains("level 1"){[3,0,0,0,0]}else{panic!("unexpected panic {msg}")}}};
 println!("{:?}{}",values,if n+1==translations.len(){""}else{","});}println!("]}}");}
'''
p=HERE/'src/main.rs'
if '--check' in sys.argv:
 if not p.exists() or p.read_text()!=s:raise SystemExit('main reference drift')
else:p.write_text(s)
