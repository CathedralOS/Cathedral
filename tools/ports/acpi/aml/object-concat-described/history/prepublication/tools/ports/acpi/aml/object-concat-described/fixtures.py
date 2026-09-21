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
  result_kind='String'if a['kind']=='String'else'Buffer';expected=None
  if not failure:
   expected=list(x.to_bytes(bits//8,'little')+y.to_bytes(bits//8,'little'))if a['kind']=='Integer'else x+y
   if len(expected)>256:failure='Capacity';expected=None
  rows.append(dict(name=name,a=a,b=b,bits=bits,owned=owned,error=failure,result_kind=result_kind,expected=expected,extra=extra,count=count,left=left,right=right,length=length,unit=unit))

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
 for owned in [False,True]:
  for kind in ['Buffer','String']:
   for left_length,right_length in [(0,256),(256,0),(128,128),(128,129),(255,1),(255,2),(256,256)]:
    add(f'combined_{owned}_{kind}_{left_length}_{right_length}',node(kind,b'A'*left_length),node(kind,b'B'*right_length),owned=owned)
  for left_length in [1,2,3]:add(f'string_buffer_format_fit_{owned}_{left_length}',node('String',b'A'*left_length),node('Buffer',bytes([0])*85),owned=owned)
  add(f'encoding_before_combined_capacity_{owned}',node('String',b'A'*256),node('String',b'B'+bytes([255])),owned=owned)
 assert len(rows)==len({r['name']for r in rows});return rows
IMPORTS='''use aml::object_concat_described::concatenate;use aml::object_concat::Concatenated;
use aml::object_conversions::ConversionFailure;
use aml::model::ObjectStore;use aml::model::Value;use aml::model::Span;use aml::model::Path;
use aml::model::StringStorage;use aml::model::BufferStorage;use aml::model::ReferenceKind;
use integer_helpers::integers::IntegerSize;
'''
HELPERS='''data DefaultResult {value:Concatenated;}
machine fill(bytes:&mut [u8;256],index:u64,count:u64,value:u8)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_=fill_one(bytes,index,count,value);transition index<count {true -> fill(bytes,index+1,count,value) _ -> (0)}}
machine fill_one(bytes:&mut [u8;256],index:u64,count:u64,value:u8)->u8 {transition index<count && index<256 {true -> put(bytes,index,value) _ -> (0)} state put(bytes:&mut [u8;256],index:u64,value:u8)->u8 {bytes[index]=value;0}}
machine source_bytes(input:&mut [u8;1024],bytes:&[u8;256],offset:u64,index:u64,count:u64)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_=source_one(input,bytes,offset,index,count);transition index<count {true -> source_bytes(input,bytes,offset,index+1,count) _ -> (0)}}
machine source_one(input:&mut [u8;1024],bytes:&[u8;256],offset:u64,index:u64,count:u64)->u8 {transition index<count && index<256 && offset<=256 {true -> position(input,bytes,offset+index,index) _ -> (0)} state position(input:&mut [u8;1024],bytes:&[u8;256],at:u64,index:u64)->u8 {transition at<1024 {true -> put(input,bytes,at,index) _ -> (0)}} state put(input:&mut [u8;1024],bytes:&[u8;256],at:u64,index:u64)->u8 {input[at]=bytes[index];0}}
machine is_failure(value:Concatenated,error:ConversionFailure)->bool {transition value {Concatenated::Failure {reason} -> (reason==error) _ -> (false)}}
machine expected_bytes(value:Concatenated,string:bool,length:u64,expected:&[u8;256])->bool {
 transition value {Concatenated::Buffer {length as actual_length,bytes} -> bytes(&bytes,expected,actual_length,length,!string) Concatenated::String {length as actual_length,bytes} -> bytes(&bytes,expected,actual_length,length,string) _ -> (false)}
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
  else:
   m=MAX if n.get('poison')else 0;p8=255 if n.get('poison')else 0
   value={'Uninitialized':'Uninitialized','Package':f'Package {{first:{m},count:{m}}}','Method':f'Method {{flags:{p8},body:Span {{unit:{m},start:{m}}}}}','NameReference':'NameReference {name:Path {},scope:Path {}}','BufferField':f'BufferField {{backing_object:{m},bit_offset:{m},bit_length:{m}}}','Device':'Device','Event':'Event','Mutex':f'Mutex {{sync_level:{p8}}}','OperationRegion':f'OperationRegion {{space:{p8},base:{m},length:{m},scope:Path {{count:{m}}}}}','PowerResource':f'PowerResource {{system_level:{p8},order:{p8}}}','Processor':f'Processor {{id:{p8},address:{p8},length:{p8}}}','ThermalZone':'ThermalZone'}[kind]
  code+=f'store.space.objects[{i}].value=Value::{value};'
 code+=r['extra'];size='FourBytes'if r['bits']==32 else'EightBytes'
 code+='let initial:DefaultResult=DefaultResult {};let value:Concatenated=initial.value;'if r['name']=='default_failure'else f'let value:Concatenated=concatenate(&input,{r["length"]},{r["unit"]},&store,{r["left"]},{r["right"]},IntegerSize::{size});'
 if r['error']:
  error=('UnsupportedValue'if r['error']=='InvalidState'else'InvalidState')if control else r['error'];code+=f'let good:bool=is_failure(value,ConversionFailure::{error});'
 else:
  expected=r['expected']+[0]*(256-len(r['expected']))
  if control:expected[-1]^=1
  code+=initialized('expected',expected)+f'let good:bool=expected_bytes(value,{str(r["result_kind"]=="String").lower()},{len(r["expected"])},&expected);'

 return code+'transition good {true -> (0) _ -> (1)}'

original_body=body
old=original_body
HELPERS+='machine check_concatenation(input:&[u8;1024],length:u64,unit:u64,store:&ObjectStore,left:u64,right:u64,size:IntegerSize,want_error:bool,error:ConversionFailure,want_string:bool,want_length:u64,expected:&[u8;256])->i32 {\n let result:Concatenated=concatenate(input,length,unit,store,left,right,size);\n transition result {\n  Concatenated::Failure {reason} -> verdict(want_error && reason==error)\n  Concatenated::String {length as actual,bytes} -> compare(&bytes,actual,expected,want_length,!want_error && want_string)\n  Concatenated::Buffer {length as actual,bytes} -> compare(&bytes,actual,expected,want_length,!want_error && !want_string)\n }\n state compare(bytes:&[u8;256],actual:u64,expected:&[u8;256],length:u64,kind:bool)->i32 {let same:bool=equal_bytes(bytes,expected,0,256,true);transition kind && actual==length && same {true -> (0) _ -> (1)}}\n state verdict(good:bool)->i32 {transition good {true -> (0) _ -> (1)}}\n}\n'
def body(r,control):
 if r['name']=='default_failure':return old(r,control)
 s=old(r,control);s=s[:s.index('let value:Concatenated=concatenate(')]
 error=r['error']or'InvalidState'
 if r['error']and control:error='UnsupportedValue'if error=='InvalidState'else'InvalidState'
 expected=(r['expected']or[])+[0]*(256-len(r['expected']or[]))
 if not r['error']and control:expected[-1]^=1
 s+=initialized('expected',expected)
 s+=f'let answer:i32=check_concatenation(&input,{r["length"]},{r["unit"]},&store,{r["left"]},{r["right"]},IntegerSize::{"FourBytes"if r["bits"]==32 else"EightBytes"},{str(bool(r["error"])).lower()},ConversionFailure::{error},{str(r["result_kind"]=="String").lower()},{len(r["expected"]or[])},&expected);answer'
 return s

basic_cases=cases
LABELS={'Uninitialized':'[Uninitialized Object]','Package':'[Package]','BufferField':'[Buffer Field]','Device':'[Device]','Event':'[Event]','Method':'[Control Method]','Mutex':'[Mutex]','OperationRegion':'[Operation Region]','PowerResource':'[Power Resource]','Processor':'[Processor]','ThermalZone':'[Thermal Zone]'}
def described_node(kind):
 value=node(kind);value['poison']=True;return value
def cases():
 rows=[r for r in basic_cases()if r['a']['kind']not in LABELS and r['b']['kind']not in LABELS]
 def add(name,a,b,bits=64,owned=False,error=None,extra='',count=2,left=0,right=1,length=512,unit=7,expected=None,result_kind=None):
  target='String'if a['kind']in LABELS else a['kind']
  x,e=(list(LABELS[a['kind']].encode()),None)if a['kind']in LABELS else converted(a,target,bits)
  if b['kind']in LABELS:
   y,f=(None,'UnsupportedValue')if target=='Integer'else(list(LABELS[b['kind']].encode())+([0]if target=='Buffer'else[]),None)
  else:y,f=converted(b,target,bits)
  failure=error or e or f;kind=result_kind or('String'if target=='String'else'Buffer')
  if expected is None and not failure:expected=list(x.to_bytes(bits//8,'little')+y.to_bytes(bits//8,'little'))if target=='Integer'else x+y
  if expected is not None and len(expected)>256:failure=failure or 'Capacity';expected=None
  rows.append(dict(name=name,a=a,b=b,bits=bits,owned=owned,error=failure,result_kind=kind,expected=expected,extra=extra,count=count,left=left,right=right,length=length,unit=unit))
 labels=list(LABELS)
 for i,kind in enumerate(labels):
  n=described_node(kind);following=described_node(labels[(i+1)%len(labels)])
  for name,a,b in [('string_left',node('String',b'S'),n),('buffer_left',node('Buffer',b'B'),n),('integer_excluded',node('Integer',number=12),n),('string_right',n,node('String',b'S')),('buffer_right',n,node('Buffer',b'A\0')),('integer_right',n,node('Integer',number=12)),('labels',n,following)]:add(f'desc_{kind}_{name}',a,b,owned=i%2==0)
  add(f'desc_{kind}_integer32',n,node('Integer',number=MAX),bits=32)
 for kind in ['Uninitialized','Device']:
  n=described_node(kind);label=len(LABELS[kind])
  for owned in [False,True]:
   for overflow in [0,1]:
    add(f'fit_string_left_{kind}_{owned}_{overflow}',node('String',b'A'*(256-label+overflow)),n,owned=owned)
    add(f'fit_string_right_{kind}_{owned}_{overflow}',n,node('String',b'B'*(256-label+overflow)),owned=owned)
    add(f'fit_buffer_left_{kind}_{owned}_{overflow}',node('Buffer',b'A'*(255-label+overflow)),n,owned=owned)
 for owned in [False,True]:
  n=described_node('Device')
  for length in [0,1,82,83,84,85,86,256]:add(f'formatted_buffer_{owned}_{length}',n,node('Buffer',b'\0'*length),owned=owned)
  add(f'left_encoding_before_right_id_{owned}',node('String',b'\xff'),n,owned=owned,right=MAX,error='Encoding')
  add(f'right_encoding_before_total_fit_{owned}',n,node('String',b'A'*255+b'\xff'),owned=owned,error='Encoding')
  add(f'right_nul_before_total_fit_{owned}',n,node('String',b'A'*255+b'\0'),owned=owned,error='Encoding')
  add(f'right_id_before_total_fit_{owned}',node('String',b'A'*256),n,owned=owned,right=MAX,error='InvalidState')
  add(f'left_bad_unit_{owned}',node('String',b'A'),n,owned=owned,unit=8,error=None if owned else 'Bounds')
  add(f'right_bad_unit_{owned}',n,node('Buffer',b'A'*86),owned=owned,unit=8,error='Capacity'if owned else'Bounds')
  add(f'ignore_source_labels_{owned}',n,described_node('Uninitialized'),owned=owned,length=MAX,unit=MAX)
  add(f'ignore_source_integer_{owned}',n,node('Integer',number=MAX),owned=owned,length=MAX,unit=MAX)
  add(f'dirty_tails_{owned}',node('String',b'A'),n,owned=owned,extra='store.bytes.blocks[0].bytes[255]=255;store.bytes.blocks[1].bytes[255]=255;input[255]=255;')
 for ref in ['Named','Local','Arg','RefOf','Index','Unresolved']:
  for side in [0,1]:add(f'desc_reference_{side}_{ref}',described_node('Device'),described_node('Package'),error='UnsupportedValue',extra=f'store.space.objects[{side}].value=Value::Reference {{kind:ReferenceKind::{ref},object_id:{1-side}}};')
 for side in [0,1]:
  add(f'desc_name_reference_{side}',described_node('Device'),described_node('Package'),error='UnsupportedValue',extra=f'store.space.objects[{side}].value=Value::NameReference {{name:Path {{}},scope:Path {{}}}};')
  for label,extra,error in [('owner',f'store.space.objects[{side}].value=Value::String {{string_storage:StringStorage::Owned {{string_owner:{1-side}}}}};','InvalidState'),('uninitialized',f'store.bytes.blocks[{side}].initialized=false;','InvalidState'),('capacity',f'store.bytes.blocks[{side}].length={MAX};','Capacity')]:
   a=node('String',b'AB')if side==0 else described_node('Device');b=node('String',b'AB')if side==1 else described_node('Device');add(f'desc_owned_{side}_{label}',a,b,owned=True,error=error,extra=extra)
 add('desc_last_slot',described_node('Device'),described_node('Package'),count=64,left=62,right=63,extra='store.space.objects[62]=store.space.objects[0];store.space.objects[63]=store.space.objects[1];')
 add('desc_alias_same_id',described_node('Device'),described_node('Package'),right=0,expected=list(b'[Device][Device]'))
 assert len(rows)==len({r['name']for r in rows});return rows

def render(r,control=False,machine='test_result()'):return 'machine '+machine+'->i32{'+body(r,control)+'}\n'
def main():
 import argparse
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();text=json.dumps(cases(),indent=2,sort_keys=True)+'\n';path=HERE/'cases.json'
 if a.check:assert path.read_text()==text
 else:path.write_text(text)
 print(len(cases()),'description-aware concatenation cases')
if __name__=='__main__':main()
