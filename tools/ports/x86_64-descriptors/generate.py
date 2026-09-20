#!/usr/bin/env python3
"""Reproduce descriptor facts, requested plans and literal pure byte codecs."""
from pathlib import Path
import ast, importlib.util, json, re, sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];CHECK='--check' in sys.argv
PIN='cc35c876d3badb57df54a66e22f7768a52be95f2';FILES=['src/structures/'+p+'.rs' for p in ('mod','tss','gdt','idt')]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
shared.snapshot(ROOT/'reference_code/rust-osdev/x86_64',PIN,FILES,'https://github.com/rust-osdev/x86_64')
def emit(path,text):
 if CHECK:
  if not path.exists() or path.read_text()!=text:raise SystemExit('generated artifact differs: '+str(path))
 else:path.write_text(text)
source=(ROOT/'reference_code/rust-osdev/x86_64/src/structures/gdt.rs').read_text();known={};constants=[]
body=source[source.index('pub struct DescriptorFlags:'):source.index('impl Descriptor {')]
for match in re.finditer(r'(?:pub )?const (\w+)\s*:\s*Self\s*=\s*(.*?);|const (\w+)\s*=\s*(.*?);',body,re.S):
 name,expr=match.group(1,2) if match.group(1) else match.group(3,4)
 expr=re.sub(r'Self::(\w+)\.bits\(\)',lambda m:str(known[m[1]]),expr)
 expr=expr.replace('Self::from_bits_truncate(','(').replace('_','').strip().replace(',\n', '\n')
 tree=ast.parse(expr,mode='eval')
 def value(n):
  if isinstance(n,ast.Expression):return value(n.body)
  if isinstance(n,ast.Constant) and type(n.value)==int:return n.value
  if isinstance(n,ast.BinOp) and isinstance(n.op,ast.BitOr):return value(n.left)|value(n.right)
  if isinstance(n,ast.BinOp) and isinstance(n.op,ast.LShift):return value(n.left)<<value(n.right)
  raise ValueError(expr)
 raw=value(tree);known[name]=raw;constants.append({'name':name,'value':raw,'line':source[:source.index(body)+match.start()].count('\n')+1,'public':name!='COMMON'})
records={
'DescriptorTablePointer':{'size':10,'align':2,'fields':[('limit','u16',0),('base','u64',2)]},
'TaskStateSegment':{'size':104,'align':4,'fields':[('reserved_1','u32',0),('privilege_stack_table','[u64; 3]',4),('reserved_2','u64',28),('interrupt_stack_table','[u64; 7]',36),('reserved_3','u64',92),('reserved_4','u16',100),('iomap_base','u16',102)]},
'GdtEntry':{'size':8,'align':8,'fields':[('raw','u64',0)]},
'TssDescriptorWords':{'size':16,'align':8,'fields':[('low','u64',0),('high','u64',8)]},
}
head=['// SPDX-License-Identifier: MIT OR Apache-2.0','// Modified fixed x86_64 descriptor facts at '+PIN+'.','// See x86_descriptors.PORT.md. Addresses are numeric and convey no backing.','// Copyright 2017 Philipp Oppermann. See the README.md (upstream idt.rs).','module x86_descriptors;','']
for name,row in records.items():head.append('pub data '+name+' [copy] { '+' '.join(f'{field}: {ty};' for field,ty,_ in row['fields'])+' }')
head+=['pub data DescriptorFlags [copy] { raw: u64; }','pub data Descriptor [copy] { case UserSegment(low: u64); case SystemSegment(low: u64, high: u64); }','pub data SelectorErrorCode [copy] { raw: u64; }','']
for c in constants:
 raw=str(c['value'])
 if c['value']>9223372036854775807:
  raw='LARGE_DESCRIPTOR_'+c['name'];head.append(f'const {raw}: u64 = ({c["value"] >> 32} as u64) << 32 | {c["value"] & 4294967295};')
 head.append(f'pub const DESCRIPTOR_{c["name"]}: DescriptorFlags = DescriptorFlags {{ raw: {raw} }};')
head+=['pub const DESCRIPTOR_TABLE_GDT: u8 = 0;','pub const DESCRIPTOR_TABLE_IDT: u8 = 1;','pub const DESCRIPTOR_TABLE_LDT: u8 = 2;','pub const TSS_SIZE: u16 = 104;','pub const TSS_MIN_LIMIT: u16 = 103;','pub const IO_MAP_MAX_DISTANCE: u16 = 57343;','pub const IO_MAP_MAX_LENGTH: u16 = 8193;','pub const IO_MAP_OK: u8 = 0;','pub const IO_MAP_BEFORE_TSS: u8 = 1;','pub const IO_MAP_TOO_FAR: u8 = 2;','pub const IO_MAP_BAD_TERMINATOR: u8 = 3;','pub const IO_MAP_TOO_LONG: u8 = 4;','pub const IO_MAP_INVALID_BASE: u8 = 5;','']
emit(ROOT/'source/drivers/facts/x86_descriptors.omg','\n'.join(head))
plans=['// SPDX-License-Identifier: MIT OR Apache-2.0','// Requested hardware geometry; not an observation or authority grant.','module x86_descriptor_layouts;','use omega::language::core::layout;','']
for name,row in records.items():
 plans+=['pub data '+name+'Layout {}','pub '+name+'Policy: '+name+'Layout satisfies Layout;','pub machine '+name+'Layout::plan(schema: Schema) -> Plan satisfies Layout::plan {','    let mut entries: [FieldEntry; 64];']
 entry=0
 for i,(_,ty,offset) in enumerate(row['fields']):
  if ty.startswith('u') and offset % (int(ty[1:])//8):
   width=16 if row['align']==2 else 32
   for bit in range(0,int(ty[1:]),width):
    plans.append(f'    entries[{entry}] = FieldEntry {{ key: schema.fields[{i}].key, placement: FieldPlan::Bits {{ container: {offset+bit//8}, container_width: {width}, destination_lsb: 0, source_lsb: {bit}, width: {width} }} }};');entry+=1
  else:
   plans.append(f'    entries[{entry}] = FieldEntry {{ key: schema.fields[{i}].key, placement: FieldPlan::At {{ offset: {offset} }} }};');entry+=1
 plans+=['    Plan { entries: entries, entry_count: '+str(entry)+', size_fixed: '+str(row['size'])+', size_is_dynamic: false, align: '+str(row['align'])+' }','}','']
emit(ROOT/'source/drivers/facts/x86_descriptor_layouts.omg','\n'.join(plans))
code=['// SPDX-License-Identifier: MIT OR Apache-2.0','// Literal little-endian codecs; no casts to pointers or inferred native ABI.','module descriptor_bytes;','use facts::x86_descriptors;','']
for name,row in records.items():
 fields=[]
 for field,ty,offset in row['fields']:
  if ty.startswith('['):
   count=int(re.search(r'; (\d+)',ty)[1]);fields +=[(f'{field}[{i}]','u64',offset+i*8) for i in range(count)]
  else:fields.append((field,ty,offset))
 code +=[f'pub machine encode_{name}(value: &x86_descriptors::{name}) -> [u8; {row["size"]}] {{',f'    let mut output: [u8; {row["size"]}];']
 for field,ty,offset in fields:
  for b in range(int(ty[1:])//8):code.append(f'    output[{offset+b}] = ((value.{field} >> {b*8}) & 255) as u8;')
 code+=['    output','}',f'pub machine decode_{name}(input: &[u8; {row["size"]}]) -> x86_descriptors::{name} {{']
 values={}
 for field,ty,offset in fields:
  expr=' | '.join(f'((input[{offset+b}] as {ty}) << {b*8})' for b in range(int(ty[1:])//8));values[field]=expr
 for field,ty,_ in row['fields']:
  expr='['+', '.join(values[f'{field}[{i}]'] for i in range(int(re.search(r'; (\d+)',ty)[1])))+']' if ty.startswith('[') else values[field]
  code.append(f'    let {field}: {ty} = {expr};')
 code+=['    x86_descriptors::'+name+' { '+', '.join(field+': '+field for field,_,_ in row['fields'])+' }','}','']
emit(ROOT/'source/libraries/x86_64/descriptor_bytes.omg','\n'.join(code))
emit(HERE/'schema.json',json.dumps({'revision':PIN,'files':FILES,'constants':constants,'records':records},indent=2)+'\n')
print(f'{len(constants)} descriptor constants; {len(records)} requested plans and byte-codec pairs')
