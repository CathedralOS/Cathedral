import json
from collections import Counter
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4];MAX=(1<<64)-1

def oracle(kind,data,number,target,bits,declared):
 if kind not in ['Integer','Buffer','String']:return None,'UnsupportedValue'
 if kind=='Integer':
  value=number&((1<<bits)-1)
  return (value if target=='Integer'else list(value.to_bytes(bits//8,'little'))if target=='Buffer'else list(f'{value:0{bits//4}X}'.encode())),None
 if kind=='Buffer':data=data+[0]*max(0,declared-len(data))
 if kind=='String'and any(b==0 or b>=128 for b in data):return None,'Encoding'
 if target=='Integer':
  if not data:return None,'Empty'
  if kind=='Buffer':return int.from_bytes(bytes(data[:bits//8]),'little'),None
  prefix=[]
  for byte in data[:bits//4]:
   if chr(byte)not in '0123456789abcdefABCDEF':break
   prefix.append(chr(byte))
  return int(''.join(prefix),16)if prefix else 0,None
 if target=='Buffer':out=data+([0]if data and kind=='String'else [])
 else:out=data if kind=='String'else list(' '.join(f'{b:02X}'for b in data).encode())
 return (out,None)if len(out)<=256 else(None,'Capacity')
def cases():
 rows=[]
 def add(name,kind,target,data=b'',number=0,bits=64,owned=False,declared=None,error=None,extra='',object_count=1,object=0,source_length=None,unit=7):
  data=list(data);declared=len(data)if declared is None else declared;expected,natural=oracle(kind,data,number,target,bits,declared)
  rows.append(dict(name=name,kind=kind,target=target,data=data,number=number,bits=bits,owned=owned,declared=declared,expected=expected,error=error or natural,extra=extra,object_count=object_count,object=object,source_length=len(data)if source_length is None else source_length,unit=unit))
 targets=['Integer','Buffer','String']
 for bits in [32,64]:
  for target in targets:
   for number in [0,MAX,0x123456789abcdef0]:add(f'number_{bits}_{target}_{number}','Integer',target,number=number,bits=bits)
   for owned in [False,True]:
    for text in ['', '1234','0x10','aBcDeF01234567890']:
     add(f'string_{bits}_{target}_{owned}_{text}', 'String',target,data=text.encode(),bits=bits,owned=owned)
    for data in [b'',bytes([1,0x23,255]),bytes(range(1,10))]:
     add(f'buffer_{bits}_{target}_{owned}_{len(data)}','Buffer',target,data=data,bits=bits,owned=owned)
 for owned in [False,True]:
  for length in [255,256]:
   for target in ['Buffer','String']:add(f'string_capacity_{length}_{owned}_{target}','String',target,data=b'A'*length,owned=owned)
  for length in [85,86,256]:
   for target in ['Buffer','String']:add(f'buffer_capacity_{length}_{owned}_{target}','Buffer',target,data=bytes(range(length)),owned=owned)
 for target in targets:
  for kind in ['String','Buffer']:
   base=dict(data=b'12',owned=True)
   add(f'wrong_owner_{kind}_{target}',kind,target,**base,error='InvalidState',extra='store.space.objects[0].value=Value::'+kind+' {'+('string_storage:StringStorage::Owned {string_owner:1}'if kind=='String'else'buffer_storage:BufferStorage::Owned {buffer_owner:1}')+'};')
   add(f'uninitialized_{kind}_{target}',kind,target,**base,error='InvalidState',extra='store.bytes.blocks[0].initialized=false;')
   add(f'length_max_{kind}_{target}',kind,target,**base,error='Capacity',extra=f'store.bytes.blocks[0].length={MAX};')
   add(f'unit_{kind}_{target}',kind,target,data=b'12',unit=8,error='Bounds')
   add(f'source_max_{kind}_{target}',kind,target,data=b'12',source_length=MAX,error='Capacity')
   add(f'poison_unused_{kind}_{target}',kind,target,**base,source_length=MAX,extra='store.bytes.blocks[0].bytes[2]=254;store.bytes.blocks[0].bytes[255]=253;')
  for data in [b'1\0',b'12!\xff']:
   for owned in [False,True]:add(f'encoding_{target}_{owned}_{data.hex()}','String',target,data=data,owned=owned)
  for label,kwargs in [('empty',dict(object_count=0)),('count65',dict(object_count=65)),('count_high',dict(object_count=1<<63)),('count_max',dict(object_count=MAX)),('past_count',dict(object=1)),('id_high',dict(object=1<<63)),('id_max',dict(object=MAX))]:add(f'{label}_{target}','Integer',target,error='InvalidState',**kwargs)
  for kind in ['Uninitialized','Package','Method','OperationRegion','NameReference','BufferField','Device']:
   add(f'unsupported_{kind}_{target}',kind,target,error='UnsupportedValue')
  for ref in ['Named','Local','Arg','RefOf','Index','Unresolved']:
   add(f'reference_{ref}_{target}','Reference',target,error='UnsupportedValue',extra=f'store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{ref},object_id:0}};')
  add(f'padding_{target}','Buffer',target,data=b'AB',declared=4)
  add(f'initializer_larger_{target}','Buffer',target,data=b'ABCD',declared=1)
  add(f'number_unused_source_{target}','Integer',target,number=MAX,source_length=MAX)
 assert len(rows)==len({r['name']for r in rows});return rows
IMPORTS="""use aml::implicit_conversions::convert;
use aml::implicit_conversions::ConversionTarget;
use aml::implicit_conversions::ImplicitResult;
use aml::object_conversions::ConversionFailure;
use aml::model::ObjectStore;use aml::model::Value;use aml::model::Span;use aml::model::Path;
use aml::model::StringStorage;use aml::model::BufferStorage;use aml::model::ReferenceKind;
use integer_helpers::integers::IntegerSize;
"""
HELPERS="""machine fill(input:&mut [u8;256],index:u64,count:u64,byte:u8)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_=fill_at(input,index,count,byte);transition index<count {true -> fill(input,index+1,count,byte) _ -> (0)}}
machine fill_at(input:&mut [u8;256],index:u64,count:u64,byte:u8)->u8 {
 transition index<count && index<256 {true -> put(input,index,byte) _ -> (0)}
 state put(input:&mut [u8;256],index:u64,byte:u8)->u8 {input[index]=byte;0}
}
machine put_input(input:&mut [u8;1024],bytes:&[u8;256],index:u64,count:u64)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_=put_input_at(input,bytes,index,count);transition index<count {true -> put_input(input,bytes,index+1,count) _ -> (0)}}
machine put_input_at(input:&mut [u8;1024],bytes:&[u8;256],index:u64,count:u64)->u8 {
 transition index<count && index<256 {true -> put(input,bytes,index) _ -> (0)}
 state put(input:&mut [u8;1024],bytes:&[u8;256],index:u64)->u8 {input[index]=bytes[index];0}
}
machine equal_bytes(a:&[u8;256],b:&[u8;256],index:u64,count:u64,same:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let next:bool=equal_at(a,b,index,count,same);transition index<count {true -> equal_bytes(a,b,index+1,count,next) _ -> (same)}}
machine equal_at(a:&[u8;256],b:&[u8;256],index:u64,count:u64,same:bool)->bool {transition index<count && index<256 {true -> (same && a[index]==b[index]) _ -> (same)}}
machine expected_failure(value:ImplicitResult,error:ConversionFailure)->bool {transition value {ImplicitResult::Failure {reason} -> (reason==error) _ -> (false)}}
machine expected_integer(value:ImplicitResult,number:u64)->bool {transition value {ImplicitResult::Integer {number as actual} -> (actual==number) _ -> (false)}}
machine expected_bytes(value:ImplicitResult,target:ConversionTarget,length:u64,expected:&[u8;256])->bool {
 transition value {
  ImplicitResult::Buffer {length as actual_length,bytes} -> compare(&bytes,expected,actual_length,length,target==ConversionTarget::Buffer)
  ImplicitResult::String {length as actual_length,bytes} -> compare(&bytes,expected,actual_length,length,target==ConversionTarget::String)
  _ -> (false)
 }
 state compare(actual:&[u8;256],expected:&[u8;256],actual_length:u64,length:u64,kind:bool)->bool {let same:bool=equal_bytes(actual,expected,0,256,true);kind && actual_length==length && same}
}
"""
def initialized(name,data):
 padded=data+[0]*(256-len(data));base=Counter(padded).most_common(1)[0][0];code='let mut '+name+':[u8;256];'
 if base:code+=f'_=fill(&mut {name},0,256,{base});'
 return code+''.join(f'{name}[{i}]={v};'for i,v in enumerate(padded)if v!=base)
def body(r,control):
 data=r['data'];kind=r['kind'];code='let mut input:[u8;1024];let mut store:ObjectStore=ObjectStore {};'+initialized('data',data)+f'_=put_input(&mut input,&data,0,{len(data)});store.space.object_count={r["object_count"]};'
 if kind in ['String','Buffer']:
  if r['owned']:
   value='String {string_storage:StringStorage::Owned {string_owner:0}}'if kind=='String'else'Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:0}}'
   code+=f'store.bytes.blocks[0].initialized=true;store.bytes.blocks[0].length={len(data)};store.bytes.blocks[0].bytes=data;'
  else:value=f'String {{string_storage:StringStorage::Source {{string_source:Span {{unit:7,end:{len(data)}}}}}}}'if kind=='String'else f'Buffer {{buffer_storage:BufferStorage::Source {{declared_size:{r["declared"]},buffer_initializer:Span {{unit:7,end:{len(data)}}}}}}}'
 elif kind=='Integer':value=f'Integer {{number:{r["number"]}}}'
 else:value={'Uninitialized':'Uninitialized','Package':'Package {first:0,count:0}','Method':'Method {flags:0,body:Span {}}','OperationRegion':'OperationRegion {space:0,base:0,length:0,scope:Path {}}','NameReference':'NameReference {name:Path {},scope:Path {}}','BufferField':'BufferField {backing_object:0,bit_offset:0,bit_length:0}','Device':'Device','Reference':'Uninitialized'}[kind]
 code+='store.space.objects[0].value=Value::'+value+';'+r['extra']
 size='FourBytes'if r['bits']==32 else'EightBytes';target=r['target']
 code+=f'let value:ImplicitResult=convert(&input,{r["source_length"]},{r["unit"]},&store,{r["object"]},IntegerSize::{size},ConversionTarget::{target});'
 if r['error']:
  error=('UnsupportedValue'if r['error']=='InvalidState'else'InvalidState')if control else r['error'];code+=f'let good:bool=expected_failure(value,ConversionFailure::{error});'
 elif target=='Integer':code+=f'let good:bool=expected_integer(value,{r["expected"]^int(control)});'
 else:
  expected=r['expected'];out=expected+[0]*(256-len(expected))
  if control:out[-1]^=1
  code+=initialized('expected',out)+f'let good:bool=expected_bytes(value,ConversionTarget::{target},{len(expected)},&expected);'
 return code+'transition good {true -> (0) _ -> (1)}'
if __name__=='__main__':
 (HERE/'cases.json').write_text(json.dumps(cases(),indent=2,sort_keys=True)+'\n');print(len(cases()),'implicit dispatch pairs')
