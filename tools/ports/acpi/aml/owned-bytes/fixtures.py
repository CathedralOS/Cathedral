#!/usr/bin/env python3
"""Original owned-byte scenarios; compare all namespace metadata and 16KiB arena."""
from pathlib import Path
import json
HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[4]; MAX=2**64-1
IMPORTS='''use aml::model::Outcome;
use aml::model::Span;
use aml::model::Path;
use aml::model::Value;
use aml::model::StringStorage;
use aml::model::BufferStorage;
use aml::model::ByteBlock;
use aml::model::ByteArena;
use aml::model::ObjectStore;
use aml::model::Object;
use aml::model::Entry;
use aml::model::Namespace;
use aml::model::NamespaceResult;
use aml::model::ReferenceKind;
use aml::byte_storage::ByteOutcome;
use aml::byte_storage::ByteKind;
use aml::byte_storage::ByteRead;
use aml::byte_storage::ByteResult;
use aml::byte_storage::read_bytes;
use aml::byte_storage::byte_length;
use aml::byte_storage::materialize;
use aml::byte_storage::clone_bytes_into;
use aml::byte_storage::create_buffer_field;
use aml::byte_storage::make_byte_index;
use aml::byte_storage::read_field_integer;
use aml::byte_storage::write_field_integer;
use aml::object_references::copy_value;
use helpers::integers::IntegerSize;
'''
HELPERS='''machine fx_bytes_equal(a:&[u8;256],b:&[u8;256],i:u64,n:u64,prior:bool)
terminates by(i,n)->Nat::BoundedDistance;
->bool {let good:bool=fx_byte_equal(a,b,i,n);transition i<n {true -> fx_bytes_equal(a,b,i+1,n,prior && good) _ -> (prior)}}
machine fx_byte_equal(a:&[u8;256],b:&[u8;256],i:u64,n:u64)->bool {transition i<n && i<256 {true -> (a[i]==b[i]) _ -> (true)}}
machine fx_span(a:Span,b:Span)->bool {a.unit==b.unit && a.start==b.start && a.end==b.end}
machine fx_path(a:Path,b:Path)->bool {let good:bool=fx_segments(&a.segments,&b.segments,0,16,true);a.absolute==b.absolute && a.parents==b.parents && a.count==b.count && good}
machine fx_segments(a:&[u32;16],b:&[u32;16],i:u64,n:u64,prior:bool)
terminates by(i,n)->Nat::BoundedDistance;
->bool {let good:bool=fx_segment(a,b,i,n);transition i<n {true -> fx_segments(a,b,i+1,n,prior && good) _ -> (prior)}}
machine fx_segment(a:&[u32;16],b:&[u32;16],i:u64,n:u64)->bool {transition i<n && i<16 {true -> (a[i]==b[i]) _ -> (true)}}
machine fx_string(a:StringStorage,b:StringStorage)->bool {
 transition a {StringStorage::Source {string_source} -> source(string_source,b) StringStorage::Owned {string_owner} -> owned(string_owner,b)}
 state source(a:Span,b:StringStorage)->bool {transition b {StringStorage::Source {string_source} -> span(a,string_source) _ -> (false)}}
 state span(a:Span,b:Span)->bool {let good:bool=fx_span(a,b);good}
 state owned(a:u64,b:StringStorage)->bool {transition b {StringStorage::Owned {string_owner} -> (a==string_owner) _ -> (false)}}
}
machine fx_buffer(a:BufferStorage,b:BufferStorage)->bool {
 transition a {BufferStorage::Source {declared_size,buffer_initializer} -> source(declared_size,buffer_initializer,b) BufferStorage::Owned {buffer_owner} -> owned(buffer_owner,b)}
 state source(size:u64,a:Span,b:BufferStorage)->bool {transition b {BufferStorage::Source {declared_size,buffer_initializer} -> span(size,a,declared_size,buffer_initializer) _ -> (false)}}
 state span(size:u64,a:Span,other:u64,b:Span)->bool {let good:bool=fx_span(a,b);size==other && good}
 state owned(a:u64,b:BufferStorage)->bool {transition b {BufferStorage::Owned {buffer_owner} -> (a==buffer_owner) _ -> (false)}}
}
'''
VARIANTS={'Uninitialized':{},'Integer':{'number':'u64'},'String':{'string_storage':'StringStorage'},'Buffer':{'buffer_storage':'BufferStorage'},'BufferField':{'backing_object':'u64','bit_offset':'u64','bit_length':'u64'},'Package':{'first':'u64','count':'u64'},'NameReference':{'name':'Path','scope':'Path'},'Reference':{'kind':'ReferenceKind','object_id':'u64'},'Method':{'flags':'u8','body':'Span'},'Device':{},'Processor':{'id':'u8','address':'u32','length':'u8'},'PowerResource':{'system_level':'u8','order':'u16'},'ThermalZone':{},'Mutex':{'sync_level':'u8'},'Event':{},'OperationRegion':{'space':'u8','base':'u64','length':'u64','scope':'Path'}}
HELPERS+='machine fx_value(a:Value,b:Value)->bool {transition a {\n'
for case,fields in VARIANTS.items():
 pat='Value::'+case+(' {'+','.join(fields)+'}' if fields else '')
 HELPERS+=pat+' -> c_'+case.lower()+'('+','.join(list(fields)+['b'])+')\n'
HELPERS+='}\n'
for case,fields in VARIANTS.items():
 params=','.join([f'a_{k}:{v}'for k,v in fields.items()]+['b:Value']);pat='Value::'+case+(' {'+','.join(fields)+'}'if fields else'')
 exprs=[];locals=''
 for k,t in fields.items():
  helper={'Span':'fx_span','Path':'fx_path','StringStorage':'fx_string','BufferStorage':'fx_buffer'}.get(t)
  if helper:locals+=f'let good_{k}:bool={helper}(a_{k},{k});';exprs.append('good_'+k)
  else:exprs.append(f'a_{k}=={k}')
 name='c_'+case.lower();args=','.join([f'a_{k}'for k in fields]+list(fields))
 if locals:
  HELPERS+=f'state {name}({params})->bool {{transition b {{{pat} -> check_{name}({args}) _ -> (false)}}}}\n'
  argspec=','.join([f'a_{k}:{v}'for k,v in fields.items()]+[f'{k}:{v}'for k,v in fields.items()]);HELPERS+=f'state check_{name}({argspec})->bool {{{locals}'+(' && '.join(exprs)or'true')+'}\n'
 else:HELPERS+=f'state {name}({params})->bool {{transition b {{{pat} -> ('+(' && '.join(exprs)or'true')+') _ -> (false)}}\n'
HELPERS+='}\n'
HELPERS+='''machine fx_store(a:&ObjectStore,b:&ObjectStore)->bool {
 let objects:bool=fx_objects(a,b,0,64,true);let entries:bool=fx_entries(&a.space,&b.space,0,32,true);
 a.space.object_count==b.space.object_count && a.space.count==b.space.count && objects && entries
}
machine fx_objects(a:&ObjectStore,b:&ObjectStore,i:u64,n:u64,prior:bool)
terminates by(i,n)->Nat::BoundedDistance;
->bool {let good:bool=fx_object(a,b,i,n);transition i<n {true -> fx_objects(a,b,i+1,n,prior && good) _ -> (prior)}}
machine fx_object(a:&ObjectStore,b:&ObjectStore,i:u64,n:u64)->bool {
 transition i<n && i<64 {true -> item(a,b,i) _ -> (true)}
 state item(a:&ObjectStore,b:&ObjectStore,i:u64)->bool {
  let left:Object=a.space.objects[i];let right:Object=b.space.objects[i];let first:ByteBlock=a.bytes.blocks[i];let second:ByteBlock=b.bytes.blocks[i];
  let payload:bool=fx_value(left.value,right.value);
  let bytes:bool=fx_bytes_equal(&first.bytes,&second.bytes,0,256,true);
  payload && bytes && left.has_next==right.has_next && left.next==right.next && first.initialized==second.initialized && first.length==second.length
 }
}
machine fx_entries(a:&Namespace,b:&Namespace,i:u64,n:u64,prior:bool)
terminates by(i,n)->Nat::BoundedDistance;
->bool {let good:bool=fx_entry(a,b,i,n);transition i<n {true -> fx_entries(a,b,i+1,n,prior && good) _ -> (prior)}}
machine fx_entry(a:&Namespace,b:&Namespace,i:u64,n:u64)->bool {
 transition i<n && i<32 {true -> item(a.entries[i],b.entries[i]) _ -> (true)}
 state item(a:Entry,b:Entry)->bool {let path:bool=fx_path(a.path,b.path);path && a.has_level==b.has_level && a.level==b.level && a.has_object==b.has_object && a.object==b.object && a.alias==b.alias}
}
'''
def span(start=0,end=2,unit=7):return f'Span {{unit:{unit},start:{start},end:{end}}}'
def source(kind='Buffer',declared=4,start=0,end=2,unit=7):
 if kind=='Buffer':return f'Value::Buffer {{buffer_storage:BufferStorage::Source {{declared_size:{declared},buffer_initializer:{span(start,end,unit)}}}}}'
 return f'Value::String {{string_storage:StringStorage::Source {{string_source:{span(start,end,unit)}}}}}'
def owned(kind='Buffer',owner=0):return f'Value::{kind} {{{kind.lower()}_storage:{kind}Storage::Owned {{{kind.lower()}_owner:{owner}}}}}'
def assignments(var,data):return ''.join(f'{var}[{i}]={v};'for i,v in enumerate(data)if v)
def seed(value=None,count=4,data=[42,19]):
 return 'let mut input:[u8;1024];'+assignments('input',data)+f'let mut store:ObjectStore=ObjectStore {{}};store.space.object_count={count};store.space.count=1;store.space.entries[0].has_object=true;store.space.entries[0].object=0;store.space.entries[0].alias=true;store.space.entries[0].path.count=1;store.space.entries[0].path.segments[0]=1145258561;store.space.objects[0].has_next=true;store.space.objects[0].next=3;store.space.objects[1].has_next=true;store.space.objects[1].next=2;store.space.objects[0].value={value or source()};store.bytes.blocks[63].bytes[255]=231;\n'
def block(var,at,data,length=None,kind='Buffer',tail=0):
 length=len(data)if length is None else length
 return f'{var}.bytes.blocks[{at}]=ByteBlock {{initialized:true,length:{length}}};'+assignments(f'{var}.bytes.blocks[{at}].bytes',data)+(f'{var}.bytes.blocks[{at}].bytes[255]={tail};'if tail else'')+f'{var}.space.objects[{at}].value={owned(kind,at)};'
def cases():
 rows=[]
 def add(name,setup,call,error='Success',changes='',check='',output=None,kind='Buffer',obj=0,length=0,value=0):
  body=setup+'let mut expected:ObjectStore=store;\n'+changes+'\n'+call+'\n'
  if output is not None:
   body+='let mut expected_bytes:[u8;256];'+assignments('expected_bytes',output)+'let output_good:bool=fx_bytes_equal(&result.bytes,&expected_bytes,0,256,true);\n'
   check=f'result.outcome==ByteOutcome::{error} && result.kind==ByteKind::{kind} && result.object=={obj} && result.length=={length} && output_good'+(' && '+check if check else'')
  else:check=f'result.outcome==ByteOutcome::{error} && result.object=={obj} && result.length=={length} && result.value=={value}'+(' && '+check if check else'')
  rows.append(dict(name=name,body=body,check=check))
 patterns=[('padded',[42,19],4,0,2),('long_initializer',[42,19,9,8],2,0,4),('empty',[],0,0,0),('zeroed',[],256,0,0),('full',list(range(256)),256,0,256),('offset',[0,0,81,82],4,2,4),('end_empty',[],0,1024,1024)]
 for label,data,declared,start,end in patterns:
  effective=max(declared,end-start);expect=data[start:end]+[0]*(effective-(end-start));length=max(len(data),end)
  setup=seed(source(declared=declared,start=start,end=end),data=data)
  add('read_'+label,setup,f'let result:ByteRead=read_bytes(&input,{length},7,&store,0);',output=expect,length=effective)
  add('materialize_'+label,setup,f'let result:ByteResult=materialize(&input,{length},7,&mut store,0);',changes=block('expected',0,expect),length=effective)
 for label,data in [('ascii',[65,66]),('empty',[]),('full',[65]*256)]:
  setup=seed(source('String',end=len(data)),data=data);add('string_'+label,setup,f'let result:ByteRead=read_bytes(&input,{len(data)},7,&store,0);',output=data,kind='String',length=len(data))
  add('string_materialize_'+label,setup,f'let result:ByteResult=materialize(&input,{len(data)},7,&mut store,0);',changes=block('expected',0,data,kind='String'),length=len(data))
 bad=[('span_reversed',source(start=2,end=1),2,7,'Bounds'),('span_end_max',source(end=MAX),2,7,'Bounds'),('span_start_max',source(start=MAX,end=MAX),2,7,'Bounds'),('wrong_unit',source(unit=8),2,7,'Bounds'),('input_length_max',source(),MAX,7,'Capacity'),('input_length_1025',source(),1025,7,'Capacity'),('declared_max',source(declared=MAX),2,7,'Capacity'),('declared_257',source(declared=257),2,7,'Capacity'),('initializer_257',source(end=257),257,7,'Capacity'),('wrong_type','Value::Integer {number:99}',2,7,'UnsupportedValue')]
 for label,payload,n,u,error in bad:
  setup=seed(payload);add('bad_read_'+label,setup,f'let result:ByteRead=read_bytes(&input,{n},{u},&store,0);',error,output=[])
  add('bad_materialize_'+label,setup,f'let result:ByteResult=materialize(&input,{n},{u},&mut store,0);',error)
 for label,data in [('nul',[65,0]),('late_nul',[65]*255+[0]),('nonascii',[255]),('late_nonascii',[65]*255+[128])]:
  setup=seed(source('String',end=len(data)),data=data);add('bad_string_'+label,setup,f'let result:ByteResult=materialize(&input,{len(data)},7,&mut store,0);','Encoding')
 for label,payload,extra,error in [('owner_other',owned(owner=1),'','InvalidState'),('owner_max',owned(owner=MAX),'','InvalidState'),('uninitialized',owned(),'','InvalidState'),('length_max',owned(),f'store.bytes.blocks[0].initialized=true;store.bytes.blocks[0].length={MAX};','Capacity'),('length_257',owned(),'store.bytes.blocks[0].initialized=true;store.bytes.blocks[0].length=257;','Capacity')]:
  add('bad_owned_'+label,seed(payload)+extra,'let result:ByteResult=materialize(&input,2,7,&mut store,0);',error)
 for count,obj,label in [(0,0,'empty'),(65,0,'bad_count'),(MAX,0,'max_count'),(4,4,'end_id'),(4,MAX,'max_id')]:
  add('bad_object_'+label,seed(count=count),f'let result:ByteRead=read_bytes(&input,2,7,&store,{obj});','InvalidState',output=[])
 setup=seed(owned())+block('store',0,[1,2,3],tail=213)
 add('read_owned_tail',setup,'let result:ByteRead=read_bytes(&input,0,999,&store,0);',output=[1,2,3]+[0]*252+[213],length=3)
 add('materialize_owned_idempotent',setup,'let result:ByteResult=materialize(&input,0,999,&mut store,0);',length=3)
 setup=seed(source(unit=MAX));add('source_unit_max',setup,f'let result:ByteRead=read_bytes(&input,2,{MAX},&store,0);',output=[42,19,0,0],length=4)
 for target in ['Uninitialized','Integer {number:99}','Package {first:2,count:1}','Reference {kind:ReferenceKind::RefOf,object_id:2}']:
  label=target.split()[0].lower();setup=seed()+f'store.space.objects[1].value=Value::{target};'
  add('clone_into_'+label,setup,'let result:ByteResult=clone_bytes_into(&input,2,7,&mut store,1,0);',changes=block('expected',1,[42,19,0,0]),obj=1,length=4)
 for kind in ['Buffer','String']:
  data=[65,66];setup=seed(owned(kind))+block('store',0,data,kind=kind,tail=213)
  add('clone_owned_'+kind.lower(),setup,'let result:ByteResult=clone_bytes_into(&input,2,7,&mut store,1,0);',changes=block('expected',1,data,kind=kind),obj=1,length=2)
  add('clone_self_'+kind.lower(),setup,'let result:ByteResult=clone_bytes_into(&input,2,7,&mut store,0,0);',changes=block('expected',0,data,kind=kind),length=2)
 for label,target in [('method','Value::Method {flags:3,body:Span {unit:7,end:2}}'),('field','Value::BufferField {backing_object:0,bit_length:8}'),('device','Value::Device'),('region','Value::OperationRegion {space:1,length:9}')]:
  add('clone_reject_'+label,seed()+f'store.space.objects[1].value={target};','let result:ByteResult=clone_bytes_into(&input,2,7,&mut store,1,0);','UnsupportedValue')
 add('clone_invalid_source',seed(owned(owner=3)),'let result:ByteResult=clone_bytes_into(&input,2,7,&mut store,1,0);','InvalidState')
 add('clone_invalid_destination_owner',seed()+f'store.space.objects[1].value={owned(owner=0)};','let result:ByteResult=clone_bytes_into(&input,2,7,&mut store,1,0);','InvalidState')
 for dest,src,label in [(MAX,0,'destination_max'),(1,MAX,'source_max')]:add('clone_'+label,seed(),f'let result:ByteResult=clone_bytes_into(&input,2,7,&mut store,{dest},{src});','InvalidState')
 # Full reference semantics are separately tested; these exercise storage integration.
 for kind in ['Named','Local','Arg','RefOf','Index','Unresolved']:
  setup=seed()+f'store.space.objects[1].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:0}};';good=kind in ['Named','Local','Arg']
  add('read_reference_'+kind.lower(),setup,'let result:ByteRead=read_bytes(&input,2,7,&store,1);','Success'if good else'UnsupportedValue',output=[42,19,0,0]if good else[],length=4 if good else 0)
 add('read_reference_cycle',seed()+ 'store.space.objects[0].value=Value::Reference {kind:ReferenceKind::Named,object_id:0};','let result:ByteRead=read_bytes(&input,2,7,&store,0);','ReferenceCycle',output=[])
 for label,start,count,outcome in [('byte',0,8,'Success'),('cross_byte',4,12,'Success'),('whole',0,32,'Success'),('zero',0,0,'Bounds'),('at_end',32,1,'Bounds'),('past_end',33,1,'Bounds'),('start_max',MAX,1,'Bounds'),('count_max',0,MAX,'Bounds')]:
  changes=f'expected.space.objects[4]=Object {{value:Value::BufferField {{backing_object:0,bit_offset:{start},bit_length:{count}}}}};expected.space.object_count=5;'if outcome=='Success'else''
  add('create_field_'+label,seed(),f'let result:ByteResult=create_buffer_field(&input,2,7,&mut store,0,{start},{count});',outcome,changes=changes,obj=4 if changes else 0,length=count if changes else 0)
 add('create_field_full',seed(count=64),'let result:ByteResult=create_buffer_field(&input,2,7,&mut store,0,0,8);','Capacity')
 for label,count,index,outcome in [('first',4,0,'Success'),('last',4,3,'Success'),('last_slots',62,0,'Success'),('one_slot',63,0,'Capacity'),('full',64,0,'Capacity'),('end',4,4,'Bounds'),('max',4,MAX,'Bounds')]:
  changes=f'expected.space.objects[{count}]=Object {{value:Value::BufferField {{backing_object:0,bit_offset:{index*8},bit_length:8}}}};expected.space.objects[{count+1}]=Object {{value:Value::Reference {{kind:ReferenceKind::RefOf,object_id:{count}}}}};expected.space.object_count={count+2};'if outcome=='Success'else''
  add('byte_index_'+label,seed(count=count),f'let result:ByteResult=make_byte_index(&input,2,7,&mut store,0,{index});',outcome,changes=changes,obj=count+1 if changes else 0,length=8 if changes else 0)
 # Field access is resolved afresh, including aliases and changed backing types.
 pattern=[(i*29+7)&255 for i in range(256)]
 for width in [4,8]:
  for label,start,count in [('low',0,8),('cross',5,17),('native',0,width*8),('last',2040,8),('too_wide',0,width*8+1)]:
   setup=seed(source(declared=256,end=256),data=pattern)+f'store.space.objects[1].value=Value::BufferField {{backing_object:0,bit_offset:{start},bit_length:{count}}};'
   good=count<=width*8;number=(int.from_bytes(bytes(pattern),'little')>>start)&((1<<count)-1) if good else 0
   add(f'field_read_{width}_{label}',setup,f'let result:ByteResult=read_field_integer(&input,256,7,&store,1,IntegerSize::{"FourBytes"if width==4 else"EightBytes"});','Success'if good else'UnsupportedValue',length=count if good else 0,value=number)
  for label,start,count,number in [('cross',5,17,MAX),('native',0,width*8,0x8877665544332211),('zero_pad',0,128,MAX),('last',2040,8,0x51)]:
   setup=seed(source(declared=256,end=256),data=pattern)+f'store.space.objects[1].value=Value::BufferField {{backing_object:0,bit_offset:{start},bit_length:{count}}};'
   data=bytearray(pattern);bits=int.from_bytes(data,'little');mask=((1<<count)-1)<<start;written=number&((1<<(width*8))-1);bits=(bits&~mask)|((written&((1<<count)-1))<<start);data=list(bits.to_bytes(256,'little'))
   add(f'field_write_{width}_{label}',setup,f'let result:ByteResult=write_field_integer(&input,256,7,&mut store,1,IntegerSize::{"FourBytes"if width==4 else"EightBytes"},{number});',changes=block('expected',0,data),length=256)
 for label,start,count,error in [('start_max',MAX,1,'Bounds'),('count_max',0,MAX,'Bounds'),('past_length',32,1,'Bounds'),('zero',0,0,'Bounds')]:
  setup=seed()+f'store.space.objects[1].value=Value::BufferField {{backing_object:0,bit_offset:{start},bit_length:{count}}};'
  add('field_bad_'+label,setup,'let result:ByteResult=write_field_integer(&input,2,7,&mut store,1,IntegerSize::EightBytes,42);',error)
 for label,number,error in [('ascii',67,'Success'),('nul',0,'Encoding'),('nonascii',255,'Encoding')]:
  setup=seed(source('String',end=2),data=[65,66])+'store.space.objects[1].value=Value::BufferField {backing_object:0,bit_offset:8,bit_length:8};';change=block('expected',0,[65,number],kind='String')if error=='Success'else''
  add('string_field_'+label,setup,f'let result:ByteResult=write_field_integer(&input,2,7,&mut store,1,IntegerSize::EightBytes,{number});',error,changes=change,length=2 if change else 0)
 setup=seed(owned())+block('store',0,[65,66],tail=213)+'store.space.objects[1].value=Value::BufferField {backing_object:0,bit_offset:8,bit_length:8};'
 add('write_preserves_owned_tail',setup,'let result:ByteResult=write_field_integer(&input,2,7,&mut store,1,IntegerSize::EightBytes,67);',changes=block('expected',0,[65,67],tail=213),length=2)
 setup=seed()+ 'store.space.objects[0].value=Value::Integer {number:5};store.space.objects[1].value=Value::BufferField {backing_object:0,bit_length:8};'
 add('field_backing_replaced_type',setup,'let result:ByteResult=write_field_integer(&input,2,7,&mut store,1,IntegerSize::EightBytes,67);','UnsupportedValue')
 # Sequential identity and independence witnesses.
 setup=seed()+ 'let cloned:ByteResult=clone_bytes_into(&input,2,7,&mut store,1,0);store.space.objects[2].value=Value::BufferField {backing_object:0,bit_length:8};'
 add('deep_copy_then_source_mutation',setup,'let result:ByteResult=write_field_integer(&input,2,7,&mut store,2,IntegerSize::EightBytes,99);',changes=block('expected',0,[99,19,0,0]),length=4,check='cloned.outcome==ByteOutcome::Success && store.bytes.blocks[1].bytes[0]==42')
 setup=seed()+ 'let indexed:ByteResult=make_byte_index(&input,2,7,&mut store,0,1);store.space.entries[0].object=1;'
 add('index_survives_name_rebinding',setup,'let result:ByteResult=write_field_integer(&input,2,7,&mut store,indexed.object,IntegerSize::EightBytes,99);',changes=block('expected',0,[42,99,0,0]),length=4,check='indexed.outcome==ByteOutcome::Success')
 setup=seed()+ 'let indexed:ByteResult=make_byte_index(&input,2,7,&mut store,0,1);'+block('store',0,[11,22])
 add('index_reads_replacement_payload',setup,'let result:ByteResult=read_field_integer(&input,2,7,&store,indexed.object,IntegerSize::EightBytes);',length=8,value=22)
 # Namespace-only copy refuses either side's owned locator, preserving whole store.
 for label,target,origin in [('source',False,True),('destination',True,False)]:
  setup=seed();setup+=block('store',0,[11,22])if origin else'';setup+=block('store',1,[33,44])if target else''
  call='let copied:NamespaceResult=copy_value(store.space,1,0);store.space=copied.space;let result:ByteResult=ByteResult {};'
  add('namespace_copy_owned_'+label,setup,call,check='copied.outcome==Outcome::InvalidState')
 return rows

def render(row,control=False,machine='test_result'):
 body=row['body']
 if control:body+='expected.bytes.blocks[63].bytes[255]=expected.bytes.blocks[63].bytes[255]^1;\n'
 body+='let unchanged:bool=fx_store(&store,&expected);\ntransition unchanged && '+row['check']+' {true -> (0) _ -> (1)}\n'
 return 'machine '+machine+('(&mut self)'if'::'in machine else'()')+'->i32 {\n'+body+'}\n'
if __name__=='__main__':
 (HERE/'cases.json').write_text(json.dumps(cases(),indent=2)+'\n');print(len(cases()),'owned-byte scenarios generated')
