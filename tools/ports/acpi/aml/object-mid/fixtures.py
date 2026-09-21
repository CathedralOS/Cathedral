import json
from pathlib import Path
from collections import Counter
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4];MAX=(1<<64)-1

def cases():
 rows=[]
 def add(name,kind='Buffer',data=b'ABCD',owned=False,index=0,requested=MAX,error=None,extra='',count=1,source=0,length=512,unit=7,expected=None):
  values=list(data)
  if error is None and kind=='String' and any(b==0 or b>=128 for b in values):error='Encoding'
  if error is None and kind not in ['String','Buffer']:error='UnsupportedValue'
  result=values[index:index+requested] if expected is None else list(expected)
  rows.append(dict(name=name,kind=kind,data=values,owned=owned,index=index,requested=requested,error=error,extra=extra,count=count,source=source,length=length,unit=unit,expected=result))
 for kind in ['Buffer','String']:
  for owned in [False,True]:
   for length in [0,4,129,256]:
    data=(bytes([0])*64+bytes([255])*64+bytes([128])*64+bytes([1])*64)[:length] if kind=='Buffer' else (b'A'*64+b'Z'*64+b'0'*64+b'! '*32)[:length]
    for index in sorted(set([0,1,max(0,length-1),length,length+1,MAX])):
     for requested in [0,1,MAX]:add(f'matrix_{kind}_{owned}_{length}_{index}_{requested}',kind,data,owned,index,requested)
 for owned in [False,True]:
  for data in [b'AB'+bytes([255]),b'A'+bytes([0])+b'B']:
   for index,requested in [(0,1),(0,0),(MAX,MAX)]:add(f'bad_tail_{owned}_{data.hex()}_{index}_{requested}','String',data,owned,index,requested)
 for count in [0,65,1<<63,MAX]:add(f'count_{count}',count=count,error='InvalidState')
 for source in [1,1<<63,MAX]:add(f'source_{source}',source=source,error='InvalidState')
 for kind in ['Integer','Uninitialized','Package','Method','Device','BufferField','NameReference']:add('unsupported_'+kind,kind)
 for ref in ['Named','Local','Arg','RefOf','Index','Unresolved']:
  add('reference_'+ref,error='UnsupportedValue',extra=f'store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{ref},object_id:0}};')
 for kind in ['Buffer','String']:
  prefix='buffer' if kind=='Buffer' else 'string'
  for index in [0,MAX]:
   add(f'owned_owner_{kind}_{index}',kind,owned=True,index=index,error='InvalidState',extra=f'store.space.objects[0].value=Value::{kind} {{{prefix}_storage:{kind}Storage::Owned {{{prefix}_owner:1}}}};')
   add(f'owned_missing_{kind}_{index}',kind,owned=True,index=index,error='InvalidState',extra='store.bytes.blocks[0].initialized=false;')
   add(f'owned_extent_{kind}_{index}',kind,owned=True,index=index,error='Capacity',extra=f'store.bytes.blocks[0].length={MAX};')
   add(f'source_unit_{kind}_{index}',kind,index=index,unit=8,error='Bounds')
   add(f'source_extent_{kind}_{index}',kind,index=index,length=MAX,error='Capacity')
  add('owned_unused_'+kind,kind,owned=True,length=MAX,unit=MAX,extra='store.bytes.blocks[0].bytes[255]=255;')
 add('virtual_padding',extra='store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:9,buffer_initializer:Span {unit:7,start:0,end:4}}};',index=2,expected=b'CD'+bytes(5))
 add('oversized_initializer',extra='store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:1,buffer_initializer:Span {unit:7,start:0,end:4}}};',index=2,expected=b'CD')
 for label,start,end,declared,error in [('reversed',4,2,4,'Bounds'),('past_input',0,513,4,'Bounds'),('max_start',MAX,MAX,4,'Bounds'),('max_end',0,MAX,4,'Bounds'),('long_initializer',0,257,4,'Capacity'),('long_declared',0,4,MAX,'Capacity')]:
  add('malformed_'+label,index=MAX,requested=0,error=error,extra=f'store.space.objects[0].value=Value::Buffer {{buffer_storage:BufferStorage::Source {{declared_size:{declared},buffer_initializer:Span {{unit:7,start:{start},end:{end}}}}}}};')
 add('default_failure',error='InvalidState')
 assert len(rows)==len({r['name']for r in rows});return rows
IMPORTS="""use aml::object_mid::extract;use aml::object_mid::Portion;
use aml::object_conversions::ConversionFailure;
use aml::model::ObjectStore;use aml::model::Value;use aml::model::Span;use aml::model::Path;
use aml::model::StringStorage;use aml::model::BufferStorage;use aml::model::ReferenceKind;
"""
HELPERS='''data DefaultResult {value:Portion;}
machine fill(bytes:&mut [u8;256],index:u64,count:u64,value:u8)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_=fill_one(bytes,index,count,value);transition index<count {true -> fill(bytes,index+1,count,value) _ -> (0)}}
machine fill_one(bytes:&mut [u8;256],index:u64,count:u64,value:u8)->u8 {transition index<count && index<256 {true -> put(bytes,index,value) _ -> (0)} state put(bytes:&mut [u8;256],index:u64,value:u8)->u8 {bytes[index]=value;0}}
machine source_bytes(input:&mut [u8;1024],bytes:&[u8;256],offset:u64,index:u64,count:u64)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_=source_one(input,bytes,offset,index,count);transition index<count {true -> source_bytes(input,bytes,offset,index+1,count) _ -> (0)}}
machine source_one(input:&mut [u8;1024],bytes:&[u8;256],offset:u64,index:u64,count:u64)->u8 {transition index<count && index<256 && offset<=256 {true -> position(input,bytes,offset+index,index) _ -> (0)} state position(input:&mut [u8;1024],bytes:&[u8;256],at:u64,index:u64)->u8 {transition at<1024 {true -> put(input,bytes,at,index) _ -> (0)}} state put(input:&mut [u8;1024],bytes:&[u8;256],at:u64,index:u64)->u8 {input[at]=bytes[index];0}}
machine is_failure(value:Portion,error:ConversionFailure)->bool {transition value {Portion::Failure {reason} -> (reason==error) _ -> (false)}}
machine expected_bytes(value:Portion,string:bool,length:u64,expected:&[u8;256])->bool {
 transition value {Portion::Buffer {length as actual_length,bytes} -> bytes(&bytes,expected,actual_length,length,!string) Portion::String {length as actual_length,bytes} -> bytes(&bytes,expected,actual_length,length,string) _ -> (false)}
 state bytes(actual:&[u8;256],expected:&[u8;256],actual_length:u64,length:u64,kind:bool)->bool {let same:bool=equal_bytes(actual,expected,0,256,true);kind && actual_length==length && same}
}
machine equal_bytes(a:&[u8;256],b:&[u8;256],index:u64,count:u64,same:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let next:bool=equal_at(a,b,index,count,same);transition index<count {true -> equal_bytes(a,b,index+1,count,next) _ -> (same)}}
machine equal_at(a:&[u8;256],b:&[u8;256],index:u64,count:u64,same:bool)->bool {transition index<count && index<256 {true -> (same && a[index]==b[index]) _ -> (same)}}

'''
def initialized(name,data):
 padded=data+[0]*(256-len(data));base=Counter(padded).most_common(1)[0][0];code=f'let mut {name}:[u8;256];'
 if base:code+=f'_=fill(&mut {name},0,256,{base});'
 i=0
 while i<256:
  if padded[i]==base:i+=1;continue
  end=i+1
  while end<256 and padded[end]==padded[i]:end+=1
  if end-i>=4:code+=f'_=fill(&mut {name},{i},{end},{padded[i]});'
  else:code+=''.join(f'{name}[{j}]={padded[j]};'for j in range(i,end))
  i=end
 return code
def body(r,control):
 code=f'let mut input:[u8;1024];let mut store:ObjectStore=ObjectStore {{}};store.space.object_count={r["count"]};'
 kind=r['kind'];data=r['data'];code+=initialized('data0',data)
 if kind in ['String','Buffer']:
  prefix='string'if kind=='String'else'buffer'
  if r['owned']:
   value=f'{kind} {{{prefix}_storage:{kind}Storage::Owned {{{prefix}_owner:0}}}}'
   code+=f'store.bytes.blocks[0].initialized=true;store.bytes.blocks[0].length={len(data)};store.bytes.blocks[0].bytes=data0;'
  else:
   code+=f'_=source_bytes(&mut input,&data0,0,0,{len(data)});';span=f'Span {{unit:7,start:0,end:{len(data)}}}'
   value=f'String {{string_storage:StringStorage::Source {{string_source:{span}}}}}'if kind=='String'else f'Buffer {{buffer_storage:BufferStorage::Source {{declared_size:{len(data)},buffer_initializer:{span}}}}}'
 else:value={'Integer':'Integer {number:42}','Uninitialized':'Uninitialized','Package':'Package {first:0,count:0}','Method':'Method {flags:0,body:Span {}}','NameReference':'NameReference {name:Path {},scope:Path {}}','BufferField':'BufferField {backing_object:0,bit_offset:0,bit_length:8}','Device':'Device'}[kind]
 code+=f'store.space.objects[0].value=Value::{value};'+r['extra']
 code+='let initial:DefaultResult=DefaultResult {};let value:Portion=initial.value;'if r['name']=='default_failure'else f'let value:Portion=extract(&input,{r["length"]},{r["unit"]},&store,{r["source"]},{r["index"]},{r["requested"]});'
 if r['error']:
  error=('UnsupportedValue'if r['error']=='InvalidState'else'InvalidState')if control else r['error'];code+=f'let good:bool=is_failure(value,ConversionFailure::{error});'
 else:
  expected=r['expected']+[0]*(256-len(r['expected']))
  if control:expected[-1]^=1
  code+=initialized('expected',expected)+f'let good:bool=expected_bytes(value,{str(kind=="String").lower()},{len(r["expected"])},&expected);'
 return code+'transition good {true -> (0) _ -> (1)}'
