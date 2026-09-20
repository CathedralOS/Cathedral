#!/usr/bin/env python3
"""Reviewed HII raw transcription for one pinned Rust source; not a Rust parser."""
from pathlib import Path
import ast
import json
import re
import subprocess
import sys
import uuid
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
sys.path.insert(0,str(HERE.parent))
import inventory
PIN='c0facddf9ba42b74906a37fca2869e6cdbc8da6a'
CHECKOUT=ROOT/'reference_code/rust-osdev/uefi-rs'
OUT=ROOT/'source/contracts/uefi/raw'
PATHS=['uefi-raw/src/protocol/hii/'+name+'.rs' for name in ['mod','config','database','font','form_browser','ifr','image','popup','string']]
manifest=inventory.snapshot(CHECKOUT,PIN,PATHS,'https://github.com/rust-osdev/uefi-rs')

def close(s,start):
    depth=1
    for i in range(start+1,len(s)):
        if s[i]=='{':depth+=1
        elif s[i]=='}':
            depth-=1
            if depth==0:return i
    raise ValueError('unclosed source')
def upper(s):return re.sub(r'(?<=[a-z0-9])(?=[A-Z])','_',s).upper()
def module(path):
    stem=Path(path).stem
    return 'hii' if stem=='mod' else 'hii_'+stem

def field_list(body):
    matches=list(re.finditer(r'\bpub\s+((?:r#)?\w+)\s*:',body))
    return [(m[1],body[m.end():matches[i+1].start() if i+1<len(matches) else len(body)].strip().rstrip(',').strip()) for i,m in enumerate(matches)]

records={};constants=[];sources={};ranges={}
for path in PATHS:
    original=(CHECKOUT/path).read_text()
    # Keep newline coordinates for exact inventory owner mapping.
    s=re.sub(r'//[^\n]*','',original)
    sources[path]=s;ranges[path]=[]
    for m in re.finditer(r'pub (struct|enum|union) (\w+)\s*(?::\s*(\w+)\s*(?:=>)?\s*)?([({])',s):
        kind,name,base,opening=m.groups();mod=module(path)
        if opening=='(':
            base=s[m.end():s.index(')',m.end())].replace('pub ','').strip();body='';end=s.index(';',m.end())
        else:end=close(s,m.end()-1);body=s[m.end():end]
        # Rust repr attribute belongs to immediately preceding declaration block.
        prefix=s[max(s.rfind('}',0,m.start()),s.rfind(';',0,m.start()))+1:m.start()]
        packed='repr(C, packed)' in prefix
        rust='uefi_raw::protocol::hii::'+('' if mod=='hii' else Path(path).stem+'::')+name
        flags=bool(kind=='struct' and base and opening=='{')
        records[name]={'name':name,'module':mod,'path':path,'rust':rust,'kind':kind,'base':base,'packed':packed,'flags':flags,'fields':[('raw',base)] if base else field_list(body)}
        ranges[path].append((m.start(),end,name))
        if base and opening=='{':
            cleaned=re.sub(r'#\s*\[[^\]]*\]','',body)
            for c in re.finditer(r'(?:const\s+)?([A-Z][A-Z_0-9]*)\s*=\s*([^;,]+)[;,]',cleaned):
                constants.append({'owner':name,'name':c[1],'type':'Self','expr':c[2].strip(),'path':path,'rust':rust+'::'+c[1]})
    for m in re.finditer(r'pub type (\w+)\s*=\s*(.*?);',s,re.S):
        name,typ=m.groups();mod=module(path);rust='uefi_raw::protocol::hii::'+('' if mod=='hii' else Path(path).stem+'::')+name
        records[name]={'name':name,'module':mod,'path':path,'rust':rust,'kind':'alias','base':typ.strip(),'packed':False,'flags':False,'fields':[('raw',typ.strip())]}
        ranges[path].append((m.start(),m.end(),name))
    for m in re.finditer(r'impl (\w+)\s*{',s):
        end=close(s,m.end()-1);owner=m[1];body=s[m.end():end];ranges[path].append((m.start(),end,owner))
        for c in re.finditer(r'pub const (\w+)\s*:\s*([^=]+?)\s*=\s*(.*?);',body,re.S):
            constants.append({'owner':owner,'name':c[1],'type':c[2].strip(),'expr':c[3].strip(),'path':path,'rust':records[owner]['rust']+'::'+c[1]})

unions={n for n,r in records.items() if r['kind']=='union'}
blocked=set(unions)
while True:
    more={n for n,r in records.items() if any(t in blocked for f,t in r['fields'])}
    if more<=blocked:break
    blocked|=more
for n,r in records.items():r['local']=n+'WireStorage' if n in blocked else n

def mapped(t,mod):
    t=' '.join(t.split())
    if '*' in t or re.search(r'\bfn\s*\(',t):return 'addr'
    if t=='usize':return 'u64'
    if t in {'Char16','QuestionId','ImageId','StringId','FormId','VarstoreId','AnimationId','DefaultId'}:return 'u16'
    if t in {'Char8','Boolean'}:return 'u8'
    if t in {'Handle','HiiHandle','FontHandle'}:return 'addr'
    if t=='Guid':return 'scalars::Guid'
    if t=='GraphicsOutputBltPixel':return 'console::GraphicsOutputBltPixel'
    if t.startswith('['):
        base,count=t[1:-1].split(';');return '['+mapped(base.strip(),mod)+'; '+count.strip()+']'
    if t in records:
        r=records[t];return ('' if r['module']==mod else r['module']+'::')+r['local']
    return t

values={};measurements={};probe=['// SPDX-License-Identifier: MIT OR Apache-2.0','use core::mem::{size_of,align_of,offset_of};','fn main() {']
for n,r in records.items():
    key=r['module']+'::'+r['local']
    for kind,op in [('size','size_of'),('alignment','align_of')]:
        measurements[key+'.'+kind]={'kind':kind,'value':0};probe.append(f'println!("{key}.{kind}={{}}", {op}::<{r["rust"]}>());')
    if not r['base']:
        for f,t in r['fields']:
            vk=key+'.'+f+'.offset';measurements[vk]={'kind':'offset','value':0};probe.append(f'println!("{vk}={{}}",offset_of!({r["rust"]},{f}));')

def number(c):
    e=c['expr'];owner=c['owner']
    e=e.replace('Self::empty()','0')
    def replace(m):return str(values[(owner if m[1]=='Self' else m[1],m[2])])
    e=re.sub(r'(Self|\w+)::(\w+)(?:\.bits\(\))?',replace,e)
    if not re.fullmatch(r'[0-9a-fA-FxX_ ()<>|&+\-]+',e):raise ValueError(('not numeric',e))
    return eval(compile(ast.parse(e,mode='eval'),'<pinned constant>','eval'),{'__builtins__':{}},{})
for c in constants:
    owner,name=c['owner'],c['name'];mod=module(c['path']);key=mod+'::'+upper(owner)+'_'+name;c['key']=key
    g=re.search(r'guid!\("([a-f0-9-]+)"\)',c['expr'])
    if g:
        c['guid']=g[1];measurements[key]={'kind':'bytes','value':uuid.UUID(g[1]).bytes_le.hex()}
        probe.append('{let bytes='+c['rust']+'.to_bytes();print!("'+key+'=");for b in bytes {print!("{:02x}",b);}println!();}')
    else:
        value=number(c);values[(owner,name)]=value;c['value']=value
        measurements[key]={'kind':'value','value':value}
        suffix='' if c['type']!='Self' else '.bits()' if records[owner]['flags'] else '.0'
        probe.append(f'println!("{key}={{}}",{c["rust"]}{suffix});')
probe.extend(['}','#[cfg(test)]','mod tests;'])
(HERE/'src/main.rs').write_text('\n'.join(probe)+'\n')
vector={'format':'cathedral-port-vectors-v1','target':{'pointer_bits':64,'endian':'little','abi':'UEFI x86-64'},'provenance':{'kind':'upstream','description':'Pinned upstream HII Rust layout/value expectations; measure.py target-checks before writing.','sources':PATHS,'revision':PIN},'measurements':measurements}
(HERE/'expected-template.json').write_text(json.dumps(vector,indent=2)+'\n')
# Translation requires an independently measured vector set, not guessed unions.
vpath=OUT/'hii.vectors.json'
if not vpath.exists():
    print('probe ready; run measure.py --write, then rerun generation');sys.exit(0)
measured=json.loads(vpath.read_text())['measurements']
outputs={module(p):['// SPDX-License-Identifier: MIT OR Apache-2.0','// Modified rust-osdev/uefi-rs translation at '+PIN+'.','// See hii.PORT.md. Inert carriers; pointer/slot bits confer no authority.',
                   'module '+module(p)+';','use scalars;','use console;'] for p in PATHS}
for mod in outputs: outputs[mod].append('')
schema=[];tails=[]
for n,r in records.items():
    mod,local=r['module'],r['local'];key=mod+'::'+local;lines=outputs[mod]
    lines.append('// Upstream: '+r['rust'])
    if n in blocked:lines.append('// PORT-BLOCKED[omega:programmable-overlays]: storage bytes are not a typed union or a validated IFR value.')
    if r['packed']:lines.append('// Required packed layout: alignment 1, no inter-field padding.')
    lines.append('pub data '+local+' [copy] {');fs=[]
    if n in unions:
        size=measured[key+'.size']['value'];fs=[('bytes','[u8; '+str(size)+']','bytes')]
        lines.append('    bytes: [u8; '+str(size)+'];')
        for f,t in r['fields']:lines.append('    // Blocked overlay member '+f+': '+t+'; upstream offset 0.')
    else:
        for f,t in r['fields']:
            if re.search(r';\s*0\s*\]',t):
                lines.append('    // PORT-BLOCKED[omega:runtime-layout-strides]: typed trailing '+f+' ('+t+') requires bounded runtime shape.')
                tails.append((mod,local,f,t));continue
            ft=mapped(t,mod);name=f+'_storage' if t in blocked else ('type_code' if f=='r#type' else f)
            lines.append('    '+name+': '+ft+';');fs.append((name,ft,f))
            if ft=='addr':lines.append('    // Foreign carrier: '+' '.join(t.split()))
    lines.extend(['}',''])
    schema.append({'module':mod,'name':local,'upstream':r['rust'],'fields':fs,'packed':r['packed'],'overlay_storage':n in blocked,'union_storage':n in unions,'tail_fields':[f for m,l,f,t in tails if m==mod and l==local]})
for c in constants:
    mod=module(c['path']);name=c['key'].split('::')[1]
    if 'guid' in c:
        u=uuid.UUID(c['guid']);a,b,cc,_,_,_=u.fields;typ='scalars::Guid'
        val=f'scalars::Guid {{ data1: 0x{a:08x}, data2: 0x{b:04x}, data3: 0x{cc:04x}, data4: ['+', '.join(f'0x{x:02x}' for x in u.bytes[8:])+'] }'
    else:
        typ=c['owner'] if c['type']=='Self' else mapped(c['type'],mod)
        val=typ+' { raw: '+str(c['value'])+' }' if c['type']=='Self' else str(c['value'])
    outputs[mod].append('pub const '+name+': '+typ+' = '+val+';')
for mod,lines in outputs.items():
    used={m for m in outputs if m!=mod and any(re.search(r'(?<![\w:])'+re.escape(m)+r'::',line) for line in lines if not line.lstrip().startswith('//'))}
    lines[6:6]=['use '+m+';' for m in sorted(used)]
    (OUT/(mod+'.omg')).write_text('\n'.join(lines)+'\n')
(HERE/'schema.json').write_text(json.dumps(schema,indent=2)+'\n')
plans=['// SPDX-License-Identifier: MIT OR Apache-2.0','// Fixed HII geometry; storage carriers do not establish union semantics.','module hii_layouts;','use omega::language::core::layout;','']
for r in schema:
    name=r['name'];key=r['module']+'::'+name
    plans.extend([f'pub data {name}X64Layout {{ }}',f'pub {name}X64Policy: {name}X64Layout satisfies Layout;',f'pub machine {name}X64Layout::plan(schema: Schema) -> Plan satisfies Layout::plan {{','    let mut entries: [FieldEntry; 64];'])
    for i,(f,t,original) in enumerate(r['fields']):
        offset=measured.get(key+'.'+original+'.offset',{'value':0})['value']
        plans.append(f'    entries[{i}] = FieldEntry {{ key: schema.fields[{i}].key, placement: FieldPlan::At {{ offset: {offset} }} }};')
    plans.extend(['    Plan { entries: entries, entry_count: '+str(len(r['fields']))+', size_fixed: '+str(measured[key+'.size']['value'])+', size_is_dynamic: false, align: '+str(measured[key+'.alignment']['value'])+' }','}',''])
(OUT/'hii_layouts.omg').write_text('\n'.join(plans))
# Exact source-owner mapping; formatter helpers are intentional non-ABI omissions.
for path,entry in manifest['files'].items():
    mod=module(path);target='source/contracts/uefi/raw/'+mod+'.omg';text=(OUT/(mod+'.omg')).read_text()
    entry.update(disposition='translated',targets=[{'path':target,'anchor':'module '+mod+';'}]);entry.pop('reason',None)
    s=sources[path]
    for key,row in entry['symbols'].items():
        line,name=key.split(':',1);pos=sum(len(x)+1 for x in s.splitlines()[:int(line)-1]);possible=[n for a,b,n in ranges[path] if a<=pos<=b]
        if not possible:
            # Declaration line starts before the `pub` match.
            pos+=len(s.splitlines()[int(line)-1])-len(s.splitlines()[int(line)-1].lstrip());possible=[n for a,b,n in ranges[path] if a<=pos<=b]
        owner=possible[-1] if possible else None;anchor=row['anchor']
        if name=='_':row.update(disposition='omitted',reason='Pinned compile-time ABI assertions are retained in target-checked geometry vectors.');continue
        if name=='fmt':row.update(disposition='omitted',reason='Rust Debug formatting is not a raw ABI operation; union projections are blocked independently.');continue
        if name=='default':row.update(disposition='blocked',reason='omega:programmable-overlays: ImageOutputDest typed union default requires overlay representation; zero storage bytes remain constructible.',targets=[{'path':target,'anchor':'pub data ImageOutputDestWireStorage '}]);continue
        if name in {'storage','size','display'} and 'fn' in anchor:
            machine={'IfrDateFlags':'ifr_date_storage','IfrTimeFlags':'ifr_time_storage','IfrNumericFlags':'ifr_numeric_'+name}[owner]
            row.update(disposition='translated',targets=[{'path':'source/libraries/uefi/hii_helpers.omg','anchor':'pub machine '+machine+'('}]);row.pop('reason',None);continue
        if anchor.startswith('pub mod'):
            row.update(disposition='translated',targets=[{'path':target,'anchor':'module '+mod+';'}],reason='Rust submodule maps to separate hii_* source module.');continue
        if owner in blocked:
            row.update(disposition='blocked',reason='omega:programmable-overlays: original typed union/member or enclosing type requires checked overlay semantics; mapped Storage only preserves inert backing.',targets=[{'path':target,'anchor':'pub data '+records[owner]['local']+' '}]);continue
        if re.search(r';\s*0\s*\]',anchor):
            row.update(disposition='blocked',reason='omega:runtime-layout-strides: fixed prefix retained, typed runtime tail requires bounded extent/stride.',targets=[{'path':target,'anchor':'typed trailing '+name+' ('}]);continue
        if name in records:a='pub data '+records[name]['local']+' '
        elif any(c['path']==path and c['name']==name and (not owner or c['owner']==owner) for c in constants):a=next(c['key'].split('::')[1] for c in constants if c['path']==path and c['name']==name and (not owner or c['owner']==owner))
        elif re.match(r'pub\s+(?:r#)?\w+\s*:',anchor):a=('type_code' if name=='r#type' else name)+':'
        elif ' '.join(anchor.split()).rstrip(',') in text:a=' '.join(anchor.split()).rstrip(',')
        else:raise ValueError(('unmapped',path,key,owner,anchor))
        row.update(disposition='translated',targets=[{'path':target,'anchor':a}]);row.pop('reason',None)
(OUT/'hii-inventory.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(len(records),'carriers,',len(constants),'constants,',len(unions),'unions,',len(blocked),'union-dependent carriers,',len(tails),'tails')
