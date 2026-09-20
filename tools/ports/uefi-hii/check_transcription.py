#!/usr/bin/env python3
"""Compare inert HII source and authored plans with pinned UEFI-target vectors."""
import json
from pathlib import Path
import re
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
RAW=ROOT/'source/contracts/uefi/raw'
vectors=json.loads((RAW/'hii.vectors.json').read_text())['measurements']
schema=json.loads((HERE/'schema.json').read_text())
records={};bykey={(r['module'],r['name']):r for r in schema};sources={}
for module in sorted({r['module'] for r in schema}):
    text=(RAW/(module+'.omg')).read_text();sources[module]=text
    for m in re.finditer(r'pub data (\w+) \[copy\] \{(.*?)\}',text,re.S):
        body=re.sub(r'//[^\n]*','',m[2]);fields=re.findall(r'(\w+):\s*([^;\n]+(?:;\s*\d+\])?)\s*;',body)
        records[(module,m[1])]=[(f,t.strip()) for f,t in fields]
assert set(records)==set(bykey),'declaration coverage mismatch'
cache={}
def geometry(module,typ):
    if typ in {'addr','u64','i64'}:return 8,8
    if typ in {'u32','i32'}:return 4,4
    if typ in {'u16','i16'}:return 2,2
    if typ in {'u8','i8'}:return 1,1
    if typ=='scalars::Guid':return 16,4
    if typ=='console::GraphicsOutputBltPixel':return 4,1
    if typ.startswith('['):
        t,n=typ[1:-1].split(';');size,align=geometry(module,t.strip());return size*int(n),align
    if '::' in typ:module,typ=typ.split('::')
    key=(module,typ)
    if key in cache:return cache[key]
    row=bykey[key];prefix=module+'::'+typ;position=0;maximum=1
    assert records[key]==[(f,t) for f,t,original in row['fields']],(key,'field mapping')
    for f,t,original in row['fields']:
        size,align=geometry(module,t)
        if row['packed']:align=1
        maximum=max(maximum,align);position=(position+align-1)//align*align
        vk=prefix+'.'+original+'.offset'
        if vk in vectors:assert vectors[vk]['value']==position,(vk,position,vectors[vk])
        position+=size
    # Backing bytes intentionally have no union members. Their explicit plan
    # carries the measured alignment; this never verifies typed union semantics.
    if row['union_storage']:maximum=vectors[prefix+'.alignment']['value']
    size=(position+maximum-1)//maximum*maximum
    cache[key]=size,maximum
    for kind,actual in [('size',size),('alignment',maximum)]:assert vectors[prefix+'.'+kind]['value']==actual,(prefix,kind,actual,vectors[prefix+'.'+kind])
    for tail in row['tail_fields']:assert vectors[prefix+'.'+tail+'.offset']['value']==position,(prefix,tail,'tail begins before final padding')
    return size,maximum
for key in records:geometry(*key)
count=0
for mod,text in sources.items():
    for m in re.finditer(r'pub const (\w+): [^=]+ = (.*);',text):
        key=mod+'::'+m[1];row=vectors[key];expr=m[2]
        if row['kind']=='value':
            raw=re.search(r'raw:\s*(\d+)',expr);value=int(raw[1] if raw else expr)
            assert value==row['value'],key
        else:
            literals=re.findall(r'0x([0-9a-f]+)',expr);assert len(literals)==11,key
            actual=b''.join(int(v,16).to_bytes(n,'little') for v,n in zip(literals[:3],[4,2,2]))+bytes(int(v,16) for v in literals[3:])
            assert actual.hex()==row['value'],key
        count+=1
plans=(RAW/'hii_layouts.omg').read_text()
for row in schema:
    prefix=row['module']+'::'+row['name']
    body=re.search(r'pub machine '+row['name']+r'X64Layout::plan\(schema: Schema\) -> Plan satisfies Layout::plan \{(.*?)\n\}',plans,re.S)[1]
    found=re.findall(r'entries\[(\d+)\] = FieldEntry \{ key: schema.fields\[(\d+)\].key, placement: FieldPlan::At \{ offset: (\d+) \}',body)
    expected=[(str(i),str(i),str(vectors.get(prefix+'.'+original+'.offset',{'value':0})['value'])) for i,(f,t,original) in enumerate(row['fields'])]
    assert found==expected,(prefix,'policy offsets')
    for spelling,kind in [('entry_count',None),('size_fixed','size'),('align','alignment')]:
        expected=len(row['fields']) if kind is None else vectors[prefix+'.'+kind]['value']
        assert int(re.search(spelling+r': (\d+)',body)[1])==expected,(prefix,spelling)
print(f'{len(records)} inert carriers/fixed plans and {count} constants agree with Rust-target geometry/values; no typed union or Omega ABI claim')
