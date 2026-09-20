#!/usr/bin/env python3
"""Observe actual pinned Rust IDT/frame values and assert target geometry."""
from pathlib import Path
import argparse,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
p=argparse.ArgumentParser();p.add_argument('--write',action='store_true');args=p.parse_args();schema=json.loads((HERE/'schema.json').read_text());rows={}
for f in schema['flags']:rows['PAGE_FAULT_'+f['name']]={'kind':'value','value':f['value'],'expr':'PageFaultErrorCode::'+f['name']+'.bits()'}
for v in schema['exception_vectors']:rows['EXCEPTION_'+v['name']]={'kind':'value','value':v['value'],'expr':'ExceptionVector::'+v['name']+' as u8'}
for typ,size,align in [('Entry<HandlerFunc>',16,4),('EntryOptions',4,2),('InterruptDescriptorTable',4096,16),('InterruptStackFrameValue',40,8),('InterruptStackFrame',40,8),('PageFaultErrorCode',8,8)]:
 rows[typ+'.size']={'kind':'size','value':size,'expr':'size_of::<'+typ+'>()'};rows[typ+'.alignment']={'kind':'alignment','value':align,'expr':'align_of::<'+typ+'>()'}
for field,offset in [('instruction_pointer',0),('code_segment',8),('cpu_flags',16),('stack_pointer',24),('stack_segment',32)]:rows['InterruptStackFrameValue.'+field+'.offset']={'kind':'offset','value':offset,'expr':'offset_of!(InterruptStackFrameValue, '+field+')'}
# Each public exception entry names its real byte slot. Reserved private fields
# are covered by source hash/specification, not fabricated public offset access.
s=(ROOT/'reference_code/rust-osdev/x86_64'/schema['source']).read_text();start=s.index('pub struct InterruptDescriptorTable {');end=s.index('impl InterruptDescriptorTable {')
import re
slots={'divide_error':0,'debug':1,'non_maskable_interrupt':2,'breakpoint':3,'overflow':4,'bound_range_exceeded':5,'invalid_opcode':6,'device_not_available':7,'double_fault':8,'invalid_tss':10,'segment_not_present':11,'stack_segment_fault':12,'general_protection_fault':13,'page_fault':14,'x87_floating_point':16,'alignment_check':17,'machine_check':18,'simd_floating_point':19,'virtualization':20,'cp_protection_exception':21,'hv_injection_exception':28,'vmm_communication_exception':29,'security_exception':30}
actual_names=set(re.findall(r'pub (\w+): Entry<',s[start:end]));assert actual_names==set(slots),(actual_names,set(slots))
for field,index in slots.items():rows['InterruptDescriptorTable.'+field+'.offset']={'kind':'offset','value':index*16,'expr':'offset_of!(InterruptDescriptorTable, '+field+')'}
imports='use core::mem::{size_of,align_of,offset_of};\nuse x86_64::structures::idt::*;\n'
source=['#[cfg(test)] mod tests;',imports,'fn main() {']
for name,row in rows.items():source.append('println!("'+name+'={}",'+row['expr']+');')
source+=['}'];(HERE/'src/main.rs').write_text('\n'.join(source)+'\n')
(HERE/'src/lib.rs').write_text('#![no_std]\n'+imports+'\n'.join('const _: () = assert!(('+row['expr']+') as u64 == '+str(row['value'])+');' for row in rows.values())+'\n')
if not (HERE/'Cargo.lock').exists():subprocess.run(['cargo','generate-lockfile','--offline','--manifest-path',str(HERE/'Cargo.toml')],cwd=ROOT,check=True)
output=subprocess.check_output(['cargo','run','--locked','--quiet','--manifest-path',str(HERE/'Cargo.toml')],cwd=ROOT,text=True)
actual={k:int(v) for k,v in (line.split('=',1) for line in output.splitlines())};assert actual=={k:r['value'] for k,r in rows.items()},(actual,rows)
subprocess.run(['cargo','check','--locked','--quiet','--lib','--manifest-path',str(HERE/'Cargo.toml'),'--target','x86_64-unknown-uefi'],cwd=ROOT,check=True)
vector={'format':'cathedral-port-vectors-v1','target':{'pointer_bits':64,'endian':'little','abi':'pinned Rust UEFI-x64 IDT/frame ABI; Cathedral gate policy alignment16 deliberately distinct'},'provenance':{'kind':'upstream','revision':schema['revision'],'sources':[schema['source']],'description':'Actual Rust host values/size/alignment/public offsets, each independently const-asserted for UEFI x64. Rust Entry alignment4 differs from Cathedral requested gate alignment16. Native Omega layout not observed.'},'measurements':{k:{'kind':r['kind'],'value':actual[k]} for k,r in rows.items()}}
p=ROOT/'source/drivers/facts/x86_interrupts.vectors.json'
if args.write:p.write_text(json.dumps(vector,indent=2)+'\n')
elif not p.exists() or json.loads(p.read_text())!=vector:raise SystemExit('vectors differ; review before --write')
print('PASS',len(rows),'actual Rust observations and UEFI-x64 const assertions')
