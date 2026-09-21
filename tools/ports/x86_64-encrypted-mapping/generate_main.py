#!/usr/bin/env python3
from pathlib import Path
import json,sys
HERE=Path(__file__).resolve().parent;rows=json.loads((HERE/'cases.json').read_text());profiles=json.loads((HERE/'profiles.json').read_text())
s='''// SPDX-License-Identifier: MIT OR Apache-2.0
// Actual public mapped calls cross-check extracted instrumented bodies.
mod observed;mod mapped;mod recursive;mod public_mapped;
pub use observed::{ObservedTable,ObservedEntry,AllocationObservations};
use observed::Queue;
use x86_64::{VirtAddr,PhysAddr,structures::{mem_encrypt::{enable_memory_encryption,MemoryEncryptionConfiguration},paging::{Page,PhysFrame,PageTableFlags,Size4KiB,Size2MiB,Size1GiB,mapper::{MapToError,UnmapError,FlagUpdateError,TranslateError}}}};
use std::panic::{catch_unwind,AssertUnwindSafe};
#[derive(Clone,Copy)]struct Case{profile:usize,recursive:bool,size:u64,op:u8,words:[u64;4],frames:[Option<u64>;3],flags:u64,parent:u64,level:u8,page:u64,frame:u64}
fn map_error<S:x86_64::structures::paging::PageSize>(e:MapToError<S>)->(u64,u64){match e{MapToError::FrameAllocationFailed=>(6,0),MapToError::ParentEntryHugePage=>(2,0),MapToError::PageAlreadyMapped(f)=>(4,f.start_address().as_u64())}}
fn unmap_error(e:UnmapError)->(u64,u64){match e{UnmapError::PageNotMapped=>(1,0),UnmapError::ParentEntryHugePage=>(2,0),UnmapError::InvalidFrameAddress(a)=>(3,a.as_u64())}}
fn flag_error(e:FlagUpdateError)->(u64,u64){match e{FlagUpdateError::PageNotMapped=>(1,0),FlagUpdateError::ParentEntryHugePage=>(2,0)}}
fn translate_error(e:TranslateError)->(u64,u64){match e{TranslateError::PageNotMapped=>(1,0),TranslateError::ParentEntryHugePage=>(2,0),TranslateError::InvalidFrameAddress(a)=>(3,a.as_u64())}}
fn execute(r:Case,tables:&mut[ObservedTable;4],queue:&mut Queue)->(u64,u64){
 let [p4,p3,p2,p1]=tables;let flags=PageTableFlags::from_bits_retain(r.flags);let parent=PageTableFlags::from_bits_retain(r.parent);
 match (r.recursive,r.size){
'''
for mode in ['mapped','recursive']:
 for size,num in [('Size1GiB',1<<30),('Size2MiB',1<<21),('Size4KiB',4096)]:
  s+=f'''({str(mode=='recursive').lower()},{num})=>{{let page=Page::<{size}>::from_start_address(VirtAddr::new(r.page)).unwrap();match r.op{{
0=>{{let frame=PhysFrame::<{size}>::from_start_address(PhysAddr::new(r.frame)).unwrap();match {mode}::map_to_with_table_flags_{size}(p4,p3,p2,p1,page,frame,flags,parent,queue){{Ok(_)=>(0,r.frame),Err(e)=>map_error(e)}}}},
1=>match {mode}::unmap_{size}(p4,p3,p2,p1,page){{Ok((f,_))=>(0,f.start_address().as_u64()),Err(e)=>unmap_error(e)}},
2=>match {mode}::update_flags_{size}(p4,p3,p2,p1,page,flags){{Ok(_)=>(0,0),Err(e)=>flag_error(e)}},
3=>match {mode}::translate_page_{size}(p4,p3,p2,p1,page){{Ok(f)=>(0,f.start_address().as_u64()),Err(e)=>translate_error(e)}},
4=>{{let result=match r.level{{4=>{mode}::set_flags_p4_entry_{size}(p4,p3,p2,p1,page,flags),3=>{mode}::set_flags_p3_entry_{size}(p4,p3,p2,p1,page,flags),2=>{mode}::set_flags_p2_entry_{size}(p4,p3,p2,p1,page,flags),_=>unreachable!()}};match result{{Ok(_)=>(0,0),Err(e)=>flag_error(e)}}}},_=>unreachable!()}}}},
'''
s+='''_=>unreachable!()}}
fn main(){std::panic::set_hook(Box::new(|_|{}));let profile:usize=std::env::args().nth(1).expect("profile index").parse().unwrap();
// Isolated process, configuration precedes all typed addresses and tables.
let profiles:&[&[(u8,bool)]]=&[
'''
for p in profiles:s+='&['+','.join(f'({bit},{str(reverse).lower()})' for bit,reverse in p['configs'])+'],\n'
s+=''' ];for &(bit,reverse) in profiles[profile]{unsafe{enable_memory_encryption(if reverse{MemoryEncryptionConfiguration::SharedBit(bit)}else{MemoryEncryptionConfiguration::EncryptedBit(bit)});}}
let cases:&[Case]=&[
'''
for r in rows:
 frames='['+','.join('None' if n is None else f'Some({n})' for n in r['frames'])+']'
 s+='Case{'+f'profile:{r["profile"]},recursive:{str(r["recursive"]).lower()},size:{r["size"]},op:{["map","unmap","update","translate","parent"].index(r["op"])},words:{r["words"]},frames:{frames},flags:{r["flags"]},parent:{r["parent"]},level:{r["level"]},page:{r["page"]},frame:{r["frame"]}'+'},\n'
s+=''' ];println!("[");let selected:Vec<_>=cases.iter().copied().enumerate().filter(|(_,r)|r.profile==profile).collect();for (n,(case_index,r)) in selected.iter().copied().enumerate(){
 let indices=[((r.page>>39)&511)as usize,((r.page>>30)&511)as usize,((r.page>>21)&511)as usize,((r.page>>12)&511)as usize];let mut tables=std::array::from_fn(|j|ObservedTable::new(indices[j],r.words[j]));let mut q=Queue{frames:r.frames,calls:0};
 let result=catch_unwind(AssertUnwindSafe(||execute(r,&mut tables,&mut q)));
 let (code,payload)=match result{Ok(v)=>v,Err(e)=>{let msg=e.downcast_ref::<&str>().copied().or_else(||e.downcast_ref::<String>().map(String::as_str)).unwrap();if msg.contains("HUGE_PAGE"){(5,0)}else if msg.contains("entry should be mapped"){(7,0)}else{panic!("unexpected panic {msg} in case {case_index}")}}};
 let words:[u64;4]=std::array::from_fn(|j|tables[j].entries[indices[j]].word());let(mut write,mut zero,mut visited)=(0,0,0);
 for j in 0..4{if tables[j].entries[indices[j]].writes>0{write|=1<<j;}if tables[j].zeros>0{zero|=1<<j;assert!(tables[j].entries[511].is_unused());}if tables[j].touches.get()>0{visited=j+1;}}
 if !r.recursive{let op=["map","unmap","update","translate","parent"][r.op as usize];let frames=r.frames.map(|x|x.unwrap_or(0));let actual=match r.size{
'''
for size,num in [('Size1GiB',1<<30),('Size2MiB',1<<21),('Size4KiB',4096)]:s+=f'{num}=>public_mapped::observe::<{size}>(op,r.level,r.page,r.words,frames,r.frame,r.flags,r.parent),\n'
s+='''_=>unreachable!()};assert_eq!((actual.kind as u64,actual.payload,actual.words,actual.calls as usize,actual.zero),(code,payload,words,q.calls,zero),"actual public mapped versus instrumented mirror case {case_index}");}
 println!("{{\\\"index\\\":{},\\\"words\\\":{:?},\\\"write\\\":{},\\\"zero\\\":{},\\\"calls\\\":{},\\\"visited\\\":{},\\\"code\\\":{},\\\"payload\\\":{}}}{}",case_index,words,write,zero,q.calls,visited,code,payload,if n+1==selected.len(){""}else{","});}println!("]");}
'''
p=HERE/'src/main.rs'
if '--check' in sys.argv:
 if not p.exists() or p.read_text()!=s:raise SystemExit('main drift')
else:p.write_text(s)
print('PASS generated main')
