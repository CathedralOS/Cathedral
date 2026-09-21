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
IMPORTS='''use aml::object_logic::binary;use aml::object_logic::negate;use aml::object_logic::LogicalResult;
use aml::object_conversions::ConversionFailure;
use aml::model::ObjectStore;use aml::model::Value;use aml::model::Span;use aml::model::Path;
use aml::model::StringStorage;use aml::model::BufferStorage;use aml::model::ReferenceKind;
use integer_helpers::integers::IntegerSize;use integer_helpers::integers::Logical;
'''
HELPERS='''data DefaultResult {value:LogicalResult;}
machine fill(bytes:&mut [u8;256],index:u64,count:u64,value:u8)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_=fill_one(bytes,index,count,value);transition index<count {true -> fill(bytes,index+1,count,value) _ -> (0)}}
machine fill_one(bytes:&mut [u8;256],index:u64,count:u64,value:u8)->u8 {transition index<count && index<256 {true -> put(bytes,index,value) _ -> (0)} state put(bytes:&mut [u8;256],index:u64,value:u8)->u8 {bytes[index]=value;0}}
machine source_bytes(input:&mut [u8;1024],bytes:&[u8;256],offset:u64,index:u64,count:u64)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_=source_one(input,bytes,offset,index,count);transition index<count {true -> source_bytes(input,bytes,offset,index+1,count) _ -> (0)}}
machine source_one(input:&mut [u8;1024],bytes:&[u8;256],offset:u64,index:u64,count:u64)->u8 {transition index<count && index<256 && offset<=256 {true -> position(input,bytes,offset+index,index) _ -> (0)} state position(input:&mut [u8;1024],bytes:&[u8;256],at:u64,index:u64)->u8 {transition at<1024 {true -> put(input,bytes,at,index) _ -> (0)}} state put(input:&mut [u8;1024],bytes:&[u8;256],at:u64,index:u64)->u8 {input[at]=bytes[index];0}}
machine is_failure(value:LogicalResult,error:ConversionFailure)->bool {transition value {LogicalResult::Failure {reason} -> (reason==error) _ -> (false)}}
machine is_integer(value:LogicalResult,expected:u64)->bool {transition value {LogicalResult::Integer {number} -> (number==expected) _ -> (false)}}
'''
def initialized(name,data):
 padded=data+[0]*(256-len(data));base=Counter(padded).most_common(1)[0][0];code=f'let mut {name}:[u8;256];'
 if base:code+=f'_=fill(&mut {name},0,256,{base});'
 return code+''.join(f'{name}[{i}]={v};'for i,v in enumerate(padded)if v!=base)
def setup(r):
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
 return code+r['extra']
def cases():
 rows=[]
 def add(name,a,b=None,bits=64,op='And',owned=False,error=None,extra='',count=2,left=0,right=1,length=512,unit=7):
  b=b or node('Integer');target='Integer'if op in ['And','Or','Not']else a['kind']
  x,e=converted(a,target,bits);y,f=converted(b,target,bits)
  failure=error or e or (f if op!='Not'else None)
  truth=False
  if not failure:
   if op=='And':truth=bool(x)and bool(y)
   elif op=='Or':truth=bool(x)or bool(y)
   elif op=='Not':truth=not x
   elif op=='Equal':truth=x==y
   elif op=='NotEqual':truth=x!=y
   elif op=='Less':truth=x<y
   elif op=='LessEqual':truth=x<=y
   elif op=='Greater':truth=x>y
   elif op=='GreaterEqual':truth=x>=y
  rows.append(dict(name=name,a=a,b=b,bits=bits,op=op,owned=owned,error=failure,number=((1<<bits)-1)if truth else 0,extra=extra,count=count,left=left,right=right,length=length,unit=unit))
 samples=[node('Integer',number=1),node('String',b'02'),node('Buffer',bytes([3]))]
 for bits in [32,64]:
  for op in ['And','Or','Equal','NotEqual','Less','LessEqual','Greater','GreaterEqual']:
   for i,a in enumerate(samples):
    for j,b in enumerate(samples):add(f'matrix_{bits}_{op}_{i}_{j}',a,b,bits,op,owned=(i+j)%2==0)
 unary=[node('Integer',number=n)for n in [0,1,1<<32,MAX]]+[node('String',b'0x12'),node('String',b'1234'),node('String',b'xyz'),node('String',b''),node('Buffer',b''),node('Buffer',bytes([0,0,0,0,1])),node('String',b'12!\xff'),node('Buffer',bytes(256))]
 for bits in [32,64]:
  for i,a in enumerate(unary):add(f'unary_{bits}_{i}',a,bits=bits,op='Not',owned=i%2==0)
 for bits in [32,64]:
  for op in ['Equal','NotEqual','Less','LessEqual','Greater','GreaterEqual']:
   for i,(a,b)in enumerate([(MAX,0),(1<<63,MAX),(0xffffffff,1<<32),(1<<32,0)]):add(f'unsigned_{bits}_{op}_{i}',node('Integer',number=a),node('Integer',number=b),bits,op)
  for op in ['And','Or']:
   for i,a in enumerate([node('Integer',number=0),node('Integer',number=1<<32),node('String',b'0x12'),node('String',b'100000000'),node('Buffer',bytes([0,0,0,0,1])),node('Buffer',bytes([0])*255+bytes([1]))]):
    add(f'truth_width_{bits}_{op}_{i}',a,node('Integer',number=1),bits,op,owned=i%2==0)
 for op in ['Equal','NotEqual','Less','LessEqual','Greater','GreaterEqual']:
  for i,(a,b)in enumerate([(b'Z',b'AA'),(b'A',b'AB'),(b'\xff',b'\x80\xff'),(b'',b''),(b'\0',b'')]):add(f'lex_{op}_{i}',node('Buffer',a),node('Buffer',b),op=op,owned=i%2==0)
 for owned in [False,True]:
  for op,n in [('And',0),('Or',1)]:
   for kind in ['Buffer','String']:
    add(f'no_truth_shortcut_{owned}_{op}_{kind}',node('Integer',number=n),node(kind),op=op,owned=owned)
   add(f'no_truth_shortcut_{owned}_{op}_invalid_id',node('Integer',number=n),op=op,owned=owned,right=MAX,error='InvalidState')
   add(f'no_truth_shortcut_{owned}_{op}_invalid_tail',node('Integer',number=n),node('String',b'1!\xff'),op=op,owned=owned)
  for side in [0,1]:
   a=node('String',b'AB'if side else b'1!\xff');b=node('String',b'1!\xff'if side else b'AB')
   add(f'full_encoding_{owned}_{side}',a,b,op='Or',owned=owned)
  add(f'left_encoding_first_{owned}',node('String',b'\xff'),op='And',owned=owned,right=MAX,error='Encoding')
  for n in [85,86]:add(f'format_{owned}_{n}',node('String',b'00'),node('Buffer',bytes(n)),op='Equal',owned=owned)
  for n in [255,256]:add(f'terminator_{owned}_{n}',node('Buffer',b'A'),node('String',b'A'*n),op='Less',owned=owned)
 for count in [0,1,65,1<<63,MAX]:add(f'count_{count}',node('Integer'),count=count,error='InvalidState')
 for side in ['left','right']:
  for index in [2,1<<63,MAX]:add(f'{side}_{index}',node('Integer'),error='InvalidState',**{side:index})
 for side in [0,1]:
  for kind in ['Uninitialized','Package','NameReference','BufferField','Method','Device']:
   add(f'unsupported_{side}_{kind}',node(kind)if side==0 else node('Integer'),node(kind)if side==1 else node('Integer'),op='Or')
  for ref in ['Named','Local','Arg','RefOf','Index','Unresolved']:
   add(f'ref_{side}_{ref}',node('Integer'),error='UnsupportedValue',extra=f'store.space.objects[{side}].value=Value::Reference {{kind:ReferenceKind::{ref},object_id:{1-side}}};')
  for label,extra,error in [('owner',f'store.space.objects[{side}].value=Value::String {{string_storage:StringStorage::Owned {{string_owner:{1-side}}}}};','InvalidState'),('uninitialized',f'store.bytes.blocks[{side}].initialized=false;','InvalidState'),('capacity',f'store.bytes.blocks[{side}].length={MAX};','Capacity')]:
   add(f'owned_{side}_{label}',node('String',b'AB'),node('String',b'AB'),owned=True,error=error,extra=extra)
 add('source_unit',node('String',b'1'),node('String',b'1'),unit=8,error='Bounds')
 add('source_length_max',node('String',b'1'),node('String',b'1'),length=MAX,error='Capacity')
 add('owned_unused_source',node('String',b'1'),node('String',b'1'),owned=True,length=MAX,unit=MAX,extra='store.bytes.blocks[0].bytes[255]=255;store.bytes.blocks[1].bytes[254]=254;')
 add('integer_unused_source',node('Integer'),node('Integer'),length=MAX,unit=MAX)
 add('default_failure',node('Integer'),error='InvalidState')
 assert len(rows)==len({r['name']for r in rows});return rows

def body(r,control):
 code=setup(r);size='FourBytes'if r['bits']==32 else'EightBytes'
 if r['name']=='default_failure':code+='let initial:DefaultResult=DefaultResult {};let value:LogicalResult=initial.value;'
 elif r['op']=='Not':code+=f'let value:LogicalResult=negate(&input,{r["length"]},{r["unit"]},&store,{r["left"]},IntegerSize::{size});'
 else:code+=f'let value:LogicalResult=binary(&input,{r["length"]},{r["unit"]},&store,{r["left"]},{r["right"]},IntegerSize::{size},Logical::{r["op"]});'
 if r['error']:
  error=('UnsupportedValue'if r['error']=='InvalidState'else'InvalidState')if control else r['error'];code+=f'let good:bool=is_failure(value,ConversionFailure::{error});'
 else:
  expected=r['number']^1 if control else r['number'];code+=f'let good:bool=is_integer(value,{expected});'
 return code+'transition good {true -> (0) _ -> (1)}'
if __name__=='__main__':
 (HERE/'cases.json').write_text(json.dumps(cases(),indent=2,sort_keys=True)+'\n');print(len(cases()),'logical pairs')
