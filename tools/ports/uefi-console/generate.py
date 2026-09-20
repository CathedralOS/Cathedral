#!/usr/bin/env python3
"""Reproduce reviewed inert UEFI-003 transcriptions and pinned Rust probes.

Not a Rust parser. Deliberately restricted to the exact recorded source pin.
Review source diffs and mappings before using this for any newer pin.
"""
from pathlib import Path
import hashlib
import json
import re
import sys
import uuid
ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT/'tools/ports'))
import inventory
PIN = 'c0facddf9ba42b74906a37fca2869e6cdbc8da6a'
CHECKOUT = ROOT/'reference_code/rust-osdev/uefi-rs'
OUT = ROOT/'source/contracts/uefi/raw'
FILES = {'console.rs':'console','console/serial.rs':'serial','loaded_image.rs':'loaded_image',
         'device_path.rs':'device_path','device_path/device_path_gen.rs':'device_path',
         'media.rs':'load_file','shell_params.rs':'shell_params'}
roots=['uefi-raw/src/protocol/'+p for p in FILES]
manifest=inventory.snapshot(CHECKOUT,PIN,roots,'https://github.com/rust-osdev/uefi-rs')

def stripped(s):
    s=re.sub(r'//[^\n]*','',s)
    s=re.sub(r'#\s*\[(?:[^]"\n]|"[^"\n]*")*\]','',s)
    return s

def endbrace(s,start):
    depth=1
    for i in range(start+1,len(s)):
        if s[i]=='{':depth+=1
        elif s[i]=='}':
            depth-=1
            if depth==0:return i
    raise ValueError('unclosed brace')

def title(s):return ''.join(p.capitalize() for p in s.split('_'))
def upper(s):return re.sub(r'(?<=[a-z0-9])(?=[A-Z])','_',s).upper()
def fields(body):
    hits=list(re.finditer(r'\bpub\s+([a-z_]\w*)\s*:',body))
    return [(m[1],body[m.end():hits[i+1].start() if i+1<len(hits) else len(body)].strip().rstrip(',').strip()) for i,m in enumerate(hits)]

records=[];constants=[];aliases=[];contexts={};tails=[]
for rel,module in FILES.items():
    path='uefi-raw/src/protocol/'+rel
    text=(CHECKOUT/path).read_text();s=stripped(text)
    mods=[]
    for m in re.finditer(r'pub mod (\w+)\s*{',s):mods.append((m.end(),endbrace(s,m.end()-1),m[1]))
    def category(pos):return next((name for a,b,name in mods if a<=pos<b),'')
    contexts[path]=[]
    for m in re.finditer(r'pub (struct|enum) (\w+)\s*(?::\s*(\w+)\s*(?:=>)?\s*)?([({])',s):
        kind,name,base,opening=m.groups();cat=category(m.start());local=(title(cat)+name) if cat else name
        rust='uefi_raw::protocol::'+('device_path::'+cat+'::' if cat else rel.removesuffix('.rs').replace('/','::')+'::')+name
        if opening=='(':
            base=s[m.end():s.index(')',m.end())].replace('pub ','').strip();body='';end=s.index(';',m.end())
        else:
            end=endbrace(s,m.end()-1);body=s[m.end():end]
        if base:
            records.append({'module':module,'name':local,'rust':rust,'fields':[('raw',base)],'base':base,'path':path,'packed':False})
            for c in re.finditer(r'(?:const\s+)?([A-Z][A-Z_0-9]*)\s*=\s*(.*?)(?:;|,(?=\s*(?:[A-Z_]|$)))',body,re.S):
                expr=c[2].strip();constants.append((module,local,c[1],expr,rust+'::'+c[1],path))
        else:
            packed=bool(cat)
            records.append({'module':module,'name':local,'rust':rust,'fields':fields(body),'path':path,'packed':packed})
        contexts[path].append((name,local))
    # Associated constants, including transparent DeviceSubType, GUID aliases.
    for m in re.finditer(r'impl (\w+)\s*{',s):
        owner=m[1];body=s[m.end():endbrace(s,m.end()-1)]
        for c in re.finditer(r'pub const (\w+)\s*:\s*[^=]+?=\s*(.*?);',body,re.S):
            constants.append((module,owner,c[1],c[2].strip(),'uefi_raw::protocol::'+rel.removesuffix('.rs').replace('/','::')+'::'+owner+'::'+c[1],path))
    for m in re.finditer(r'pub type (\w+)\s*=\s*(.*?);',s,re.S):
        aliases.append((module,m[1],m[2].strip(),path))

known={(r['module'],r['name']):r for r in records}
external={'Boolean':'u8','Char16':'u16','Event':'addr','Handle':'addr','PhysicalAddress':'u64','MemoryType':'u32'}
def translated_type(t,module):
    t=' '.join(t.split())
    if '*' in t or 'fn(' in t or 'fn (' in t:return 'addr'
    if t in external:return external[t]
    if t=='usize':return 'u64'
    if t=='DevicePathHeader':return 'DevicePathProtocol'
    if t=='Guid':return 'scalars::Guid'
    if t.startswith('device_path::'):
        _,cat,name=t.split('::');return title(cat)+name
    if t.startswith('['):
        typ,num=t[1:-1].split(';');return '['+translated_type(typ.strip(),module)+'; '+num.strip().removesuffix('usize')+']'
    return t

def guid_value(value):
    u=uuid.UUID(value);a,b,c,_,_,_=u.fields
    return f'scalars::Guid {{ data1: 0x{a:08x}, data2: 0x{b:04x}, data3: 0x{c:04x}, data4: ['+', '.join(f'0x{x:02x}' for x in u.bytes[8:])+'] }'

outputs={m:['// SPDX-License-Identifier: MIT OR Apache-2.0',
    '// Modified Omega translation of rust-osdev/uefi-rs at '+PIN+'.',
    '// See console.PORT.md; reproduced by tools/ports/uefi-console/generate.py.',
    '// Inert x86-64 representation corpus. No pointer dereference or callable authority.',
    '// Layout vectors describe required foreign geometry, not default Omega layout.',
    f'module {m};','use scalars;',''] for m in set(FILES.values())}
probe=['// SPDX-License-Identifier: MIT OR Apache-2.0','// Host-only measurements of pinned upstream, never Omega execution.',
'use core::mem::{size_of, align_of, offset_of};','fn main() {']
measures={};schema=[]
for r in records:
    module,name=r['module'],r['name'];dst=outputs[module]
    dst.append('// Upstream: '+r['rust'])
    if r['packed']:dst.append('// Required packed placement: alignment 1; no implicit inter-field padding.')
    dst.append('pub data '+name+' [copy] {')
    fs=[]
    for field,typ in r['fields']:
        is_tail=bool(re.search(r';\s*0(?:usize)?\s*\]',typ))
        if is_tail:
            dst.append('    // PORT-BLOCKED[omega:runtime-layout-strides]: typed trailing '+field+' ('+typ+') needs runtime extent/stride contract.')
            tails.append((module,name,field,typ,r['path']))
        else:
            mapped=translated_type(typ,module);dst.append('    '+field+': '+mapped+';')
            if mapped=='addr':dst.append('    // Foreign carrier: '+' '.join(typ.split()))
            fs.append((field,mapped))
        key=module+'::'+name+'.'+field
        # tuple wrapper raw fields use .0 upstream, not a named field
        orig='0' if field=='raw' else field
        if field != 'raw':
            probe.append(f'println!("{key}.offset={{}}", offset_of!({r["rust"]}, {orig}));')
            measures[key+'.offset']={'kind':'offset','value':0}
    dst.extend(['}',''])
    key=module+'::'+name
    for kind,op in [('size','size_of'),('alignment','align_of')]:
        probe.append(f'println!("{key}.{kind}={{}}", {op}::<{r["rust"]}>());')
        measures[key+'.'+kind]={'kind':kind,'value':0}
    schema.append({'module':module,'name':name,'upstream':r['rust'],'fields':fs,'packed':r['packed'],'tail_fields':[f for mod,n,f,t,p in tails if mod==module and n==name]})
for module,name,typ,path in aliases:
    outputs[module].extend(['// Upstream carrier: '+' '.join(typ.split()),f'pub data {name} [copy] {{ raw: addr; }}',''])
    rust='uefi_raw::protocol::'+next(k.removesuffix('.rs').replace('/','::') for k,v in FILES.items() if 'uefi-raw/src/protocol/'+k==path)+'::'+name
    for kind,op in [('size','size_of'),('alignment','align_of')]:
        key=module+'::'+name+'.'+kind;probe.append(f'println!("{key}={{}}", {op}::<{rust}>());');measures[key]={'kind':kind,'value':0}
constmap={}
for module,owner,name,expr,rust,path in constants:
    cname=upper(owner)+'_'+name;key=module+'::'+cname;typ=owner
    expr=expr.replace('\n',' ')
    guid=re.search(r'guid\s*!\s*\(\s*"([0-9a-f-]+)"\s*\)',expr)
    if guid:
        typ='scalars::Guid' if name=='GUID' else owner
        raw=guid_value(guid[1]);val=raw if name=='GUID' else owner+' { raw: '+raw+' }'
        b=uuid.UUID(guid[1]).bytes_le.hex();measures[key]={'kind':'bytes','value':b}
    elif name=='GUID':
        typ='scalars::Guid';val=upper(expr.split('::')[0])+'_GUID'
        # upstream equality checked as GUID bytes by probe below
        measures[key]={'kind':'bytes','value':'00'*16}
    else:
        num=re.fullmatch(r'(?:Self\()?\s*(0x[0-9a-fA-F_]+|[0-9_]+)\s*\)?',expr)
        if num:integer=int(num[1].replace('_',''),0) if num[1].startswith('0x') else int(num[1].replace('_',''));raw=str(integer)
        elif name=='SETTABLE':integer=0x7003;raw=str(integer)
        else:raise ValueError((key,expr))
        val=owner+' { raw: '+raw+' }';measures[key]={'kind':'value','value':integer}
    outputs[module].append(f'pub const {cname}: {typ} = {val};')
    constmap[(path,owner,name)]=cname
    if name=='GUID' or guid:
        probe.append('{ let value = '+rust+('' if name=='GUID' else '.0')+'; let bytes: [u8; 16] = value.to_bytes(); print!("'+key+'="); for b in bytes { print!("{:02x}", b); } println!(); }')
    else:
        is_flag=any(x in expr for x in ['.bits()']) or owner in ['AbsolutePointerModeAttributes','KeyShiftState','KeyToggleState','ControlBits','MessagingInfinibandResourceFlags','MessagingIscsiLoginOptions']
        probe.append(f'println!("{key}={{}}", {rust}'+('.bits()' if is_flag else '.0')+');')
probe.extend(['}', '#[cfg(test)]', 'mod tests;'])
for module,lines in outputs.items():(OUT/(module+'.omg')).write_text('\n'.join(lines)+'\n')
Path(__file__).with_name('schema.json').write_text(json.dumps(schema,indent=2)+'\n')
Path(__file__).with_name('src').joinpath('main.rs').write_text('\n'.join(probe)+'\n')
# Refine lexical anchors with full-file source hashes and explicit target rows.
for path,entry in manifest['files'].items():
    rel=path.removeprefix('uefi-raw/src/protocol/');module=FILES[rel];target='source/contracts/uefi/raw/'+module+'.omg'
    entry.update(disposition='translated',targets=[{'path':target,'anchor':'module '+module+';'}]);entry.pop('reason',None)
    for key,row in entry['symbols'].items():
        line,name=key.split(':',1);anchor=row['anchor']
        if name in ['default','length']:
            machine={'default':'graphics_output_mode_default','length':'device_path_length'}[name]
            row.update(disposition='translated',targets=[{'path':'source/libraries/uefi/console_helpers.omg','anchor':'pub machine '+machine+'('}]);row.pop('reason',None);continue
        if name=='_':
            row.update(disposition='omitted',reason={'default':'Rust Default ergonomic constructor; raw zero fields remain representable; no live behavior copied.', 'length':'Translated pure length decode is tracked in console_helpers.omg and its fixture cases.', '_':'Compile-time ABI assertions retained as measured geometry vectors.'}[name])
            continue
        if name=='serial' or name=='device_path_gen' or anchor.startswith('pub use') or anchor.startswith('pub mod'):
            row.update(disposition='translated',targets=[{'path':target,'anchor':'module '+module+';'}],reason='Rust module/reexport organization flattened into named Omega modules/category prefixes.');continue
        # Match field or declaration exact token; compact macros additionally covered by constants vectors.
        choices=[]
        original=(CHECKOUT/path).read_text()
        prefix='\n'.join(original.splitlines()[:int(line)])
        category_matches=list(re.finditer(r'pub mod (\w+)\s*\{',prefix))
        category_name=category_matches[-1][1] if category_matches and rel.endswith('device_path_gen.rs') else ''
        if category_name and (module,title(category_name)+name) in known:
            candidates=[title(category_name)+name]
        else:candidates=[name]+[n for orig,n in contexts[path] if orig==name]
        for local in candidates:
            if 'pub data '+local+' ' in (OUT/(module+'.omg')).read_text():choices.append('pub data '+local+' ')
        text=(OUT/(module+'.omg')).read_text()
        if choices:a=choices[-1]
        elif name=='GUID':
            owner=list(re.finditer(r'impl (\w+)\s*\{',prefix))[-1][1]
            a=constmap[(path,owner,name)]+':'
        elif re.match(r'pub\s+\w+\s*:',anchor):
            if re.search(r';\s*0(?:usize)?\s*\]',anchor):
                row.update(disposition='blocked',reason='omega:runtime-layout-strides: fixed prefix retained; runtime trailing element view requires a checked extent/stride shape.',targets=[{'path':target,'anchor':'typed trailing '+name+' ('}]);continue
            a=name+':'
        elif name in [n for m,n,t,p in aliases]:a='pub data '+name+' '
        elif any(p==path and n==name for p,o,n in constmap):a=next(c for (p,o,n),c in constmap.items() if p==path and n==name)
        elif ' '.join(anchor.split()).rstrip(',') in text:a=' '.join(anchor.split()).rstrip(',')
        else:raise ValueError(('unmapped',path,key,anchor))
        row.update(disposition='translated',targets=[{'path':target,'anchor':a}]);row.pop('reason',None)
(OUT/'console-inventory.json').write_text(json.dumps(manifest,indent=2)+'\n')
vector={'format':'cathedral-port-vectors-v1','target':{'pointer_bits':64,'endian':'little','abi':'UEFI x86-64'},'provenance':{'kind':'upstream','description':'Pinned upstream Rust host layout/value measurement; UEFI x64 data geometry, not Omega output. Generated by tools/ports/uefi-console/measure.py.','sources':roots,'revision':PIN},'measurements':measures}
Path(__file__).with_name('expected-template.json').write_text(json.dumps(vector,indent=2)+'\n')
print('generated',len(records),'records',len(constants),'constants',len(aliases),'aliases;',len(tails),'runtime tails')

# Target-checked geometry supplies fixed policies. Dynamic tails are not fields
# of the prefix declaration and never become invented zero-byte typed views.
vector_path=OUT/'console.vectors.json'
if vector_path.exists():
    measured=json.loads(vector_path.read_text())['measurements']
    plans=['// SPDX-License-Identifier: MIT OR Apache-2.0',
           '// Authored fixed geometry for UEFI-003, derived from pinned target-checked vectors.',
           '// Selecting a policy grants no backing, foreign call or runtime-tail authority.',
           'module console_layouts;', 'use omega::language::core::layout;', '']
    plan_records=schema+[{'module':m,'name':n,'fields':[('raw','addr')],'tail_fields':[]} for m,n,t,p in aliases]
    for record in plan_records:
        name=record['name'];key=record['module']+'::'+name;policy=name+'X64Layout'
        if record['tail_fields']:
            plans.append('// Fixed prefix only; PORT-BLOCKED[omega:runtime-layout-strides]: typed tails require runtime extent/stride forms.')
        plans.extend([f'pub data {policy} {{ }}',f'pub {name}X64Policy: {policy} satisfies Layout;',
            f'pub machine {policy}::plan(schema: Schema) -> Plan satisfies Layout::plan {{',
            '    let mut entries: [FieldEntry; 64];'])
        for index,(field,typ) in enumerate(record['fields']):
            offset=measured.get(key+'.'+field+'.offset',{'value':0})['value']
            plans.append(f'    entries[{index}] = FieldEntry {{ key: schema.fields[{index}].key, placement: FieldPlan::At {{ offset: {offset} }} }};')
        plans.extend(['    Plan { entries: entries, entry_count: '+str(len(record['fields']))+', size_fixed: '+str(measured[key+'.size']['value'])+', size_is_dynamic: false, align: '+str(measured[key+'.alignment']['value'])+' }','}',''])
    (OUT/'console_layouts.omg').write_text('\n'.join(plans))
