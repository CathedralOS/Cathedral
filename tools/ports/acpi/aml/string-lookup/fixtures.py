import json
from pathlib import Path
from collections import Counter
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4];MAX=(1<<64)-1

def cases():
 rows=[]
 def add(name,text=b'ITEM',owned=False,scope='DEV0.SUB0',object=3,path='DEV0.SUB0.ITEM',failure=None,error=None,offset=0,extra='',count=6,entries=9,source=0,length=512,unit=7,scope_extra='',expected_tail=None):
  rows.append(dict(name=name,text=list(text),owned=owned,scope=scope,object=object,path=path,failure=failure,error=error,offset=offset,extra=extra,count=count,entries=entries,source=source,length=length,unit=unit,scope_extra=scope_extra,expected_tail=expected_tail or {}))
 for owned in [False,True]:
  examples=[('nearest',b'ITEM','DEV0.SUB0',3,'DEV0.SUB0.ITEM'),('parent',b'^ITEM','DEV0.SUB0',2,'DEV0.ITEM'),('ancestor',b'ROOT','DEV0.SUB0',1,'ROOT'),('absolute',b'\\ROOT','DEV0.SUB0',1,'ROOT'),('relative_multi',b'DEV0.ITEM','',2,'DEV0.ITEM'),('alias',b'ALS0','DEV0.SUB0',2,'DEV0.ALS0'),('lowercase',b'item','DEV0.SUB0',3,'DEV0.SUB0.ITEM'),('field_identity',b'FLD0','DEV0.SUB0',4,'DEV0.SUB0.FLD0'),('method_identity',b'MTH0','DEV0.SUB0',5,'DEV0.SUB0.MTH0'),('absolute_missing_scope',b'\\ROOT','NOPE',1,'ROOT')]
  for name,text,scope,object,path in examples:add(f'{name}_{owned}',text,owned,scope,object,path)
  for name,text,scope,error in [('missing',b'NOPE','DEV0.SUB0','MissingObject'),('missing_level',b'ITEM','NOPE','MissingLevel'),('no_multilevel_upward',b'DEV0.ITEM','DEV0.SUB0','MissingObject'),('above_root',b'^ROOT','','AboveRoot'),('root_no_object',b'\\','DEV0.SUB0','MissingObject'),('parent_level_only',b'^','DEV0.SUB0','MissingObject')]:add(f'{name}_{owned}',text,owned,scope,failure='Search',error=error)
  for name,text,error,offset in [('empty',b'','InvalidName',0),('digit',b'1BAD','InvalidName',0),('long_segment',b'ABCDE','InvalidName',4),('trailing_dot',b'ITEM.','InvalidName',5),('double_dot',b'DEV0..ITEM','InvalidName',5),('interior_parent',b'I^EM','InvalidName',1),('too_many_parents',b'^'*17,'Capacity',16)]:add(f'name_{name}_{owned}',text,owned,failure='Name',error=error,offset=offset)
  for byte in [0,128,255]:add(f'encoding_{owned}_{byte}',b'ITEM'+bytes([byte]),owned,failure='Storage',error='Encoding')
  add(f'bad_binding_{owned}',owned=owned,extra=f'store.space.entries[5].object={MAX};',object=2,path='DEV0.ITEM')
  add(f'bad_all_bindings_{owned}',owned=owned,extra=f'store.space.entries[5].object={MAX};store.space.entries[4].object={MAX};',failure='Search',error='MissingObject')
  add(f'self_identity_{owned}',owned=owned,extra='store.space.entries[5].object=0;',object=0)
  add(f'uninitialized_target_{owned}',owned=owned,extra='store.space.objects[3].value=Value::Uninitialized;')
 for count in [0,65,1<<63,MAX]:add(f'object_count_{count}',count=count,failure='InvalidState')
 for entries in [33,1<<63,MAX]:add(f'entry_count_{entries}',entries=entries,failure='InvalidState')
 for source in [6,1<<63,MAX]:add(f'source_{source}',source=source,failure='InvalidState')
 for kind in ['Named','Local','Arg','RefOf','Index','Unresolved']:add('opaque_'+kind,extra=f'store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:0}};',failure='UnsupportedValue')
 for name,value in [('integer','Integer {number:0}'),('buffer','Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:0}}'),('uninitialized','Uninitialized'),('name_ref','NameReference {name:Path {},scope:Path {}}'),('field','BufferField {backing_object:0,bit_offset:0,bit_length:8}')]:add('unsupported_'+name,extra=f'store.space.objects[0].value=Value::{value};',failure='UnsupportedValue')
 add('owner_mismatch',owned=True,extra='store.space.objects[0].value=Value::String {string_storage:StringStorage::Owned {string_owner:1}};',failure='Storage',error='InvalidState')
 add('missing_owned',owned=True,extra='store.bytes.blocks[0].initialized=false;',failure='Storage',error='InvalidState')
 add('owned_capacity',owned=True,extra=f'store.bytes.blocks[0].length={MAX};',failure='Storage',error='Capacity')
 add('unused_owned_source',owned=True,length=MAX,unit=MAX,extra='store.bytes.blocks[0].bytes[255]=255;')
 add('source_unit',unit=8,failure='Storage',error='Bounds')
 add('source_length',length=MAX,failure='Storage',error='Capacity')
 add('default_failure',failure='InvalidState')
 add('scope_nonabsolute',scope_extra='scope.absolute=false;',failure='Search',error='NotAbsolute')
 for n in [17,MAX]:
  add(f'scope_count_{n}',scope_extra=f'scope.count={n};',failure='Search',error='InvalidName')
  add(f'scope_parents_{n}',scope_extra=f'scope.parents={n};',failure='Search',error='InvalidName')
 add('name_error_before_bad_scope',text=b'1BAD',scope_extra=f'scope.count={MAX};',failure='Name',error='InvalidName',offset=0)
 add('dirty_scope_relative',scope_extra='scope.segments[15]=0xdeadbeef;',expected_tail={15:0xdeadbeef})
 add('dirty_scope_absolute',text=b'\\ROOT',object=1,path='ROOT',scope_extra='scope.segments[15]=0xdeadbeef;')
 add('dirty_scope_ancestor',text=b'ROOT',object=1,path='ROOT',scope_extra='scope.segments[15]=0xdeadbeef;',expected_tail={1:int.from_bytes(b'SUB0','little'),15:0xdeadbeef})
 for r in rows:
  if r['name']in ['ancestor_False','ancestor_True']:r['expected_tail'][1]=int.from_bytes(b'SUB0','little')

 for r in rows:r['expected_tail']={str(k):v for k,v in r['expected_tail'].items()}
 assert len(rows)==len({r['name']for r in rows});return rows
IMPORTS="""use aml::string_lookup::lookup;use aml::string_lookup::StringLookup;use aml::string_lookup::LookupFailure;
use aml::model::ObjectStore;use aml::model::Value;use aml::model::Span;use aml::model::Path;
use aml::model::StringStorage;use aml::model::BufferStorage;use aml::model::ReferenceKind;
use aml::model::Outcome;use aml::byte_storage::ByteOutcome;
"""
HELPERS='''data DefaultResult {value:StringLookup;}
machine fill(bytes:&mut [u8;256],index:u64,count:u64,value:u8)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_=fill_one(bytes,index,count,value);transition index<count {true -> fill(bytes,index+1,count,value) _ -> (0)}}
machine fill_one(bytes:&mut [u8;256],index:u64,count:u64,value:u8)->u8 {transition index<count && index<256 {true -> put(bytes,index,value) _ -> (0)} state put(bytes:&mut [u8;256],index:u64,value:u8)->u8 {bytes[index]=value;0}}
machine source_bytes(input:&mut [u8;1024],bytes:&[u8;256],offset:u64,index:u64,count:u64)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_=source_one(input,bytes,offset,index,count);transition index<count {true -> source_bytes(input,bytes,offset,index+1,count) _ -> (0)}}
machine source_one(input:&mut [u8;1024],bytes:&[u8;256],offset:u64,index:u64,count:u64)->u8 {transition index<count && index<256 && offset<=256 {true -> position(input,bytes,offset+index,index) _ -> (0)} state position(input:&mut [u8;1024],bytes:&[u8;256],at:u64,index:u64)->u8 {transition at<1024 {true -> put(input,bytes,at,index) _ -> (0)}} state put(input:&mut [u8;1024],bytes:&[u8;256],at:u64,index:u64)->u8 {input[at]=bytes[index];0}}
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
def path_value(path):
 parts=path.split('.')if path else[]
 code='Path {absolute:true,count:'+str(len(parts))+'}'
 return code,parts
def path_init(name,path):
 value,parts=path_value(path);code=f'let mut {name}:Path={value};'
 for i,part in enumerate(parts):code+=f'{name}.segments[{i}]={int.from_bytes(part.encode().ljust(4,b"_"),"little")};'
 return code
def body(r,control):
 code=f'let mut input:[u8;1024];let mut store:ObjectStore=ObjectStore {{}};store.space.object_count={r["count"]};store.space.count={r["entries"]};'
 entries=[('',True,None),('DEV0',True,None),('DEV0.SUB0',True,None),('ROOT',False,1),('DEV0.ITEM',False,2),('DEV0.SUB0.ITEM',False,3),('DEV0.ALS0',False,2),('DEV0.SUB0.FLD0',False,4),('DEV0.SUB0.MTH0',False,5)]
 for i,(path,level,obj)in enumerate(entries):
  code+=path_init(f'path{i}',path)+f'store.space.entries[{i}].path=path{i};store.space.entries[{i}].has_level={str(level).lower()};'
  if obj is not None:code+=f'store.space.entries[{i}].has_object=true;store.space.entries[{i}].object={obj};'
 code+='store.space.entries[6].alias=true;store.space.objects[1].value=Value::Integer {number:11};store.space.objects[2].value=Value::Integer {number:22};store.space.objects[3].value=Value::Integer {number:33};store.space.objects[4].value=Value::BufferField {backing_object:63,bit_offset:0,bit_length:8};store.space.objects[5].value=Value::Method {flags:0,body:Span {unit:99,start:0,end:999}};'
 code+=initialized('data0',r['text'])
 if r['owned']:code+=f'store.space.objects[0].value=Value::String {{string_storage:StringStorage::Owned {{string_owner:0}}}};store.bytes.blocks[0].initialized=true;store.bytes.blocks[0].length={len(r["text"])};store.bytes.blocks[0].bytes=data0;'
 else:code+=f'_=source_bytes(&mut input,&data0,0,0,{len(r["text"])});store.space.objects[0].value=Value::String {{string_storage:StringStorage::Source {{string_source:Span {{unit:7,start:0,end:{len(r["text"])}}}}}}};'
 code+=r['extra']+path_init('scope',r['scope'])+r['scope_extra']
 if r['failure']:
  reason=r['failure'];error=r['error']
  if control:
   if reason in ['Storage','Search','Name']:error='InvalidState'if error!='InvalidState'else'Capacity'
   else:reason='UnsupportedValue'if reason=='InvalidState'else'InvalidState'
  inner='LookupFailure::'+reason
  if reason in ['Storage','Search','Name']:
   typ='ByteOutcome'if reason=='Storage'else'Outcome';inner+=' {reason:'+typ+'::'+error+(f',offset:{r["offset"]}'if reason=='Name'else'')+'}'
  code+='let expected:StringLookup=StringLookup::Failure {reason:'+inner+'};'
 else:
  code+=path_init('expected_path',r['path'])
  for index,value in r['expected_tail'].items():code+=f'expected_path.segments[{index}]={value};'
  dirty_control=control and r['name']in ['dirty_scope_relative','dirty_scope_absolute','dirty_scope_ancestor']
  if dirty_control:code+=f'expected_path.segments[15]={r["expected_tail"].get("15",0)^1};'
  target=r['object']+(1 if control and not dirty_control else 0)
  code+=f'let expected:StringLookup=StringLookup::Found {{object:{target},path:expected_path}};'
 default=str(r['name']=='default_failure').lower()
 code+=f'let result:i32=check_lookup(&input,{r["length"]},{r["unit"]},&store,{r["source"]},scope,{default},expected);result'

 return code
EXTRA="""machine path_same(a:Path,b:Path,index:u64,count:u64,same:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let next:bool=path_at(a,b,index,same);transition index<count {true -> path_same(a,b,index+1,count,next) _ -> (same && a.absolute==b.absolute && a.parents==b.parents && a.count==b.count)}}
machine path_at(a:Path,b:Path,index:u64,same:bool)->bool {transition index<16 {true -> (same && a.segments[index]==b.segments[index]) _ -> (same)}}
"""
HELPERS+=EXTRA

HELPERS+="""
machine check_lookup(input:&[u8;1024],length:u64,unit:u64,store:&ObjectStore,source:u64,scope:Path,use_default:bool,expected:StringLookup)->i32 {
 transition use_default {true -> default(expected) _ -> actual(input,length,unit,store,source,scope,expected)}
 state actual(input:&[u8;1024],length:u64,unit:u64,store:&ObjectStore,source:u64,scope:Path,expected:StringLookup)->i32 {
  let observed:StringLookup=lookup(input,length,unit,store,source,scope);let good:bool=lookup_same(observed,expected);
  transition good {true -> (0) _ -> (1)}
 }
 state default(expected:StringLookup)->i32 {let initial:DefaultResult=DefaultResult {};let observed:StringLookup=initial.value;let good:bool=lookup_same(observed,expected);transition good {true -> (0) _ -> (1)}}
}
machine lookup_same(actual:StringLookup,expected:StringLookup)->bool {
 transition actual {
  StringLookup::Found {object,path} -> found(object,path,expected)
  StringLookup::Failure {reason} -> failure(reason,expected)
 }
 state found(actual_object:u64,actual_path:Path,expected:StringLookup)->bool {
  transition expected {StringLookup::Found {object,path} -> paths(actual_object,actual_path,object,path) _ -> (false)}
 }
 state paths(actual_object:u64,actual_path:Path,expected_object:u64,expected_path:Path)->bool {
  let same:bool=path_same(actual_path,expected_path,0,16,true);same && actual_object==expected_object
 }
 state failure(actual:LookupFailure,expected:StringLookup)->bool {
  transition expected {StringLookup::Failure {reason} -> compare(actual,reason) _ -> (false)}
 }
 state compare(actual:LookupFailure,expected:LookupFailure)->bool {let same:bool=failure_same(actual,expected);same}
}
machine failure_same(actual:LookupFailure,expected:LookupFailure)->bool {
 transition actual {
  LookupFailure::InvalidState -> invalid(expected)
  LookupFailure::UnsupportedValue -> unsupported(expected)
  LookupFailure::Storage {reason} -> storage(reason,expected)
  LookupFailure::Name {reason,offset} -> name(reason,offset,expected)
  LookupFailure::Search {reason} -> search(reason,expected)
 }
 state invalid(expected:LookupFailure)->bool {transition expected {LookupFailure::InvalidState -> (true) _ -> (false)}}
 state unsupported(expected:LookupFailure)->bool {transition expected {LookupFailure::UnsupportedValue -> (true) _ -> (false)}}
 state storage(actual_reason:ByteOutcome,expected:LookupFailure)->bool {transition expected {LookupFailure::Storage {reason} -> (actual_reason==reason) _ -> (false)}}
 state name(actual_reason:Outcome,actual_offset:u64,expected:LookupFailure)->bool {transition expected {LookupFailure::Name {reason,offset} -> (actual_reason==reason && actual_offset==offset) _ -> (false)}}
 state search(actual_reason:Outcome,expected:LookupFailure)->bool {transition expected {LookupFailure::Search {reason} -> (actual_reason==reason) _ -> (false)}}
}
"""
