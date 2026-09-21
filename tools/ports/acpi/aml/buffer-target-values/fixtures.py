#!/usr/bin/env python3
"""Original target-extent preparation vectors with independent byte expectations."""
from collections import Counter
from pathlib import Path
import json
HERE=Path(__file__).resolve().parent;MAX=(1<<64)-1

def cases():
 rows=[]
 def add(name,kind='Integer',number=0,data=b'AB',bits=64,extent=8,owned=False,**kw):
  r=dict(name=name,kind=kind,number=number,data=list(data),bits=bits,extent=extent,owned=owned,count=1,source=0,slot=0,length=1024,unit=7,extra='',error=None);r.update(kw)
  if not r['error']:
   if extent==0:r['error']='Bounds'
   elif extent>256:r['error']='Capacity'
   elif kind not in ['Integer','String']:r['error']='UnsupportedValue'
   elif kind=='String'and any(x==0 or x>127 for x in r['data']):r['error']='Encoding'
   elif kind=='String'and not r['data']:r['error']='Empty'
   else:
    raw=(number&((1<<bits)-1)).to_bytes(bits//8,'little')if kind=='Integer'else bytes(data)
    r['expected']=list(raw[:extent])+[0]*max(0,extent-len(raw))
  rows.append(r)
 for bits in [32,64]:
  for number in [0,MAX,1<<32,0x8877665544332211]:
   for extent in [1,3,4,5,8,9,255,256]:add(f'integer_{bits}_{number}_{extent}',bits=bits,number=number,extent=extent)
 for owned in [False,True]:
  for length in [1,2,4,7,8,255,256]:
   data=bytes(65+i%26 for i in range(length))
   for extent in [1,3,8,255,256]:add(f'string_{int(owned)}_{length}_{extent}',kind='String',data=data,extent=extent,owned=owned)
  for bad in [b'A\0',b'A\xff',b'A'*255+b'\x80']:
   add(f'encoding_full_{int(owned)}_{len(bad)}_{bad[-1]}',kind='String',data=bad,extent=1,owned=owned)
  add(f'empty_{int(owned)}',kind='String',data=b'',owned=owned)
 for extent in [0,257,1<<63,MAX]:
  for kind in ['Integer','String','Buffer']:
   add(f'extent_before_type_storage_{extent}_{kind}',kind=kind,data=b'\xff',extent=extent,owned=True)
 for count in [0,65,1<<63,MAX]:
  for extent in [0,8]:add(f'count_{count}_{extent}',count=count,extent=extent,error='InvalidState')
 for source in [1,64,1<<63,MAX]:add(f'source_{source}',source=source,error='InvalidState')
 for kind in ['Buffer','Uninitialized','Package','NameReference','BufferField','Device','Method']:
  add('unsupported_'+kind,kind=kind)
 for ref in ['Named','Local','Arg','RefOf','Index','Unresolved']:
  add('reference_'+ref,extra=f'store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{ref},object_id:0}};',error='UnsupportedValue')
 for owner in [1,64,1<<63,MAX]:add(f'owner_{owner}',kind='String',data=b'',owned=True,extra=f'store.space.objects[0].value=Value::String {{string_storage:StringStorage::Owned {{string_owner:{owner}}}}};',error='InvalidState')
 add('missing_owned_before_empty',kind='String',data=b'',owned=True,extra='store.bytes.blocks[0].initialized=false;',error='InvalidState')
 for n in [257,1<<63,MAX]:add(f'owned_capacity_{n}',kind='String',owned=True,extra=f'store.bytes.blocks[0].length={n};',error='Capacity')
 for n in [0,11,1025,1<<63,MAX]:add(f'input_length_{n}',kind='String',length=n,error='Bounds'if n<1024 else'Capacity')
 for unit in [0,1<<63,MAX]:add(f'unit_{unit}',kind='String',unit=unit,error='Bounds')
 for start,end in [(12,10),(MAX,MAX),(0,MAX),(0,257)]:
  add(f'span_{start}_{end}',kind='String',extra=f'store.space.objects[0].value=Value::String {{string_storage:StringStorage::Source {{string_source:Span {{unit:7,start:{start},end:{end}}}}}}};',error='Capacity'if end==257 else'Bounds')
 add('owned_ignored_source_tail',kind='String',owned=True,length=MAX,unit=MAX,extra='store.bytes.blocks[0].bytes[255]=255;')
 add('integer_ignored_source',number=MAX,bits=32,extent=9,length=MAX,unit=MAX)
 add('last_integer_slot',slot=63,source=63,count=64,number=MAX,extent=3)
 add('last_owned_string_slot',kind='String',slot=63,source=63,count=64,owned=True,extent=256)
 add('default_failure',error='InvalidState')
 assert len(rows)==len({r['name']for r in rows});return rows
IMPORTS='''use aml::buffer_target_values::prepare_buffer_extent;
use aml::implicit_conversions::ImplicitResult;use aml::object_conversions::ConversionFailure;
use aml::model::ObjectStore;use aml::model::Value;use aml::model::Span;use aml::model::Path;
use aml::model::StringStorage;use aml::model::BufferStorage;use aml::model::ReferenceKind;
use integer_helpers::integers::IntegerSize;
'''
HELPERS='''data DefaultResult {value:ImplicitResult;}
machine fill(bytes:&mut [u8;256],index:u64,count:u64,value:u8)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_=fill_one(bytes,index,count,value);transition index<count {true -> fill(bytes,index+1,count,value) _ -> (0)}}
machine fill_one(bytes:&mut [u8;256],index:u64,count:u64,value:u8)->u8 {transition index<count && index<256 {true -> put(bytes,index,value) _ -> (0)} state put(bytes:&mut [u8;256],index:u64,value:u8)->u8 {bytes[index]=value;0}}
machine source_bytes(input:&mut [u8;1024],bytes:&[u8;256],index:u64,count:u64)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_=source_one(input,bytes,index,count);transition index<count {true -> source_bytes(input,bytes,index+1,count) _ -> (0)}}
machine source_one(input:&mut [u8;1024],bytes:&[u8;256],index:u64,count:u64)->u8 {transition index<count && index<256 {true -> position(input,bytes,index,index+10) _ -> (0)} state position(input:&mut [u8;1024],bytes:&[u8;256],index:u64,at:u64)->u8 {transition at<1024 {true -> put(input,bytes,index,at) _ -> (0)}} state put(input:&mut [u8;1024],bytes:&[u8;256],index:u64,at:u64)->u8 {input[at]=bytes[index];0}}
machine equal(actual:&[u8;256],expected:&[u8;256],index:u64,limit:u64,prior:bool)
terminates by(index,limit)->Nat::BoundedDistance;
->bool {let good:bool=equal_one(actual,expected,index);transition index<limit {true -> equal(actual,expected,index+1,limit,prior && good) _ -> (prior)}}
machine equal_one(actual:&[u8;256],expected:&[u8;256],index:u64)->bool {transition index<256 {true -> (actual[index]==expected[index]) _ -> (true)}}
machine is_buffer(value:ImplicitResult,wanted:u64,expected:&[u8;256])->bool {transition value {ImplicitResult::Buffer {length,bytes} -> check(length,wanted,bytes,expected) _ -> (false)} state check(length:u64,wanted:u64,bytes:[u8;256],expected:&[u8;256])->bool {let good:bool=equal(&bytes,expected,0,256,true);length==wanted && good}}
machine is_failure(value:ImplicitResult,error:ConversionFailure)->bool {transition value {ImplicitResult::Failure {reason} -> (reason==error) _ -> (false)}}
'''
def initialized(name,data):
 padded=data+[0]*(256-len(data));n=0
 while n<256 and padded[n]==65+n%26:n+=1
 if n>=16 and not any(padded[n:]):return f'let mut {name}:[u8;256];_=alphabet(&mut {name},0,{n});'
 base=Counter(padded).most_common(1)[0][0];code=f'let mut {name}:[u8;256];'
 if base:code+=f'_=fill(&mut {name},0,256,{base});'
 return code+''.join(f'{name}[{i}]={v};'for i,v in enumerate(padded)if v!=base)
def body(r,control):
 slot=r['slot'];code=f'let mut input:[u8;1024];let mut store:ObjectStore=ObjectStore {{}};store.space.object_count={r["count"]};'+initialized('data',r['data']);kind=r['kind']
 if kind=='String':
  if r['owned']:
   value=f'String {{string_storage:StringStorage::Owned {{string_owner:{slot}}}}}';code+=f'store.bytes.blocks[{slot}].initialized=true;store.bytes.blocks[{slot}].length={len(r["data"])};store.bytes.blocks[{slot}].bytes=data;'
  else:
   value=f'String {{string_storage:StringStorage::Source {{string_source:Span {{unit:7,start:10,end:{10+len(r["data"])}}}}}}}'
   code+=f'_=source_bytes(&mut input,&data,0,{len(r["data"])});'
 elif kind=='Integer':value=f'Integer {{number:{r["number"]}}}'
 else:value={'Uninitialized':'Uninitialized','Buffer':'Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:0}}','Package':'Package {first:0,count:0}','NameReference':'NameReference {name:Path {},scope:Path {}}','BufferField':'BufferField {backing_object:0,bit_offset:0,bit_length:8}','Device':'Device','Method':'Method {flags:0,body:Span {}}'}[kind]
 code+=f'store.space.objects[{slot}].value=Value::'+value+';'+r['extra'];size='FourBytes'if r['bits']==32 else'EightBytes'
 code+='let initial:DefaultResult=DefaultResult {};let result:ImplicitResult=initial.value;'if r['name']=='default_failure'else f'let result:ImplicitResult=prepare_buffer_extent(&input,{r["length"]},{r["unit"]},&store,{r["source"]},IntegerSize::{size},{r["extent"]});'
 if r['error']:
  error=('UnsupportedValue'if r['error']=='InvalidState'else'InvalidState')if control else r['error'];code+=f'let good:bool=is_failure(result,ConversionFailure::{error});'
 else:
  code+=initialized('expected',r['expected'])
  if control:code+='expected[255]='+str((r['expected'][255]if len(r['expected'])==256 else 0)^1)+';'
  code+=f'let good:bool=is_buffer(result,{r["extent"]},&expected);'
 return code+'transition good {true -> (0) _ -> (1)}'
if __name__=='__main__':(HERE/'cases.json').write_text(json.dumps(cases(),indent=2,sort_keys=True)+'\n');print(len(cases()),'pairs')

HELPERS+="""
machine alphabet(bytes:&mut [u8;256],index:u64,count:u64)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_=alphabet_one(bytes,index,count);transition index<count {true -> alphabet(bytes,index+1,count) _ -> (0)}}
machine alphabet_one(bytes:&mut [u8;256],index:u64,count:u64)->u8 {
 transition index<count && index<256 {true -> put(bytes,index) _ -> (0)}
 state put(bytes:&mut [u8;256],index:u64)->u8 {let value:u64=65+index%26;bytes[index]=value as u8;0}
}
"""
