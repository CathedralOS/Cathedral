#!/usr/bin/env python3
"""Extract pinned pure fragments, supplying observations and excluding hardware tails."""
from pathlib import Path
import importlib.util,json,sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];UP=ROOT/'reference_code/rust-osdev/x86_64';PIN='cc35c876d3badb57df54a66e22f7768a52be95f2'
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');i=importlib.util.module_from_spec(spec);spec.loader.exec_module(i)
FILES=['src/registers/control.rs','src/registers/model_specific.rs','src/registers/rflags.rs','src/registers/xcontrol.rs','src/instructions/tlb.rs']
i.snapshot(UP,PIN,FILES,'https://github.com/rust-osdev/x86_64')
sources={p:(UP/p).read_text() for p in FILES};control=sources[FILES[0]];msr=sources[FILES[1]];xf=sources[FILES[3]]
def between(s,start,end):
 a=s.index(start);b=s.index(end,a);return s[a:b]
def merge(s,typ):return between(s,'            let reserved = old_value & !('+typ+'::all().bits());','\n\n            unsafe {')
pinned='''// SPDX-License-Identifier: MIT OR Apache-2.0
// Generated exact pinned body fragments. Explicit old observations replace reads;
// returned operands replace instruction writes. No privileged API is called.
use core::fmt;
use x86_64::{PhysAddr,VirtAddr};
use x86_64::structures::paging::{Page,PhysFrame,Size4KiB};
use x86_64::registers::{control::{Cr0Flags,Cr3Flags,Cr4Flags,PriorityClass},model_specific::{EferFlags,CetFlags,ApicBaseFlags},rflags::RFlags,xcontrol::XCr0Flags,segmentation::SegmentSelector};
'''
for name,typ,s in [('cr0','Cr0Flags',control),('cr4','Cr4Flags',control),('efer','EferFlags',msr),('rflags','RFlags',sources[FILES[2]])]:
 # rflags indentation differs; preserve exact extracted statements.
 fragment=between(s,'let reserved = old_value & !('+typ+'::all().bits());','\n\n')
 pinned+=f'pub fn {name}(old_value:u64, flags:{typ})->u64 {{\n'+fragment+'\nnew_value\n}\n'
pinned+='pub fn xcr0_merge(old_value:u64,flags:XCr0Flags)->u64 {\n'+between(xf,'            let reserved = old_value & !(XCr0Flags::all().bits());','\n\n')+'\nnew_value\n}\n'
pinned+='pub fn xcr0(old_value:u64,flags:XCr0Flags)->u64 {\n'+merge(xf,'XCr0Flags')+'\nnew_value\n}\n'
pcid=between(sources[FILES[4]],'#[repr(transparent)]\n#[derive(Debug, Clone, Copy','/// Invalidate the given address in the TLB using the `invpcid`')
pinned+=pcid
pinned+='pub fn cr3_observed(value:u64)->(PhysFrame,u16) {\n'+between(control,'            let addr = PhysAddr::new(value & 0x_000f_ffff_ffff_f000);','\n        }')+'\n}\n'
pinned+='pub fn cr3_raw(top_bit:bool,frame:PhysFrame,val:u16)->u64 {\n'+between(control,'            let addr = frame.start_address();','\n\n            unsafe {')+'\nvalue\n}\n'
pinned+='pub fn cr3_flags(frame:PhysFrame,flags:Cr3Flags)->u64 { cr3_raw(false,frame,flags.bits() as u16) }\n'
pinned+='pub fn cr3_pcid(top_bit:bool,frame:PhysFrame,pcid:Pcid)->u64 { cr3_raw(top_bit,frame,pcid.value()) }\n'
assert 'PriorityClass::new(Self::read_raw() as u8)' in control
pinned+='pub fn cr8_observed(raw:u64)->Option<PriorityClass> { PriorityClass::new(raw as u8) }\n'
pinned+='pub fn cr8_operand(priority_class:Option<PriorityClass>)->u64 {\n'+between(control,'            let value = priority_class.map_or(0, |pc| pc as u64);','\n            Self::write_raw(value);')+'\nvalue\n}\n'
pinned+='pub fn star(raw:(u16,u16))->(SegmentSelector,SegmentSelector,SegmentSelector,SegmentSelector) {\n'+between(msr,'            (\n                SegmentSelector(raw.0 + 16),','\n        }')+'\n}\n'
cet=between(msr,'            let cet_flags = CetFlags::from_bits_truncate(value);','\n        }');assert msr.count(cet)==2
pinned+='pub fn cet_observed(value:u64)->(CetFlags,Page) {\n'+cet+'\n}\n'
assert msr.count('Self::write_raw(flags.bits() | legacy_bitmap.start_address().as_u64());')==2
pinned+='pub fn cet_operand(flags:CetFlags,legacy_bitmap:Page)->u64 { flags.bits() | legacy_bitmap.start_address().as_u64() }\n'
pinned+='pub fn apic_observed(raw:u64)->(PhysFrame,u64) {\n'+between(msr,'            let addr = PhysAddr::new_truncate(raw);','\n        }')+'\n}\n'
pinned+='pub fn apic_preserving(old_flags:u64,frame:PhysFrame,flags:ApicBaseFlags)->u64 {\n'+between(msr,'            let reserved = old_flags & !(ApicBaseFlags::all().bits());','\n\n            unsafe {')+'\napic_raw(frame,new_flags)\n}\n'
assert 'msr.write(flags | addr.as_u64());' in msr
pinned+='pub fn apic_raw(frame:PhysFrame,flags:u64)->u64 { let addr=frame.start_address(); flags | addr.as_u64() }\n'
# Generic from_bits_retain receives every input bit, matching actual typed API semantics.
M=(1<<64)-1;cases=[]
def case(op,*a):cases.append({'op':op,'args':list(a)})
for kind in range(5):
 for bit in range(64):case('merge',kind,M,1<<bit)
 for old,supplied in [(0,0),(0,M),(M,0),(0xaaaaaaaaaaaaaaaa,0x5555555555555555),(0x5555555555555555,0xaaaaaaaaaaaaaaaa)]:case('merge',kind,old,supplied)
for flags in range(256):case('xcr0',M,flags)
for flags in [0,1,3,7,31,255]:
 for extra in [512,1<<62,1<<63]:case('xcr0',0,flags|extra)
for raw in [0,M,0x12345018,0x8000000000000001]+[1<<b for b in range(64)]:case('cr3_observed',raw)
for frame in [0,4096,0xffffffffff000]:
 for val in [0,24,4095,4096,0x8000,65535]:
  for top in [0,1]:case('cr3_raw',frame,val,top)
 for flags in [0,24,65535,65536,M]:case('cr3_flags',frame,flags)
 for pcid in [0,1,4095]:
  for top in [0,1]:case('cr3_pcid',frame,pcid,top)
for frame in [1,4095,1<<52,M]:case('cr3_raw',frame,65535,1)
for raw in list(range(256))+[257,271,272,512,M]:case('cr8_observed',raw)
for priority in range(16):case('cr8_operand',priority)
for sysret in [0,3,65519,65520,65527,65528,65535]:
 for syscall in [0,8,65527,65528,65535]:case('star',(sysret<<48)|(syscall<<32)|0xffffffff)
for raw in [0,M,0xffff800012345fff]+[1<<b for b in range(64)]:case('cet_observed',raw)
for bitmap in [0,4096,0xffff800000000000,0xfffffffffffff000]:
 for flags in [0,3135,4096,1<<47,1<<63,M]:case('cet_operand',bitmap,flags)
for bitmap in [1,4095,1<<47]:case('cet_operand',bitmap,1)
for raw in [0,M,0xfee00d00]+[1<<b for b in range(64)]:case('apic_observed',raw)
for frame in [0,4096,0xffffffffff000]:
 for flags in [0,3328,4096,M]:case('apic_raw',frame,flags)
for frame in [1,1<<52,M]:case('apic_raw',frame,0)
for old in [0,M,0xfee00d00,4096,0x8000000000000000]:
 for frame in [0,8192,0xfee00000]:
  for flags in [0,3328,4096,1<<63,M]:case('apic_preserving',old,frame,flags)
main='''// SPDX-License-Identifier: MIT OR Apache-2.0
mod pinned;
use pinned::*;
use x86_64::{PhysAddr,VirtAddr};
use x86_64::structures::paging::{PhysFrame,Page};
use x86_64::registers::{control::{Cr0Flags,Cr3Flags,Cr4Flags,PriorityClass},model_specific::{EferFlags,CetFlags,ApicBaseFlags},rflags::RFlags,xcontrol::XCr0Flags};
use std::panic::{catch_unwind,AssertUnwindSafe};
fn frame(x:u64)->PhysFrame { PhysFrame::from_start_address(PhysAddr::new(x)).unwrap() }
fn result(f:impl FnOnce()->u64)->[u64;5] { match catch_unwind(AssertUnwindSafe(f)){Ok(x)=>[0,x,0,0,0],Err(_)=>[1,0,0,0,0]} }
fn observe(op:&str,a:&[u64])->[u64;5] {
match op {
"merge"=>[0,match a[0]{0=>cr0(a[1],Cr0Flags::from_bits_retain(a[2])),1=>cr4(a[1],Cr4Flags::from_bits_retain(a[2])),2=>efer(a[1],EferFlags::from_bits_retain(a[2])),3=>rflags(a[1],RFlags::from_bits_retain(a[2])),4=>xcr0_merge(a[1],XCr0Flags::from_bits_retain(a[2])),_=>unreachable!()},0,0,0],
"xcr0"=>match catch_unwind(||xcr0(a[0],XCr0Flags::from_bits_retain(a[1]))){Ok(x)=>[0,x,0,0,0],Err(e)=>{let msg=e.downcast_ref::<&str>().copied().or_else(||e.downcast_ref::<String>().map(String::as_str)).unwrap();let code=if msg.contains("X87"){1}else if msg.contains("without enabling SSE"){2}else if msg.contains("MPX"){3}else if msg.contains("without enabling AVX"){4}else if msg.contains("AVX-512 flags"){5}else{panic!("unrecognized assertion {msg}")};[code,0,0,0,0]}},
"cr3_observed"=>{let(f,v)=cr3_observed(a[0]);[f.start_address().as_u64(),v as u64,Cr3Flags::from_bits_truncate(v.into()).bits(),Pcid::new(v).unwrap().value() as u64,0]},
"cr3_raw"=>result(||cr3_raw(a[2]!=0,frame(a[0]),a[1] as u16)),
"cr3_flags"=>result(||cr3_flags(frame(a[0]),Cr3Flags::from_bits_retain(a[1]))),
"cr3_pcid"=>result(||cr3_pcid(a[2]!=0,frame(a[0]),Pcid::new(a[1] as u16).unwrap())),
"cr8_observed"=>match cr8_observed(a[0]){Some(p)=>[0,p as u64,0,0,0],None=>[1,0,0,0,0]},
"cr8_operand"=>[0,cr8_operand(if a[0]==0{None}else{Some(PriorityClass::new(a[0] as u8).unwrap())}),0,0,0],
"star"=>match catch_unwind(||star(((a[0]>>48)as u16,(a[0]>>32)as u16))){Ok((a,b,c,d))=>[0,a.0 as u64,b.0 as u64,c.0 as u64,d.0 as u64],Err(_)=>[1,0,0,0,0]},
"cet_observed"=>match catch_unwind(||cet_observed(a[0])){Ok((f,p))=>[0,f.bits(),p.start_address().as_u64(),0,0],Err(_)=>[1,0,0,0,0]},
"cet_operand"=>result(||cet_operand(CetFlags::from_bits_retain(a[1]),Page::from_start_address(VirtAddr::new(a[0])).unwrap())),
"apic_observed"=>{let(f,raw)=apic_observed(a[0]);[f.start_address().as_u64(),raw,ApicBaseFlags::from_bits_truncate(raw).bits(),0,0]},
"apic_raw"=>result(||apic_raw(frame(a[0]),a[1])),
"apic_preserving"=>result(||apic_preserving(a[0],frame(a[1]),ApicBaseFlags::from_bits_retain(a[2]))),
_=>panic!("unknown op")}}
fn main(){std::panic::set_hook(Box::new(|_|{}));let cases:&[(&str,&[u64])]=&[
'''
main+=''.join('('+json.dumps(c['op'])+',&['+','.join(str(a) for a in c['args'])+']),\n' for c in cases)
main+=''' ];println!("[");for(n,(op,args))in cases.iter().enumerate(){let v=observe(op,args);println!("  {:?}{}",v,if n+1==cases.len(){""}else{","});}println!("]");}
'''
for p,text in {HERE/'src/pinned.rs':pinned,HERE/'src/main.rs':main,HERE/'cases.json':json.dumps(cases,indent=2)+'\n'}.items():
 if '--check' in sys.argv:
  if not p.exists() or p.read_text()!=text:raise SystemExit('source/case artifact drift: '+str(p))
 else:p.write_text(text)
print('Exact pure fragments generated; cases:',len(cases))
