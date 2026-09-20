#!/usr/bin/env python3
"""Pinned register fact transcription and upstream reference/vector scaffolding."""
from pathlib import Path
import ast
import sys
import importlib.util
import json
import re
ROOT=Path(__file__).resolve().parents[3];HERE=Path(__file__).resolve().parent
CHECK='--check' in sys.argv
def emit(path,text):
 if CHECK:
  if not path.exists() or path.read_text()!=text:raise SystemExit('generated artifact differs: '+str(path))
 else:path.write_text(text)
PIN='cc35c876d3badb57df54a66e22f7768a52be95f2'
CHECKOUT=ROOT/'reference_code/rust-osdev/x86_64'
spec=importlib.util.spec_from_file_location('port_inventory',ROOT/'tools/ports/inventory.py');shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
FILES=['src/lib.rs']+['src/registers/'+x+'.rs' for x in ('control','debug','model_specific','mxcsr','rflags','segmentation','xcontrol')]
shared.snapshot(CHECKOUT,PIN,FILES,'https://github.com/rust-osdev/x86_64')
def prefix(name):return re.sub(r'([a-z0-9])([A-Z])',r'\1_\2',name).upper()
def evaluate(expression,known):
 expression=re.sub(r'Self::(\w+)\.bits\(\)',lambda m:str(known[m.group(1)]),expression).replace('_','')
 tree=ast.parse(expression.strip(),mode='eval')
 def value(n):
  if isinstance(n,ast.Expression):return value(n.body)
  if isinstance(n,ast.Constant) and type(n.value) is int:return n.value
  if isinstance(n,ast.BinOp):
   a,b=value(n.left),value(n.right)
   if isinstance(n.op,ast.BitOr):return a|b
   if isinstance(n.op,ast.LShift):return a<<b
  raise ValueError('unsupported pinned expression '+expression)
 return value(tree)
flags=[];constants=[]
for path in FILES:
 source=(CHECKOUT/path).read_text();module=Path(path).stem
 for match in re.finditer(r'pub struct (\w+): (u\d+)\s*\{(.*?)\n    \}',source,re.S):
  name,width,body=match.groups();known={};members=[]
  for member in re.finditer(r'const (\w+)\s*=\s*(.*?);',body,re.S):
   key,expression=member.groups();raw=evaluate(expression,known);known[key]=raw
   item={'name':prefix(name)+'_'+key,'type':name,'width':width,'value':raw,'path':path,'line':source[:match.start(3)+member.start()].count('\n')+1,'rust':f'x86_64::registers::{module}::{name}::{key}.bits()','source_name':key}
   constants.append(item);members.append(item)
  flags.append({'name':name,'width':width,'members':members,'known_bits':__import__('functools').reduce(int.__or__,known.values(),0),'path':path})
# Explicit open code carriers preserve unknown input until validated; no CPU state.
enums={
'PrivilegeLevel':('u8',[('RING0',0),('RING1',1),('RING2',2),('RING3',3)]),
'PriorityClass':('u8',[(f'CLASS{i}',i) for i in range(1,16)]),
'DebugAddressRegisterNumber':('u8',[(f'DR{i}',i) for i in range(4)]),
'BreakpointCondition':('u8',[('INSTRUCTION_EXECUTION',0),('DATA_WRITES',1),('IO_READS_WRITES',2),('DATA_READS_WRITES',3)]),
'BreakpointSize':('u8',[('LENGTH1B',0),('LENGTH2B',1),('LENGTH8B',2),('LENGTH4B',3)]),
}
msrs=[];source=(CHECKOUT/'src/registers/model_specific.rs').read_text()
for match in re.finditer(r'impl (\w+)\s*\{\s*/// The underlying model specific register\.\s*pub const MSR: Msr = Msr\((0x[0-9A-Fa-f_]+)\);',source):
 name,literal=match.groups();msrs.append({'name':prefix(name)+'_MSR','owner':name,'value':int(literal.replace('_',''),16),'path':'src/registers/model_specific.rs','line':source[:source.index('pub const MSR',match.start())].count('\n')+1})
head=['// SPDX-License-Identifier: MIT OR Apache-2.0','// Modified x86_64 register translation at '+PIN+'.','// See x86_registers.PORT.md. Raw bits only; no CPU validity or authority.','module x86_registers;','use local_apic;','']
for flag in flags:
 head.append('pub data '+flag['name']+' [copy] { raw: '+flag['width']+'; }')
 for member in flag['members']:
  value=str(member['value'])
  if flag['name']=='ApicBaseFlags':
   bit={'BSP':'IA32_APIC_BASE_BSP_BIT','X2APIC_ENABLE':'IA32_APIC_BASE_X2APIC_ENABLE_BIT','LAPIC_ENABLE':'IA32_APIC_BASE_GLOBAL_ENABLE_BIT'}[member['source_name']]
   value='(1 as u64) << '+bit
  head.append('pub const '+member['name']+': '+flag['name']+' = '+flag['name']+' { raw: '+value+' };')
 head.append('pub const '+prefix(flag['name'])+'_KNOWN_BITS: '+flag['width']+' = '+str(flag['known_bits'])+';\n')
for name,(width,members) in enums.items():
 head.append('pub data '+name+' [copy] { raw: '+width+'; }')
 for key,value in members:head.append(f'pub const {prefix(name)}_{key}: {name} = {name} {{ raw: {value} }};')
 head.append('')
head+=['pub data SegmentSelector [copy] { raw: u16; }','pub const SEGMENT_SELECTOR_NULL: SegmentSelector = SegmentSelector { raw: 0 };','pub data Msr [copy] { raw: u32; }','pub data Dr7Bits [copy] { raw: u64; }','']
for msr in msrs:
 value='IA32_APIC_BASE_MSR' if msr['owner']=='ApicBase' else str(msr['value'])
 head.append(f'pub const {msr["name"]}: Msr = Msr {{ raw: {value} }};')
head+=['','pub const MXCSR_RESET_BITS: u32 = 8064;','pub const DR7_VALID_BITS: u64 = 4294913023;','']
# Validate the independent hand-derived mask against pinned flags.
dr7=next(f['known_bits'] for f in flags if f['name']=='Dr7Flags')|0xffff0000
assert dr7==4294913023,hex(dr7)
emit(ROOT/'source/drivers/facts/x86_registers.omg','\n'.join(head))
schema={'revision':PIN,'files':FILES,'flags':flags,'enums':enums,'msrs':msrs,'dr7_valid_bits':dr7}
emit(HERE/'schema.json',json.dumps(schema,indent=2)+'\n')
# Flags are public Rust observations; MSR one-field wrappers require explicit
# private-representation inspection in the host/target test, never target code.
measurements={}
for c in constants:measurements[c['name']]={'expression':c['rust'],'value':c['value']}
for m in msrs:measurements[m['name']]={'expression':'unsafe { core::mem::transmute::<x86_64::registers::model_specific::Msr, u32>(x86_64::registers::model_specific::'+m['owner']+'::MSR) }','value':m['value']}
for flag in flags:
 measurements[prefix(flag['name'])+'_KNOWN_BITS']={'expression':f"x86_64::registers::{Path(flag['path']).stem}::{flag['name']}::all().bits()",'value':flag['known_bits']}
enum_paths={'PrivilegeLevel':'x86_64::PrivilegeLevel','PriorityClass':'x86_64::registers::control::PriorityClass','DebugAddressRegisterNumber':'x86_64::registers::debug::DebugAddressRegisterNumber','BreakpointCondition':'x86_64::registers::debug::BreakpointCondition','BreakpointSize':'x86_64::registers::debug::BreakpointSize'}
variant_names={'BreakpointCondition':['InstructionExecution','DataWrites','IoReadsWrites','DataReadsWrites'],'BreakpointSize':['Length1B','Length2B','Length8B','Length4B']}
for name,(_,members) in enums.items():
 for i,(key,value) in enumerate(members):
  variant=variant_names[name][i] if name in variant_names else 'Ring'+str(i) if name=='PrivilegeLevel' else 'PriorityClass'+str(i+1) if name=='PriorityClass' else 'Dr'+str(i)
  measurements[prefix(name)+'_'+key]={'expression':enum_paths[name]+'::'+variant+' as u8','value':value}
measurements['SEGMENT_SELECTOR_NULL']={'expression':'x86_64::registers::segmentation::SegmentSelector::NULL.0','value':0}
measurements['DR7_VALID_BITS']={'expression':'x86_64::registers::debug::Dr7Value::from_bits_truncate(u64::MAX).bits()','value':dr7}
measurements['MXCSR_RESET_BITS']={'expression':'x86_64::registers::mxcsr::MxCsr::default().bits()','value':8064}
probe=['#[cfg(test)] mod tests;','fn main() {']
for name,row in measurements.items():probe.append('    println!("'+name+'={}", '+row['expression']+');')
probe+=['}'];emit(HERE/'src/main.rs','\n'.join(probe)+'\n')
emit(HERE/'measurements.json',json.dumps(measurements,indent=2)+'\n')
print(f'{len(flags)} flag carriers, {len(constants)} flag constants, {len(msrs)} MSR identities generated')
