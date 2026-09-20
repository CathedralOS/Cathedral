#!/usr/bin/env python3
from pathlib import Path
import json,sys,re
HERE=Path(__file__).resolve().parent
schema=json.loads((HERE/'schema.json').read_text())
lines=['// SPDX-License-Identifier: MIT OR Apache-2.0','// Independent literal expectations; evaluate actual translated machine bodies.','module cases;','use facts::x86_descriptors;','use facts::x86_descriptors::TaskStateSegment;','use facts::x86_descriptors::DescriptorTablePointer;','use facts::x86_descriptors::GdtEntry;','use facts::x86_descriptors::TssDescriptorWords;','use facts::x86_descriptors::SelectorErrorCode;','use x86_values::descriptors;','use x86_values::descriptor_bytes;']
def groups(label,checks):
 names=[]
 for start in range(0,len(checks),12):
  name=label+'_'+str(start//12);names.append(name+'()');lines.append('machine '+name+'() -> bool {')
  comparisons=[]
  for i,check in enumerate(checks[start:start+12]):
   if ' == ' in check:
    left,right=check.split(' == ',1);lines.append(f'let actual{i}: u64 = {left};');lines.append(f'let expected{i}: u64 = {right};');comparisons.append(f'actual{i} == expected{i}')
   else:comparisons.append(check)
  lines.append(' &&\n'.join(comparisons)+' }')
 lines.append('pub machine '+label+'() -> bool { '+' && '.join(names)+' }')
groups('constants_match',[f'x86_descriptors::DESCRIPTOR_{c["name"]}.raw == {c["value"]}' for c in schema['constants']])
checks=[]
for base in [0,1,0x1122334455667788,0xffffffffffffffff]:
 for limit in [0,103,104,65535]:
  low=limit|((base&0xffffff)<<16)|(((base>>24)&255)<<56)|0x890000000000;high=base>>32
  checks.extend([f'descriptors::tss_descriptor({base}, {limit}).low == {low}',f'descriptors::tss_descriptor({base}, {limit}).high == {high}'])
groups('descriptor_cases',checks)
checks=['!descriptors::selector_error_valid(65536)','!descriptors::selector_error_valid(0xffffffffffffffff)','descriptors::selector_error_truncate(0xffffffffffffffff).raw == 65535']
for raw in [0,1,2,3,4,5,6,7,8,9,65528,65529,65530,65531,65532,65533,65534,65535]:
 name='error'+str(raw);lines.append(f'machine {name}() -> bool {{ let value: SelectorErrorCode = SelectorErrorCode {{ raw: {raw} }}; '+f'descriptors::selector_error_valid({raw}) && descriptors::selector_error_external(&value) == '+str(bool(raw&1)).lower()+f' && descriptors::selector_error_table(&value) == '+str({0:0,1:1,2:2,3:1}[(raw>>1)&3])+f' && descriptors::selector_error_index(&value) == {raw>>3} && descriptors::selector_error_is_null(&value) == '+str(raw==0).lower()+' }');checks.append(name+'()')
groups('error_cases',checks)
# Every field/byte gets a distinguishable initialized value, including reserved
# fields; codecs preserve raw bits rather than falsely validate a live TSS.
names=[]
for name,row in schema['records'].items():
 field_values={};expected=bytearray(row['size'])
 for field,ty,offset in row['fields']:
  if ty.startswith('['):
   n=int(ty.split(';')[1].strip(' ]'));values=[0x0102030405060708+i*0x0808080808080808 for i in range(n)];field_values[field]='['+', '.join(map(str,values))+']'
   for i,v in enumerate(values):expected[offset+i*8:offset+(i+1)*8]=v.to_bytes(8,'little')
  else:
   width=int(ty[1:])//8;v=int.from_bytes(bytes(range(offset+1,offset+1+width)),'little');field_values[field]=str(v);expected[offset:offset+width]=v.to_bytes(width,'little')
 fn='bytes_'+name;names.append(fn+'()');lines.append('machine '+fn+'() -> bool {')
 lines.append(f' let value: {name} = {name} {{ '+', '.join(f+': '+v for f,v in field_values.items())+' };')
 lines.append(f' let encoded: [u8; {row["size"]}] = descriptor_bytes::encode_{name}(&value);')
 lines.append(f' let golden: [u8; {row["size"]}] = ['+', '.join(map(str,expected))+'];')
 lines.append(f' let decoded: {name} = descriptor_bytes::decode_{name}(&golden);')
 tests=[f'encoded[{i}] == {b}' for i,b in enumerate(expected)]
 for field,ty,offset in row['fields']:
  if ty.startswith('['):tests +=[f'decoded.{field}[{i}] == value.{field}[{i}]' for i in range(int(ty.split(';')[1].strip(' ]')))]
  else:tests.append(f'decoded.{field} == value.{field}')
 lines +=[' &&\n'.join(tests),'}']
lines.append('machine tss_default() -> bool { let value: TaskStateSegment = descriptors::tss_new(); value.iomap_base == 104 && value.reserved_1 == 0 && value.reserved_2 == 0 && value.reserved_3 == 0 && value.reserved_4 == 0 && '+ ' && '.join(f'value.{name}[{i}] == 0' for name,count in [('privilege_stack_table',3),('interrupt_stack_table',7)] for i in range(count))+' }')
lines.append('machine descriptor_roundtrip() -> bool { let value: TssDescriptorWords = descriptors::tss_descriptor(0x1122334455667788, 103); let bad: TssDescriptorWords = TssDescriptorWords { low: 0x890000000067, high: 0x100000000 }; descriptors::tss_descriptor_base(&value) == 0x1122334455667788 && descriptors::tss_descriptor_limit(&value) == 103 && descriptors::tss_descriptor_shape_valid(&value) && !descriptors::tss_descriptor_shape_valid(&bad) }')
lines.append('pub machine byte_cases() -> bool { '+' && '.join(names+['tss_default()','descriptor_roundtrip()'])+' }')
text='\n'.join(lines)+'\n'
large=sorted({m.group() for m in re.finditer(r'\b[0-9]{19,}\b',text) if int(m.group()) > 9223372036854775807})
for i,value in enumerate(large):text=re.sub(r'\b'+value+r'\b','LARGE_'+str(i),text)
text += ''.join('const LARGE_'+str(i)+': u64 = '+'('+str(int(value)>>32)+' as u64) << 32 | '+str(int(value)&4294967295)+';\n' for i,value in enumerate(large))
p=HERE/'cases.omg'
if '--check' in sys.argv:
 if not p.exists() or p.read_text()!=text:raise SystemExit('generated fixture differs')
else:p.write_text(text)
print('22 flag comparisons,16 TSS descriptor inputs,18 selector errors,4 complete byte codecs')

probe=(HERE/'main.omg').read_text()
for i,name in enumerate(['DOUBLE_FAULT','NMI','MACHINE_CHECK','MASKABLE_IRQ'],1):
 probe=probe.replace('= X86IstStackClass { stack_class: '+str(i)+', ist_index: '+str(i)+' };','= X86_'+name+'_STACK;')
probe='// Reproducer: positive fixture with actual imported legacy aggregate constants.\n'+probe
p=HERE/'legacy_ist_constant_probe.omg'
if '--check' in sys.argv:
 if not p.exists() or p.read_text()!=probe:raise SystemExit('legacy IST probe differs')
else:p.write_text(probe)
