import json
from pathlib import Path
from collections import Counter
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4];MAX=(1<<64)-1

def node(kind,data=b'',number=0):return dict(kind=kind,data=list(data),number=number)
def converted(n,target,bits):
 kind=n['kind'];data=n['data']
 if kind not in ['Integer','Buffer','String']:return None,'UnsupportedValue'
 if kind=='Integer':
  value=n['number']&((1<<bits)-1)
  return value if target=='Integer'else list(value.to_bytes(bits//8,'little'))if target=='Buffer'else list(f'{value:0{bits//4}X}'.encode()),None
 if kind=='String'and any(b==0 or b>=128 for b in data):return None,'Encoding'
 if target=='Integer':
  if not data:return None,'Empty'
  if kind=='Buffer':return int.from_bytes(bytes(data[:bits//8]),'little'),None
  digits=[]
  for byte in data[:bits//4]:
   if chr(byte)not in '0123456789abcdefABCDEF':break
   digits.append(chr(byte))
  return int(''.join(digits),16)if digits else 0,None
 if target=='Buffer':out=data+([0]if kind=='String'and data else [])
 else:out=data if kind=='String'else list(' '.join(f'{b:02X}'for b in data).encode())
 return (out,None)if len(out)<=256 else(None,'Capacity')
def cases():
 rows=[]
 def add(name,a,b,bits=64,owned=False,error=None,extra='',count=2,left=0,right=1,length=512,unit=7):
  x,e=converted(a,a['kind'],bits);y,f=converted(b,a['kind'],bits)
  failure=error or e or f
  order=None if failure else 'Less'if x<y else'Greater'if x>y else'Equal'
  rows.append(dict(name=name,a=a,b=b,bits=bits,owned=owned,error=failure,order=order,extra=extra,count=count,left=left,right=right,length=length,unit=unit))
 samples=[node('Integer',number=0),node('Integer',number=0x1234),node('Integer',number=MAX),node('String',b''),node('String',b'1234'),node('String',b'0x10'),node('String',b'0000000000001234'),node('Buffer',b''),node('Buffer',bytes([0x34,0x12])),node('Buffer',b'1234')]
 for bits in [32,64]:
  for i,a in enumerate(samples):
   for j,b in enumerate(samples):add(f'matrix_{bits}_{i}_{j}',a,b,bits,owned=(i+j)%2==0)
 for bits in [32,64]:
  for a,b in [(MAX,0),(1<<63,MAX),(0xffffffff,1<<32),(1<<32,0)]:add(f'unsigned_{bits}_{a}_{b}',node('Integer',number=a),node('Integer',number=b),bits)
 for a,b in [(b'Z',b'AA'),(b'A',b'A\0'),(b'\xff',b'\x80\xff'),(b'AB',b'A'),(b'\0',b'')]:add(f'lex_{a.hex()}_{b.hex()}',node('Buffer',a),node('Buffer',b))
 for owned in [False,True]:
  for length in [85,86]:add(f'format_capacity_{owned}_{length}',node('String',b'00'),node('Buffer',bytes([0])*length),owned=owned)
  for length in [255,256]:add(f'terminator_capacity_{owned}_{length}',node('Buffer',b'A'),node('String',b'A'*length),owned=owned)
  for side in ['a','b']:
   for bad in [b'1\0',b'12!\xff']:
    a=node('String',bad if side=='a'else b'AB');b=node('String',bad if side=='b'else b'AB');add(f'encoding_{owned}_{side}_{bad.hex()}',a,b,owned=owned)
  add(f'left_encoding_precedence_{owned}',node('String',b'\xff'),node('Integer'),owned=owned,error='Encoding',right=MAX)
  add(f'empty_integer_error_{owned}',node('Integer'),node('Buffer'),owned=owned)
 for count in [0,1,65,1<<63,MAX]:add(f'count_{count}',node('Integer'),node('Integer'),count=count,error='InvalidState')
 for side in ['left','right']:
  for index in [2,1<<63,MAX]:add(f'{side}_{index}',node('Integer'),node('Integer'),error='InvalidState',**{side:index})
 for side in [0,1]:
  for kind in ['Uninitialized','Package','NameReference','BufferField','Method','Device']:
   a=node(kind)if side==0 else node('Integer');b=node(kind)if side==1 else node('Integer');add(f'unsupported_{side}_{kind}',a,b)
  for ref in ['Named','Local','Arg','RefOf','Index','Unresolved']:
   add(f'ref_{side}_{ref}',node('Integer'),node('Integer'),error='UnsupportedValue',extra=f'store.space.objects[{side}].value=Value::Reference {{kind:ReferenceKind::{ref},object_id:{1-side}}};')
  for label,extra,error in [('owner',f'store.space.objects[{side}].value=Value::String {{string_storage:StringStorage::Owned {{string_owner:{1-side}}}}};','InvalidState'),('uninitialized',f'store.bytes.blocks[{side}].initialized=false;','InvalidState'),('capacity',f'store.bytes.blocks[{side}].length={MAX};','Capacity')]:
   add(f'owned_{side}_{label}',node('String',b'AB'),node('String',b'AB'),owned=True,error=error,extra=extra)
 add('source_unit',node('String',b'A'),node('String',b'A'),unit=8,error='Bounds')
 add('source_length_max',node('String',b'A'),node('String',b'A'),length=MAX,error='Capacity')
 add('owned_unused_source',node('String',b'A'),node('String',b'A'),owned=True,length=MAX,unit=MAX,extra='store.bytes.blocks[0].bytes[255]=255;store.bytes.blocks[1].bytes[254]=254;')
 add('default_failure',node('Integer'),node('Integer'),error='InvalidState',extra='')
 assert len(rows)==len({r['name']for r in rows});return rows
IMPORTS='''use aml::object_comparison::compare;use aml::object_comparison::ObjectComparison;
use aml::object_conversions::ConversionFailure;
use aml::model::ObjectStore;use aml::model::Value;use aml::model::Span;use aml::model::Path;
use aml::model::StringStorage;use aml::model::BufferStorage;use aml::model::ReferenceKind;
use integer_helpers::integers::IntegerSize;
'''
HELPERS='''data DefaultResult {value:ObjectComparison;}
machine fill(bytes:&mut [u8;256],index:u64,count:u64,value:u8)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_=fill_one(bytes,index,count,value);transition index<count {true -> fill(bytes,index+1,count,value) _ -> (0)}}
machine fill_one(bytes:&mut [u8;256],index:u64,count:u64,value:u8)->u8 {transition index<count && index<256 {true -> put(bytes,index,value) _ -> (0)} state put(bytes:&mut [u8;256],index:u64,value:u8)->u8 {bytes[index]=value;0}}
machine source_bytes(input:&mut [u8;1024],bytes:&[u8;256],offset:u64,index:u64,count:u64)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_=source_one(input,bytes,offset,index,count);transition index<count {true -> source_bytes(input,bytes,offset,index+1,count) _ -> (0)}}
machine source_one(input:&mut [u8;1024],bytes:&[u8;256],offset:u64,index:u64,count:u64)->u8 {transition index<count && index<256 && offset<=256 {true -> position(input,bytes,offset+index,index) _ -> (0)} state position(input:&mut [u8;1024],bytes:&[u8;256],at:u64,index:u64)->u8 {transition at<1024 {true -> put(input,bytes,at,index) _ -> (0)}} state put(input:&mut [u8;1024],bytes:&[u8;256],at:u64,index:u64)->u8 {input[at]=bytes[index];0}}
machine is_failure(value:ObjectComparison,error:ConversionFailure)->bool {transition value {ObjectComparison::Failure {reason} -> (reason==error) _ -> (false)}}
machine order(value:ObjectComparison)->u8 {transition value {ObjectComparison::Less -> (1) ObjectComparison::Equal -> (2) ObjectComparison::Greater -> (3) _ -> (0)}}
'''
def initialized(name,data):
 padded=data+[0]*(256-len(data));base=Counter(padded).most_common(1)[0][0];code=f'let mut {name}:[u8;256];'
 if base:code+=f'_=fill(&mut {name},0,256,{base});'
 return code+''.join(f'{name}[{i}]={v};'for i,v in enumerate(padded)if v!=base)
def body(r,control):
 code=f'let mut input:[u8;1024];let mut store:ObjectStore=ObjectStore {{}};store.space.object_count={r["count"]};'
 for i,n in enumerate([r['a'],r['b']]):
  kind=n['kind'];data=n['data'];offset=i*256;code+=initialized(f'data{i}',data)
  if kind in ['String','Buffer']:
   if r['owned']:
    value=f'{kind} {{'+(f'string_storage:StringStorage::Owned {{string_owner:{i}}}'if kind=='String'else f'buffer_storage:BufferStorage::Owned {{buffer_owner:{i}}}')+'}'
    code+=f'store.bytes.blocks[{i}].initialized=true;store.bytes.blocks[{i}].length={len(data)};store.bytes.blocks[{i}].bytes=data{i};'
   else:
    code+=f'_=source_bytes(&mut input,&data{i},{offset},0,{len(data)});';span=f'Span {{unit:7,start:{offset},end:{offset+len(data)}}}'
    value=f'String {{string_storage:StringStorage::Source {{string_source:{span}}}}}'if kind=='String'else f'Buffer {{buffer_storage:BufferStorage::Source {{declared_size:{len(data)},buffer_initializer:{span}}}}}'
  elif kind=='Integer':value=f'Integer {{number:{n["number"]}}}'
  else:value={'Uninitialized':'Uninitialized','Package':'Package {first:0,count:0}','Method':'Method {flags:0,body:Span {}}','NameReference':'NameReference {name:Path {},scope:Path {}}','BufferField':'BufferField {backing_object:0,bit_offset:0,bit_length:0}','Device':'Device'}[kind]
  code+=f'store.space.objects[{i}].value=Value::{value};'
 code+=r['extra'];size='FourBytes'if r['bits']==32 else'EightBytes'
 code+='let initial:DefaultResult=DefaultResult {};let value:ObjectComparison=initial.value;'if r['name']=='default_failure'else f'let value:ObjectComparison=compare(&input,{r["length"]},{r["unit"]},&store,{r["left"]},{r["right"]},IntegerSize::{size});'
 if r['error']:
  error=('UnsupportedValue'if r['error']=='InvalidState'else'InvalidState')if control else r['error'];code+=f'let good:bool=is_failure(value,ConversionFailure::{error});'
 else:
  expected={'Less':1,'Equal':2,'Greater':3}[r['order']];expected=expected%3+1 if control else expected;code+=f'let actual:u8=order(value);let good:bool=actual=={expected};'
 return code+'transition good {true -> (0) _ -> (1)}'
if __name__=='__main__':
 (HERE/'cases.json').write_text(json.dumps(cases(),indent=2,sort_keys=True)+'\n');print(len(cases()),'comparison pairs')
