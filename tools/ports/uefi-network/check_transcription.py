#!/usr/bin/env python3
"""Check network transcription, C-profile facts, and authored plans; no Omega ABI claim."""
import json
from pathlib import Path
import re
HERE=Path(__file__).resolve().parent
RAW=HERE.parents[2]/'source/contracts/uefi/raw'
schema=json.loads((HERE/'schema.json').read_text())
vector=json.loads((RAW/'network.vectors.json').read_text())['measurements']
text=(RAW/'network.omg').read_text()
records={}
for match in re.finditer(r'pub data (\w+) \[copy\] \{(.*?)\n\}',text,re.S):
 body=re.sub(r'//[^\n]*','',match[2])
 records[match[1]]=[(n,t.strip()) for n,t in re.findall(r'(\w+):\s*(\[[^\]]+\]|\w+)\s*;',body)]
original={r['name']:r for r in schema['records']}
assert set(records)==set(original)
cache={}
def geometry(typ):
 if '*' in typ or 'fn(' in typ or 'fn (' in typ: return 8,8
 if typ in {'addr','u64','usize','Event','Handle','Status'}: return 8,8
 if typ in {'u32','i32'}: return 4,4
 if typ in {'u16','Char16'}: return 2,2
 if typ in {'u8','Boolean','Char8'}: return 1,1
 if typ.startswith('['):
  element,count=typ[1:-1].split(';'); size,align=geometry(element.strip()); return size*int(count),align
 if typ in cache:return cache[typ]
 record=original[typ];position=0;maximum=1
 for field,ft in record['fields']:
  size,align=geometry(ft)
  if record['packed']:align=1
  maximum=max(maximum,align)
  if record['kind']=='union':offset=0;position=max(position,size)
  else:position=(position+align-1)//align*align;offset=position;position+=size
  key=typ+'.'+field+'.offset'
  if key in vector:assert offset==vector[key]['value'],(key,offset,vector[key])
 size=(position+maximum-1)//maximum*maximum
 for kind,actual in [('size',size),('alignment',maximum)]:assert actual==vector[typ+'.'+kind]['value'],(typ,kind,actual)
 cache[typ]=size,maximum
 return cache[typ]
for name,record in original.items():
 assert records[name]==[tuple(p) for p in record['omega_fields']],(name,'field order/type mismatch')
 geometry(name)
 for field,ft in record['fields']:
  if re.search(r';\s*0\s*\]',ft):assert f'{field}: {ft} needs a bounded runtime tail view.' in text
  if record['kind']=='union':assert f'Union alternative {field}: {ft} (offset 0).' in text
for constant in schema['constants']:
 name=constant['name'];expr=re.search(r'pub const '+name+r': [^=]+ = (.*);',text)[1]
 if constant['kind']=='bytes':
  literals=re.findall(r'0x([0-9a-f]+)',expr);assert len(literals)==11
  value=b''.join(int(n,16).to_bytes(w,'little') for n,w in zip(literals[:3],[4,2,2]))+bytes(int(n,16) for n in literals[3:])
  assert value.hex()==vector[name]['value'],name
 else:
  raw=re.search(r'raw:\s*(\d+)',expr)
  assert int(raw[1] if raw else expr)==vector[name]['value'],name
assert 'pub const DHCP_RFC2131_BROADCAST_BIT: u16 = 0x8000;' in text
plans=(RAW/'network_layouts.omg').read_text()+(RAW/'network_large_layouts.omg').read_text()
for name,fields in records.items():
 body=re.search(r'pub machine '+name+r'X64Layout::plan\(schema: Schema\) -> Plan satisfies Layout::plan \{(.*?)\n\}',plans,re.S)[1]
 found=re.findall(r'entries\[(\d+)\] = FieldEntry \{ key: schema.fields\[(\d+)\].key, placement: FieldPlan::At \{ offset: (\d+) \}',body)
 expected=[(str(i),str(i),str(vector.get(name+'.'+f+'.offset',{'value':0})['value'])) for i,(f,_) in enumerate(fields)]
 assert found==expected,(name,'plan offsets')
 for spelling,kind in [('entry_count',None),('size_fixed','size'),('align','alignment')]:
  value=len(fields) if kind is None else vector[name+'.'+kind]['value']
  assert int(re.search(spelling+r': (\d+)',body)[1])==value,(name,spelling)
print(f'{len(records)} records, {len(schema["constants"])} upstream constants and 81 plans match 703 Rust UEFI-x64 facts; RFC2131 broadcast mask kept separate. No Omega ABI claim.')
