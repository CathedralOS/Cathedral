#!/usr/bin/env python3
"""Original bounded object admission and ACPI ToString expectation vectors."""
from collections import Counter
from pathlib import Path
import json
MAX=(1<<64)-1
HERE=Path(__file__).resolve().parent

def cases():
 rows=[]
 def add(name,data=b'AB',owned=False,maximum=MAX,**kw):
  r=dict(name=name,data=list(data),owned=owned,maximum=maximum,count=1,object=0,kind='Buffer',declared=len(data),length=1024,unit=7,extra='',error=None);r.update(kw)
  if not r['error']:
   material=r['data']+[0]*max(0,r['declared']-len(r['data'])) if not owned else r['data']
   selected=material[:maximum];selected=selected[:selected.index(0)]if 0 in selected else selected
   if any(b>127 for b in selected):r['error']='Encoding'
   else:r['expected']=selected
  rows.append(r)
 patterns=[b'',b'A',b'ABC',b'\x00ABC',b'A\0\xff',b'AB\0C',b'A\xffC',b'\xc3\xa9',b'\x7f',b'A'*255+b'\0',b'A'*255+b'\xff',b'Z'*256]
 for owned in [False,True]:
  for i,data in enumerate(patterns):
   for maximum in [0,1,2,255,1<<32,MAX]:add(f'matrix_{int(owned)}_{i}_{maximum}',data,owned,maximum)
  for bad in [257,1<<63,MAX]:
   extra=f'store.bytes.blocks[0].length={bad};'if owned else f'store.space.objects[0].value=Value::Buffer {{buffer_storage:BufferStorage::Source {{declared_size:{bad},buffer_initializer:Span {{unit:7,start:10,end:12}}}}}};'
   add(f'backing_capacity_{int(owned)}_{bad}',owned=owned,maximum=0,extra=extra,error='Capacity')
 for n in [0,1,2,256]:add(f'padding_{n}',b'A',declared=n)
 for count in [0,65,1<<63,MAX]:add(f'count_{count}',count=count,error='InvalidState')
 for obj in [1,64,1<<63,MAX]:add(f'object_{obj}',object=obj,error='InvalidState')
 for kind in ['Uninitialized','Integer','String','Package','NameReference','BufferField','Device','Method']:
  add(f'unsupported_{kind}',kind=kind,error='UnsupportedValue')
 for ref in ['Named','Local','Arg','RefOf','Index','Unresolved']:
  add(f'reference_{ref}',extra=f'store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{ref},object_id:0}};',error='UnsupportedValue')
 for owner in [1,64,1<<63,MAX]:add(f'owner_{owner}',owned=True,maximum=0,extra=f'store.space.objects[0].value=Value::Buffer {{buffer_storage:BufferStorage::Owned {{buffer_owner:{owner}}}}};',error='InvalidState')
 add('uninitialized',owned=True,maximum=0,extra='store.bytes.blocks[0].initialized=false;',error='InvalidState')
 for length in [0,11,1025,1<<63,MAX]:add(f'input_length_{length}',length=length,maximum=0,error='Bounds'if length<1024 else'Capacity')
 for unit in [0,1<<63,MAX]:add(f'unit_{unit}',unit=unit,maximum=0,error='Bounds')
 for start,end in [(12,10),(MAX,MAX),(0,MAX),(0,257)]:
  add(f'span_{start}_{end}',maximum=0,extra=f'store.space.objects[0].value=Value::Buffer {{buffer_storage:BufferStorage::Source {{declared_size:0,buffer_initializer:Span {{unit:7,start:{start},end:{end}}}}}}};',error='Capacity'if end==257 else'Bounds')
 add('owned_ignored_source',owned=True,length=MAX,unit=MAX,extra='store.bytes.blocks[0].bytes[254]=255;store.bytes.blocks[0].bytes[255]=253;')
 add('default_failure',error='InvalidState')
 assert len(rows)==len({r['name']for r in rows});return rows
IMPORTS='''use aml::object_to_string::to_string;use aml::object_to_string::StringResult;
use aml::object_conversions::ConversionFailure;
use aml::model::ObjectStore;use aml::model::Value;use aml::model::Span;use aml::model::Path;
use aml::model::StringStorage;use aml::model::BufferStorage;use aml::model::ReferenceKind;
'''
HELPERS='''data DefaultResult {value:StringResult;}
machine fill(bytes:&mut [u8;256],index:u64,count:u64,value:u8)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_=fill_one(bytes,index,count,value);transition index<count {true -> fill(bytes,index+1,count,value) _ -> (0)}}
machine fill_one(bytes:&mut [u8;256],index:u64,count:u64,value:u8)->u8 {transition index<count && index<256 {true -> put(bytes,index,value) _ -> (0)} state put(bytes:&mut [u8;256],index:u64,value:u8)->u8 {bytes[index]=value;0}}
machine source_bytes(input:&mut [u8;1024],bytes:&[u8;256],index:u64,count:u64)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_=source_one(input,bytes,index,count);transition index<count {true -> source_bytes(input,bytes,index+1,count) _ -> (0)}}
machine source_one(input:&mut [u8;1024],bytes:&[u8;256],index:u64,count:u64)->u8 {transition index<count && index<256 {true -> put(input,bytes,index) _ -> (0)} state put(input:&mut [u8;1024],bytes:&[u8;256],index:u64)->u8 {let at:u64=index+10;transition at<1024 {true -> write(input,bytes,index,at) _ -> (0)}} state write(input:&mut [u8;1024],bytes:&[u8;256],index:u64,at:u64)->u8 {transition index<256 && at<1024 {true -> store(input,bytes,index,at) _ -> (0)}} state store(input:&mut [u8;1024],bytes:&[u8;256],index:u64,at:u64)->u8 {input[at]=bytes[index];0}}
machine equal(actual:&[u8;256],expected:&[u8;256],index:u64,limit:u64,prior:bool)
terminates by(index,limit)->Nat::BoundedDistance;
->bool {let good:bool=equal_one(actual,expected,index);transition index<limit {true -> equal(actual,expected,index+1,limit,prior && good) _ -> (prior)}}
machine equal_one(actual:&[u8;256],expected:&[u8;256],index:u64)->bool {transition index<256 {true -> (actual[index]==expected[index]) _ -> (true)}}
machine is_string(value:StringResult,wanted:u64,expected:&[u8;256])->bool {transition value {StringResult::String {length,bytes} -> check(length,wanted,bytes,expected) _ -> (false)} state check(length:u64,wanted:u64,bytes:[u8;256],expected:&[u8;256])->bool {let good:bool=equal(&bytes,expected,0,256,true);length==wanted && good}}
machine is_failure(value:StringResult,error:ConversionFailure)->bool {transition value {StringResult::Failure {reason} -> (reason==error) _ -> (false)}}
'''
def initialized(name,data):
 padded=data+[0]*(256-len(data));base=Counter(padded).most_common(1)[0][0];code=f'let mut {name}:[u8;256];'
 if base:code+=f'_=fill(&mut {name},0,256,{base});'
 return code+''.join(f'{name}[{i}]={v};'for i,v in enumerate(padded)if v!=base)
def body(r,control):
 code=f'let mut input:[u8;1024];let mut store:ObjectStore=ObjectStore {{}};store.space.object_count={r["count"]};'+initialized('data',r['data'])
 kind=r['kind']
 if kind=='Buffer':
  if r['owned']:
   value='Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:0}}';code+=f'store.bytes.blocks[0].initialized=true;store.bytes.blocks[0].length={len(r["data"])};store.bytes.blocks[0].bytes=data;'
  else:
   value=f'Buffer {{buffer_storage:BufferStorage::Source {{declared_size:{r["declared"]},buffer_initializer:Span {{unit:7,start:10,end:{10+len(r["data"])}}}}}}}'
   code+=f'_=source_bytes(&mut input,&data,0,{len(r["data"])});'
 else:value={'Uninitialized':'Uninitialized','Integer':'Integer {number:0}','String':'String {string_storage:StringStorage::Owned {string_owner:0}}','Package':'Package {first:0,count:0}','NameReference':'NameReference {name:Path {},scope:Path {}}','BufferField':'BufferField {backing_object:0,bit_offset:0,bit_length:8}','Device':'Device','Method':'Method {flags:0,body:Span {}}'}[kind]
 code+='store.space.objects[0].value=Value::'+value+';'+r['extra']
 code+='let initial:DefaultResult=DefaultResult {};let result:StringResult=initial.value;'if r['name']=='default_failure'else f'let result:StringResult=to_string(&input,{r["length"]},{r["unit"]},&store,{r["object"]},{r["maximum"]});'
 if r['error']:
  error=('UnsupportedValue'if r['error']=='InvalidState'else'InvalidState')if control else r['error'];code+=f'let good:bool=is_failure(result,ConversionFailure::{error});'
 else:
  code+=initialized('expected',r['expected'])
  if control:code+='expected[255]='+str((r['expected'][255]if len(r['expected'])==256 else 0)^1)+';'
  code+=f'let good:bool=is_string(result,{len(r["expected"])},&expected);'
 return code+'transition good {true -> (0) _ -> (1)}'
if __name__=='__main__':(HERE/'cases.json').write_text(json.dumps(cases(),indent=2,sort_keys=True)+'\n');print(len(cases()),'pairs')
