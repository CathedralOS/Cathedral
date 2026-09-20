#!/usr/bin/env python3
"""Check source facts against Rust-target vectors; does not model Omega layout."""
import json
from pathlib import Path
import re
ROOT=Path(__file__).resolve().parents[3]
RAW=ROOT/'source/contracts/uefi/raw'
vector=json.loads((RAW/'console.vectors.json').read_text())['measurements']
schemas=json.loads(Path(__file__).with_name('schema.json').read_text())
records={};sources={}
for module in ['console','serial','loaded_image','device_path','load_file','shell_params']:
    text=(RAW/(module+'.omg')).read_text();sources[module]=text
    for match in re.finditer(r'pub data (\w+) \[copy\] \{(.*?)\}',text,re.S):
        body=re.sub(r'//[^\n]*','',match[2])
        fields=re.findall(r'(\w+):\s*([^;\n]+(?:;\s*\d+\])?)\s*;',body)
        records[(module,match[1])]=[(name,typ.strip()) for name,typ in fields]

# Independent C-profile geometry from transcribed scalar widths/order. This
# checks transcription against upstream; it cannot establish Omega home layout.
packed={(r['module'],r['name']) for r in schemas if r['packed']}
cache={}
def geometry(module,typ):
    if typ in {'addr','u64','i64'}:return 8,8
    if typ in {'u32','i32'}:return 4,4
    if typ in {'u16','i16'}:return 2,2
    if typ in {'u8','i8'}:return 1,1
    if typ=='scalars::Guid':return 16,4
    if typ.startswith('['):
        base,num=typ[1:-1].split(';');size,align=geometry(module,base.strip());return size*int(num),align
    key=(module,typ)
    if key in cache:return cache[key]
    position=0;maximum=1
    for field,ft in records[key]:
        size,align=geometry(module,ft)
        if key in packed:align=1
        maximum=max(maximum,align);position=(position+align-1)//align*align
        vk=module+'::'+typ+'.'+field+'.offset'
        if vk in vector:assert position==vector[vk]['value'],(vk,position,vector[vk])
        position+=size
    size=(position+maximum-1)//maximum*maximum
    cache[key]=size,maximum
    for kind,actual in [('size',size),('alignment',maximum)]:
        vk=module+'::'+typ+'.'+kind
        assert actual==vector[vk]['value'],(vk,actual,vector[vk])
    return size,maximum
for key in records:geometry(*key)
for schema in schemas:
    key=schema['module'],schema['name']
    assert records[key]==[tuple(p) for p in schema['fields']],('field mapping',key)
    for tail in schema['tail_fields']:
        assert vector[key[0]+'::'+key[1]+'.'+tail+'.offset']['value']==cache[key][0]

count=0
for module,text in sources.items():
    for m in re.finditer(r'pub const (\w+): [^=]+ = (.*);',text):
        key=module+'::'+m[1];row=vector[key];expr=m[2]
        if row['kind']=='value':
            assert int(re.search(r'raw:\s*(\d+)',expr)[1])==row['value'],key
        else:
            literals=re.findall(r'0x([0-9a-f]+)',expr)
            if literals:
                assert len(literals)==11,(key,literals)
                data=b''.join(int(v,16).to_bytes(width,'little') for v,width in zip(literals[:3],[4,2,2]))+bytes(int(v,16) for v in literals[3:])
                assert data.hex()==row['value'],key
            else:
                assert vector[module+'::'+expr]['value']==row['value'],key
        count+=1
print(f'{len(records)} inert records and {count} constants agree with pinned Rust-target vectors; no Omega layout/execution claim')

plans=(RAW/'console_layouts.omg').read_text()
for (module,name),fields in records.items():
    body=re.search(r'pub machine '+name+r'X64Layout::plan\(schema: Schema\) -> Plan satisfies Layout::plan \{(.*?)\n\}',plans,re.S)[1]
    found=re.findall(r'entries\[(\d+)\] = FieldEntry \{ key: schema.fields\[(\d+)\].key, placement: FieldPlan::At \{ offset: (\d+) \}',body)
    expected=[]
    for index,(field,_) in enumerate(fields):
        offset=vector.get(module+'::'+name+'.'+field+'.offset',{'value':0})['value']
        expected.append((str(index),str(index),str(offset)))
    assert found==expected,(module,name,'plan offsets')
    for spelling,kind in [('entry_count',None),('size_fixed','size'),('align','alignment')]:
        value=len(fields) if kind is None else vector[module+'::'+name+'.'+kind]['value']
        assert int(re.search(spelling+r': (\d+)',body)[1])==value,(module,name,spelling)
print(f'{len(records)} authored fixed policies match every target-checked field offset/size/alignment')
