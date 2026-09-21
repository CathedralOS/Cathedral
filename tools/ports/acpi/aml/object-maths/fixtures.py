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
IMPORTS='''use aml::object_maths::binary;use aml::object_maths::unary;use aml::object_maths::MathResult;use aml::object_maths::MathFailure;use aml::object_maths::Unary;
use aml::object_conversions::ConversionFailure;
use aml::model::ObjectStore;use aml::model::Value;use aml::model::Span;use aml::model::Path;
use aml::model::StringStorage;use aml::model::BufferStorage;use aml::model::ReferenceKind;
use integer_helpers::integers::IntegerSize;use integer_helpers::integers::Binary;
'''
HELPERS='''data DefaultResult {value:MathResult;}
machine fill(bytes:&mut [u8;256],index:u64,count:u64,value:u8)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_=fill_one(bytes,index,count,value);transition index<count {true -> fill(bytes,index+1,count,value) _ -> (0)}}
machine fill_one(bytes:&mut [u8;256],index:u64,count:u64,value:u8)->u8 {transition index<count && index<256 {true -> put(bytes,index,value) _ -> (0)} state put(bytes:&mut [u8;256],index:u64,value:u8)->u8 {bytes[index]=value;0}}
machine source_bytes(input:&mut [u8;1024],bytes:&[u8;256],offset:u64,index:u64,count:u64)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_=source_one(input,bytes,offset,index,count);transition index<count {true -> source_bytes(input,bytes,offset,index+1,count) _ -> (0)}}
machine source_one(input:&mut [u8;1024],bytes:&[u8;256],offset:u64,index:u64,count:u64)->u8 {transition index<count && index<256 && offset<=256 {true -> position(input,bytes,offset+index,index) _ -> (0)} state position(input:&mut [u8;1024],bytes:&[u8;256],at:u64,index:u64)->u8 {transition at<1024 {true -> put(input,bytes,at,index) _ -> (0)}} state put(input:&mut [u8;1024],bytes:&[u8;256],at:u64,index:u64)->u8 {input[at]=bytes[index];0}}
machine is_conversion(value:MathResult,error:ConversionFailure)->bool {transition value {MathResult::Failure {reason} -> reason_check(reason,error) _ -> (false)} state reason_check(reason:MathFailure,error:ConversionFailure)->bool {transition reason {MathFailure::Conversion {reason} -> (reason==error) _ -> (false)}}}
machine is_integer(value:MathResult,expected:u64)->bool {transition value {MathResult::Integer {number} -> (number==expected) _ -> (false)}}
machine is_division(value:MathResult,quotient:u64,remainder:u64)->bool {transition value {MathResult::Division {quotient as a,remainder as b} -> (a==quotient && b==remainder) _ -> (false)}}
machine is_DivideByZero(value:MathResult)->bool {transition value {MathResult::Failure {reason} -> reason_check(reason) _ -> (false)} state reason_check(reason:MathFailure)->bool {transition reason {MathFailure::DivideByZero -> (true) _ -> (false)}}}
machine is_InvalidBcd(value:MathResult)->bool {transition value {MathResult::Failure {reason} -> reason_check(reason) _ -> (false)} state reason_check(reason:MathFailure)->bool {transition reason {MathFailure::InvalidBcd -> (true) _ -> (false)}}}
machine is_Overflow(value:MathResult)->bool {transition value {MathResult::Failure {reason} -> reason_check(reason) _ -> (false)} state reason_check(reason:MathFailure)->bool {transition reason {MathFailure::Overflow -> (true) _ -> (false)}}}
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
BINARY=['Add','Subtract','Multiply','Divide','Mod','ShiftLeft','ShiftRight','And','Nand','Or','Nor','Xor']
UNARY=['BitwiseNot','FindSetLeft','FindSetRight','FromBcd','ToBcd']
def calculate(op,a,b,bits):
 mask=(1<<bits)-1
 if op in ['Divide','Mod']and b==0:return None,None,'DivideByZero'
 if op=='FromBcd':
  digits=f'{a:X}'
  if any(x in 'ABCDEF'for x in digits):return None,None,'InvalidBcd'
  return int(digits),None,None
 if op=='ToBcd':
  if a>=10**(bits//4):return None,None,'Overflow'
  return int(str(a),16),None,None
 value={'Add':lambda:a+b,'Subtract':lambda:a-b,'Multiply':lambda:a*b,'Divide':lambda:a//b,'Mod':lambda:a%b,'ShiftLeft':lambda:a<<b if b<bits else 0,'ShiftRight':lambda:a>>b if b<bits else 0,'And':lambda:a&b,'Nand':lambda:~(a&b),'Or':lambda:a|b,'Nor':lambda:~(a|b),'Xor':lambda:a^b,'BitwiseNot':lambda:~a,'FindSetLeft':lambda:a.bit_length(),'FindSetRight':lambda:(a&-a).bit_length()}[op]()&mask
 return value,a%b if op=='Divide'else None,None

def cases():
 rows=[]
 def add(name,a,b=None,bits=64,op='Add',owned=False,error=None,extra='',count=2,left=0,right=1,length=512,unit=7,control_remainder=False):
  b=b or node('Integer');x,e=converted(a,'Integer',bits);y,f=converted(b,'Integer',bits);failure=error or e or (f if op in BINARY else None)
  value=remainder=None;math_error=None
  if not failure:value,remainder,math_error=calculate(op,x,y,bits)
  rows.append(dict(name=name,a=a,b=b,bits=bits,op=op,owned=owned,error=failure,math_error=math_error,number=value,remainder=remainder,control_remainder=control_remainder,extra=extra,count=count,left=left,right=right,length=length,unit=unit))
 samples=[node('Integer',number=17),node('String',b'10'),node('Buffer',bytes([3]))]
 for bits in [32,64]:
  for op in BINARY:
   for i,a in enumerate(samples):
    for j,b in enumerate(samples):add(f'matrix_{bits}_{op}_{i}_{j}',a,b,bits,op,owned=(i+j)%2==0,control_remainder=j%2==0)
  for op in BINARY:
   pairs=[(MAX,1),(0,1),(1<<63,MAX)]
   if op in ['Divide','Mod']:pairs.extend([(MAX,0),(MAX,1<<32)])
   if op in ['ShiftLeft','ShiftRight']:pairs.extend([(MAX,bits-1),(MAX,bits),(MAX,bits+1),(MAX,MAX)])
   for i,(a,b)in enumerate(pairs):add(f'edge_{bits}_{op}_{i}',node('Integer',number=a),node('Integer',number=b),bits,op,control_remainder=i%2==0)
  values=[node('Integer',number=n)for n in [0,1,1<<31,1<<32,0x1234,MAX]]+[node('String',b'0x12'),node('String',b'1234'),node('Buffer',bytes([0,0,0,0,1])),node('String',b''),node('Buffer',b'')]
  for op in UNARY:
   for i,a in enumerate(values):add(f'unary_{bits}_{op}_{i}',a,bits=bits,op=op,owned=i%2==0)
  for index in range(bits//4):add(f'invalid_bcd_{bits}_{index}',node('Integer',number=10<<(index*4)),bits=bits,op='FromBcd')
  for number in [10**(bits//4)-1,10**(bits//4),10**(bits//4)+1]:add(f'bcd_capacity_{bits}_{number}',node('Integer',number=number),bits=bits,op='ToBcd')
 for owned in [False,True]:
  add(f'full_string_tail_{owned}',node('String',b'1!\xff'),op='Add',owned=owned)
  add(f'full_right_tail_{owned}',node('Integer'),node('String',b'0!\xff'),op='Divide',owned=owned)
  add(f'left_error_before_zero_{owned}',node('String',b'\xff'),op='Divide',owned=owned)
  add(f'left_error_before_bad_id_{owned}',node('String',b'\xff'),op='Add',owned=owned,right=MAX,error='Encoding')
 for count in [0,1,65,1<<63,MAX]:add(f'count_{count}',node('Integer'),count=count,error='InvalidState')
 for side in ['left','right']:
  for index in [2,1<<63,MAX]:add(f'{side}_{index}',node('Integer'),error='InvalidState',**{side:index})
 for side in [0,1]:
  for kind in ['Uninitialized','Package','NameReference','BufferField','Method','Device']:
   add(f'unsupported_{side}_{kind}',node(kind)if side==0 else node('Integer'),node(kind)if side==1 else node('Integer'))
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
 if r['name']=='default_failure':code+='let initial:DefaultResult=DefaultResult {};let value:MathResult=initial.value;'
 elif r['op']in UNARY:code+=f'let value:MathResult=unary(&input,{r["length"]},{r["unit"]},&store,{r["left"]},IntegerSize::{size},Unary::{r["op"]});'
 else:code+=f'let value:MathResult=binary(&input,{r["length"]},{r["unit"]},&store,{r["left"]},{r["right"]},IntegerSize::{size},Binary::{r["op"]});'
 if r['error']:
  error=('UnsupportedValue'if r['error']=='InvalidState'else'InvalidState')if control else r['error'];code+=f'let good:bool=is_conversion(value,ConversionFailure::{error});'
 elif r['math_error']:
  error=('InvalidBcd'if r['math_error']=='DivideByZero'else'DivideByZero')if control else r['math_error'];code+=f'let good:bool=is_{error}(value);'
 elif r['op']=='Divide':
  q=r['number'];m=r['remainder']
  if control:
   if r['control_remainder']:m^=1
   else:q^=1
  code+=f'let good:bool=is_division(value,{q},{m});'
 else:
  expected=r['number']^1 if control else r['number'];code+=f'let good:bool=is_integer(value,{expected});'
 return code+'transition good {true -> (0) _ -> (1)}'
if __name__=='__main__':
 (HERE/'cases.json').write_text(json.dumps(cases(),indent=2,sort_keys=True)+'\n');print(len(cases()),'arithmetic pairs')
