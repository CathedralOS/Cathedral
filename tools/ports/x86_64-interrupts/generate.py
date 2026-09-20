#!/usr/bin/env python3
"""Pinned interrupt facts and complete fixed-record little-endian codecs."""
from pathlib import Path
import ast,importlib.util,json,re,sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];CHECK='--check' in sys.argv
PIN='cc35c876d3badb57df54a66e22f7768a52be95f2';SOURCE='src/structures/idt.rs'
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
shared.snapshot(ROOT/'reference_code/rust-osdev/x86_64',PIN,[SOURCE],'https://github.com/rust-osdev/x86_64')
def emit(path,text):
 if CHECK:
  if not path.exists() or path.read_text()!=text:raise SystemExit('generated artifact differs: '+str(path))
 else:path.write_text(text)
s=(ROOT/'reference_code/rust-osdev/x86_64'/SOURCE).read_text();part=s[s.index('pub struct PageFaultErrorCode:'):s.index('/// Describes an error code referencing')]
flags=[]
for m in re.finditer(r'const (\w+) = (.*?);',part):
 name,expr=m.groups();tree=ast.parse(expr,mode='eval')
 def val(n):
  if isinstance(n,ast.Expression):return val(n.body)
  if isinstance(n,ast.Constant):return n.value
  if isinstance(n,ast.BinOp) and isinstance(n.op,ast.LShift):return val(n.left)<<val(n.right)
  raise ValueError(expr)
 flags.append({'name':name,'value':val(tree),'line':s[:s.index(part)+m.start()].count('\n')+1})
records={
 'EntryOptions':{'size':4,'align':2,'fields':[('selector','SegmentSelector',0),('bits','u16',2)]},
 'RawInterruptStackFrame':{'size':40,'align':8,'fields':[('instruction_pointer','u64',0),('code_segment','SegmentSelector',8),('reserved_1','[u8; 6]',10),('cpu_flags','RFlags',16),('stack_pointer','u64',24),('stack_segment','SegmentSelector',32),('reserved_2','[u8; 6]',34)]},
}
header=['// SPDX-License-Identifier: MIT OR Apache-2.0','// Copyright 2017 Philipp Oppermann. See the README.md (upstream idt.rs).','// Modified inert IDT/frame translation at '+PIN+'. See x86_interrupts.PORT.md.']
facts=header+['module x86_interrupts;','use x86_registers::SegmentSelector;','use x86_registers::RFlags;','use x86_idt_gate;','']
for name,row in records.items():facts.append('pub data '+name+' [copy] { '+' '.join(f'{field}: {ty};' for field,ty,_ in row['fields'])+' }')
facts+=['pub data PageFaultErrorCode [copy] { raw: u64; }','pub data DetachedIdt [copy] { entries: [X86IdtGate; 256]; }','']
for f in flags:facts.append(f'pub const PAGE_FAULT_{f["name"]}: PageFaultErrorCode = PageFaultErrorCode {{ raw: {f["value"]} }};')
facts+=['pub const PAGE_FAULT_KNOWN_BITS: u64 = '+str(sum(f['value'] for f in flags))+';','pub const IDT_ENTRY_COUNT: u16 = 256;','pub const IDT_BYTE_LENGTH: u16 = 4096;','pub const IDT_BYTE_LIMIT: u16 = 4095;','']
emit(ROOT/'source/drivers/facts/x86_interrupts.omg','\n'.join(facts))
plans=header+['module interrupt_frame_layouts;','use omega::language::core::layout;','']
for name,row in records.items():
 plans+=['pub data '+name+'Layout {}','pub '+name+'Policy: '+name+'Layout satisfies Layout;','pub machine '+name+'Layout::plan(schema: Schema) -> Plan satisfies Layout::plan {','    let mut entries: [FieldEntry; 64];']
 for i,(_,_,offset) in enumerate(row['fields']):plans.append(f'    entries[{i}] = FieldEntry {{ key: schema.fields[{i}].key, placement: FieldPlan::At {{ offset: {offset} }} }};')
 plans+=['    Plan { entries: entries, entry_count: '+str(len(row['fields']))+', size_fixed: '+str(row['size'])+', size_is_dynamic: false, align: '+str(row['align'])+' }','}','']
emit(ROOT/'source/drivers/facts/interrupt_frame_layouts.omg','\n'.join(plans))
code=header+['module interrupt_bytes;','use facts::x86_interrupts::EntryOptions;','use facts::x86_interrupts::RawInterruptStackFrame;','use facts::x86_registers::SegmentSelector;','use facts::x86_registers::RFlags;','']
for name,row in records.items():
 fields=[]
 for field,ty,offset in row['fields']:
  if ty.startswith('['):fields +=[(f'{field}[{i}]','u8',offset+i) for i in range(6)]
  elif ty in ('SegmentSelector','RFlags'):fields.append((field+'.raw','u16' if ty=='SegmentSelector' else 'u64',offset))
  else:fields.append((field,ty,offset))
 code +=[f'pub machine encode_{name}(value: &{name}) -> [u8; {row["size"]}] {{',f'    let mut output: [u8; {row["size"]}];']
 for field,ty,offset in fields:
  for b in range(int(ty[1:])//8):code.append(f'    output[{offset+b}] = ((value.{field} >> {b*8}) & 255) as u8;')
 code+=['    output','}',f'pub machine decode_{name}(input: &[u8; {row["size"]}]) -> {name} {{']
 values={}
 for field,ty,offset in fields:values[field]=' | '.join(f'((input[{offset+b}] as {ty}) << {b*8})' for b in range(int(ty[1:])//8))
 for field,ty,_ in row['fields']:
  if ty.startswith('['):expr='['+', '.join(values[f'{field}[{i}]'] for i in range(6))+']'
  elif ty in ('SegmentSelector','RFlags'):expr=ty+' { raw: '+values[field+'.raw']+' }'
  else:expr=values[field]
  code.append(f'    let {field}: {ty} = {expr};')
 code+=['    '+name+' { '+', '.join(field+': '+field for field,_,_ in row['fields'])+' }','}','']
emit(ROOT/'source/libraries/x86_64/interrupt_bytes.omg','\n'.join(code)+'\n'+(HERE/'gate_codecs.template.omg').read_text())
enum=s[s.index('pub enum ExceptionVector {'):s.index('/// Exception vector number is invalid')]
vectors=[{'name':m[1],'value':int(m[2],0),'line':s[:s.index(enum)+m.start()].count('\n')+1} for m in re.finditer(r'    (\w+) = (0x[0-9A-Fa-f]+),',enum)]
emit(HERE/'schema.json',json.dumps({'revision':PIN,'source':SOURCE,'flags':flags,'records':records,'exception_vectors':vectors},indent=2)+'\n')
print(f'{len(flags)} page-fault flags,{len(vectors)} exception identities,2 raw fixed records/codec pairs; existing gate reused')
