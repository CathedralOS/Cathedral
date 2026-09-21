#!/usr/bin/env python3
"""Generate actual-body Omega fixtures and public pinned Rust observations."""
import json,subprocess,sys
from pathlib import Path
from corpus import CASES,parse,template
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[3]
CHECK='--check'in sys.argv
def write(path,text):
 if CHECK:assert path.read_text()==text,('stale',path)
 else:path.write_text(text)
def b(v):return str(bool(v)).lower()
imports='\n'.join('use resources::resource_model::'+name+';'for name in ['Outcome','Parsed','Resource','Irq','Interrupts','Trigger','Polarity','Dma','DmaSpeed','DmaWidth','Io','FixedMemory','Address','AddressKind','Source','Template','NumberResult','Gpio','GpioConnection','GpioPolarity','PinConfiguration','IoRestriction','I2c','AddressingMode','Span'])+'\nuse resources::resource_parse::parse_one;\nuse resources::resource_template::validate_template;\nuse resources::resource_irq::irq_at;\nuse resources::resource_connections::pin_at;\n'
footer='\nconst RESULT:i32=test();\nmachine require_ok(value:i32) requires value==0;{}\ndata Main{}\nmachine Main::main(&mut self){require_ok(RESULT);}\n'
metadata={};rust=[]
def source_checks(s):return [f'p.source.present=={b(s["present"])}',f'p.source.index=={s["index"]}',f'p.source.name.start=={s["start"]}',f'p.source.name.end=={s["end"]}']
for case in CASES:
 name=case['name'];data=case['data'];length=case.get('length',len(data));at=case.get('at',0);expected=template(data,length)if case.get('template')else parse(data,length,at)
 assigns='let mut input:[u8;4096];\n'+'\n'.join(f'input[{i}]={v};'for i,v in enumerate(data)if v)
 helpers='';c=[]
 if case.get('template'):
  call=f'let r:Template=validate_template(&input,{length});';c=[f'r.outcome==Outcome::{expected["outcome"]}']+[f'r.{key}=={expected[key]}'for key in ['next','count','unsupported','checksum']]
 else:
  call=f'let r:Parsed=parse_one(&input,{length},{at});';c=[f'r.outcome==Outcome::{expected["outcome"]}']
  if expected['outcome']!='Success':c+=['r.resource in Resource::None','r.next==0']
  else:
   e=expected;kind=e['kind'];c += [f'r.next=={e["end"]}',f'r.encoding.start=={at}',f'r.encoding.end=={e["end"]}']
   if kind=='EndTag':helpers=f'machine check_resource(input:&[u8;4096],value:Resource)->bool {{transition value {{Resource::EndTag{{checksum}} -> (checksum=={e["checksum"]}) _ -> (false)}}}}'
   elif kind=='Unsupported':helpers=f'machine check_resource(input:&[u8;4096],value:Resource)->bool {{transition value {{Resource::Unsupported{{large,tag,encoding}} -> (large=={b(e["large"])} && tag=={e["tag"]} && encoding.start=={at} && encoding.end=={e["end"]}) _ -> (false)}}}}'
   else:
    field={'Irq':'irq','Dma':'dma','Io':'io','FixedMemory':'memory','Address':'address','Gpio':'gpio','I2c':'i2c'}[kind];tests=[];setup=''
    if kind=='Irq':
     tests=[f'p.extended=={b(e["extended"])}',f'p.is_consumer=={b(e["consumer"])}',f'p.trigger==Trigger::{"Edge"if e["edge"]else"Level"}',f'p.polarity==Polarity::{"ActiveLow"if e["low"]else"ActiveHigh"}',f'p.shared=={b(e["shared"])}',f'p.wake=={b(e["wake"])}',f'p.flags=={e["flags"]}']+source_checks(e["source"])
     # Actual mask expansion and table numeric reads, plus absent index result.
     for j in sorted(set([0,len(e['irqs'])-1])):
      if j>=0 and j<len(e['irqs']):setup+=f'let i{j}:NumberResult=irq_at(input,{length},p.interrupts,{j});';tests += [f'i{j}.outcome==Outcome::Success',f'i{j}.value=={e["irqs"][j]}']
     setup+=f'let missing:NumberResult=irq_at(input,{length},p.interrupts,{len(e["irqs"])});';tests+=['missing.outcome==Outcome::InvalidState']
     if e['extended']:it=f'Interrupts::Table{{entries,count}} -> (count=={len(e["irqs"])} && entries.start=={e["table_start"]} && entries.end=={e["table_end"]})'
     else:it=f'Interrupts::Mask{{bits}} -> (bits=={e["mask"]})'
     helpers+=f'machine check_interrupts(value:Interrupts)->bool {{transition value {{{it} _ -> (false)}}}}\n';setup+='let shape:bool=check_interrupts(p.interrupts);';tests+=['shape']
    elif kind=='Dma':tests=[f'p.channels=={e["channels"]}',f'p.speed==DmaSpeed::{["Compatibility","TypeA","TypeB","TypeF"][e["speed"]]}',f'p.width==DmaWidth::{["Bits8","Bits8And16","Bits16"][e["width"]]}',f'p.bus_master=={b(e["master"])}',f'p.flags=={e["flags"]}']
    elif kind=='Io':tests=[f'p.decodes_full=={b(e["decode"])}']+[f'p.{k}=={e[k]}'for k in ['minimum','maximum','alignment','length']]
    elif kind=='FixedMemory':tests=[f'p.writable=={b(e["writable"])}']+[f'p.{k}=={e[k]}'for k in ['base','length','flags']]
    elif kind=='Address':tests=[f'p.width=={e["width"]}',f'p.kind==AddressKind::{["Memory","IoRange","BusNumber"][e["address_kind"]]}',f'p.general_flags=={e["flags"]}',f'p.type_flags=={e["type_flags"]}',f'p.maximum_fixed=={b(e["flags"]&8)}',f'p.minimum_fixed=={b(e["flags"]&4)}',f'p.subtractive=={b(e["flags"]&2)}']+[f'p.{key}=={v}'for key,v in zip(['granularity','minimum','maximum','translation','length'],e['values'])]+source_checks(e["source"])
    elif kind in ['Gpio','I2c']:
     tests=[f'p.is_consumer=={b(e["consumer"])}',f'p.shared=={b(e["shared"])}',f'p.vendor.start=={e["vendor_start"]}',f'p.vendor.end=={e["vendor_end"]}']+source_checks(e["source"])
     if kind=='Gpio':
      tests += [f'p.pin_configuration==PinConfiguration::{["Default","PullUp","PullDown","NoPull"][e["config"]]}',f'p.drive_strength=={e["drive"]}',f'p.debounce_timeout=={e["debounce"]}',f'p.pins.start=={e["pin_start"]}',f'p.pins.end=={e["pin_end"]}',f'p.pin_count=={len(e["pins"])}',f'p.general_flags=={e["general"]}',f'p.connection_flags=={e["flags"]}']
      for j in sorted(set([0,len(e['pins'])-1])):
       setup+=f'let pin{j}:NumberResult=pin_at(input,{length},p.pins,{j});';tests += [f'pin{j}.outcome==Outcome::Success',f'pin{j}.value=={e["pins"][j]}']
      setup+=f'let missing:NumberResult=pin_at(input,{length},p.pins,{len(e["pins"])});';tests+=['missing.outcome==Outcome::InvalidState']
      conn=e['connection']
      arm=f'GpioConnection::Interrupt{{trigger,polarity,wake}} -> (trigger==Trigger::{"Edge"if conn[1]else"Level"} && polarity==GpioPolarity::{["ActiveHigh","ActiveLow","ActiveBoth"][conn[2]]} && wake=={b(conn[3])})'if conn[0]==0 else f'GpioConnection::Io{{restriction}} -> (restriction==IoRestriction::{["None","InputOnly","OutputOnly","Preserve"][conn[1]]})'
      helpers+=f'machine check_connection(value:GpioConnection)->bool {{transition value {{{arm} _ -> (false)}}}}\n';setup+='let connection:bool=check_connection(p.connection);';tests+=['connection']
     else:tests += [f'p.device_initiated=={b(e["device"])}',f'p.addressing==AddressingMode::{"Bits10"if e["mode"]else"Bits7"}',f'p.speed=={e["speed"]}',f'p.address=={e["address"]}',f'p.legacy_virtual_register=={e["lvr"]}',f'p.general_flags=={e["flags"]}']
    helpers+=f'machine check_resource(input:&[u8;4096],value:Resource)->bool {{transition value {{Resource::{kind}Resource{{{field}}} -> details(input,{field}) _ -> (false)}} state details(input:&[u8;4096],p:{kind})->bool {{{setup}\n'+' && '.join(tests)+'}}'
   call+='let payload:bool=check_resource(&input,r.resource);';c+=['payload']
 old=f'r.outcome==Outcome::{expected["outcome"]}';new='r.outcome==Outcome::'+('BadEncoding'if expected['outcome']=='Success'else'Success')
 text='// SPDX-License-Identifier: MIT OR Apache-2.0\n// Original synthetic fixture; generated by generate.py.\n'+imports+helpers+'\nmachine test()->i32 {\n'+assigns+'\n'+call+'\ntransition '+' && '.join(c)+' {true -> (0) _ -> (1)}\n}'+footer
 write(HERE/'cases'/f'{name}.omg',text);metadata[name]={'mutation':[old,new],'expected':expected,'input':data,'logical_length':length,'offset':at,'template':case.get('template',False)}
 if length<=len(data)and at<=length:
  raw=data if case.get('template')else data[at:length]
  rust.append('observe('+json.dumps(name)+',&'+str(raw)+');')
for name,start,end,count,index,length in [
 ('irq-access-reversed',8,4,1,0,8),('irq-access-length',0,4,1,0,3),
 ('irq-access-zero-count',0,0,0,0,8),('irq-access-big-count',0,1024,256,0,1024),
 ('irq-access-index',0,4,1,1,4),('irq-access-index-max',0,4,1,2**64-1,4),
 ('irq-access-offset-max',2**64-1,2**64-1,1,0,4096),
 ('irq-access-wrong-span',0,8,1,0,8),('irq-access-length-max',0,4,1,0,2**64-1),
 ('irq-access-last',4092,4096,1,0,4096)]:
 outcome='Success'if name=='irq-access-last'else'InvalidState';number=0xfedcba98 if outcome=='Success'else 0
 writes='input[4092]=152;input[4093]=186;input[4094]=220;input[4095]=254;'if number else''
 body=f'let mut input:[u8;4096];{writes}let interrupts:Interrupts=Interrupts::Table{{entries:Span{{start:{start},end:{end}}},count:{count}}};let r:NumberResult=irq_at(&input,{length},interrupts,{index});transition r.outcome==Outcome::{outcome} && r.value=={number} {{true -> (0) _ -> (1)}}'
 text='// SPDX-License-Identifier: MIT OR Apache-2.0\nuse resources::resource_model::Span;\n'+imports+'machine test()->i32 {'+body+'}'+footer
 write(HERE/'cases'/f'{name}.omg',text);metadata[name]={'mutation':[f'r.outcome==Outcome::{outcome}',f'r.outcome==Outcome::{"InvalidState"if outcome=="Success"else"Success"}'],'expected':{'outcome':outcome,'value':number},'mode':'irq-access','template':False}
for name,start,end,index,length in [('pin-access-reversed',4,2,0,4),('pin-access-odd',0,3,0,3),('pin-access-empty',0,0,0,0),('pin-access-length',0,4,0,3),('pin-access-index-max',0,4,2**64-1,4),('pin-access-offset-max',2**64-1,2**64-1,0,4096),('pin-access-length-max',0,2,0,2**64-1),('pin-access-last',4094,4096,0,4096)]:
 outcome='Success'if name=='pin-access-last'else'InvalidState';number=65535 if outcome=='Success'else 0
 writes='input[4094]=255;input[4095]=255;'if number else''
 body=f'let mut input:[u8;4096];{writes}let pins:Span=Span{{start:{start},end:{end}}};let r:NumberResult=pin_at(&input,{length},pins,{index});transition r.outcome==Outcome::{outcome} && r.value=={number} {{true -> (0) _ -> (1)}}'
 text='// SPDX-License-Identifier: MIT OR Apache-2.0\n'+imports+'machine test()->i32 {'+body+'}'+footer
 write(HERE/'cases'/f'{name}.omg',text);metadata[name]={'mutation':[f'r.outcome==Outcome::{outcome}',f'r.outcome==Outcome::{"InvalidState"if outcome=="Success"else"Success"}'],'expected':{'outcome':outcome,'value':number},'mode':'pin-access','template':False}
# Exact fixture membership; removed formerly Unsupported GPIO/serial cases are stale.
for path in (HERE/'cases').glob('*.omg'):
 if path.stem not in metadata:
  if CHECK:raise AssertionError(('stale fixture',path))
  path.unlink()
write(HERE/'cases.json',json.dumps(metadata,indent=2,sort_keys=True)+'\n')
write(HERE/'src/main.rs',(HERE/'probe.rs.in').read_text().replace('// GENERATED','\n'.join(rust)))
result=subprocess.check_output(['cargo','+nightly-2026-09-04','run','--quiet','--locked','--manifest-path',str(HERE/'Cargo.toml')],cwd=ROOT,text=True)
observations={row['name']:{k:v for k,v in row.items()if k!='name'}for row in map(json.loads,result.splitlines())}
matched=0
for name,m in metadata.items():
 e=m['expected'];o=observations.get(name)
 if not m['template']and e['outcome']=='Success'and e.get('norm')is not None:
  assert o=={'result':'ok','resources':[e['norm']]},(name,e,o);matched+=1
write(HERE/'observations.json',json.dumps(observations,indent=2,sort_keys=True)+'\n')
print(len(metadata),'Omega fixtures;',len(observations),'actual pinned public parser observations;',matched,'normalized supported descriptor agreements')
