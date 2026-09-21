#!/usr/bin/env python3
import argparse,collections,json
from pathlib import Path
import store_checks
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4];MAX=(1<<64)-1

def baseline_cases():
 rows=[]
 def add(name,count=8,start=0,data=b'Z',payload=b'A',owned=False,kind='Buffer',error=None,**kw):
  r=dict(name=name,count=count,start=start,data=list(data),payload=list(payload),owned=owned,kind=kind,error=error,declared=kw.pop('declared',len(data)),source_length=kw.pop('source_length',len(data)),unit=kw.pop('unit',7),field=kw.pop('field',0),object_count=kw.pop('object_count',2),extra=kw.pop('extra',''),backing_reference=kw.pop('backing_reference',None),top_reference=kw.pop('top_reference',None),alias=kw.pop('alias',False),payload_length=kw.pop('payload_length',len(payload)),public=kw.pop('public',True));assert not kw
  logical=data+b'\0'*max(0,r['declared']-len(data));r['logical']=list(logical)
  if error is None:
   value=int.from_bytes(logical,'little');source=int.from_bytes(logical if r['alias']else payload[:r['payload_length']],'little');mask=((1<<count)-1)<<start
   result=(value&~mask)|((source<<start)&mask);r['expected']=list(result.to_bytes(len(logical),'little'))
  else:r['expected']=None
  rows.append(r)
 for owned in [False,True]:
  for count in [1,8,9,31,32,33,63,64,65,127,2047,2048]:
   start=1 if count==2047 else 0 if count==2048 else 3
   for size in [0,1,8,9,256]:add(f'write_{count}_{size}_{owned}',count,start,b'\xa5'*((start+count+7)//8),b'\x96'*size,owned)
 for owned in [False,True]:
  add('nonuniform_'+str(owned),count=65,start=3,data=bytes.fromhex('1032547698badcfe80'),payload=bytes.fromhex('123456789abcdef011'),owned=owned)
  add('alias_snapshot_'+str(owned),count=16,start=8,data=bytes.fromhex('12345678'),payload=b'',owned=owned,alias=True,payload_length=4)
  add('dirty_tail_'+str(owned),count=9,start=3,data=b'AZ',payload=b'XY',owned=owned,extra='store.bytes.blocks[1].bytes[2]=231;store.bytes.blocks[1].bytes[255]=232;')
  for label,payload,error in [('ascii',b'B',None),('nul',b'\0','Encoding'),('nonascii',b'\x80','Encoding'),('zeroextend',b'','Encoding')]:add('string_'+label+'_'+str(owned),data=b'AZ',payload=payload,owned=owned,kind='String',error=error,public=label!='nonascii')
  add('string_wide_'+str(owned),count=65,start=0,data=b'A'*9,payload=b'B'*9,owned=owned,kind='String')
  add('bad_original_string_'+str(owned),data=b'A\0',payload=b'B',owned=owned,kind='String',error='Encoding')
  add('string_late_encoding_'+str(owned),data=b'A'*255+b'\0',payload=b'B',owned=owned,kind='String',error='Encoding')
 add('payload_tail_ignored',count=16,data=b'ZZ',payload=b'A'+b'\xff'*255,payload_length=1)
 add('empty_dirty_payload',count=16,data=b'ZZ',payload=b'\xff'*256,payload_length=0)
 for label,count,start in [('zero',0,0),('outside',1,8),('count_past',9,0),('offset_max',1,MAX),('count_max',MAX,0)]:add(label,count,start,error='Bounds',public=label=='zero')
 for label,kw in [('field_max',dict(field=MAX)),('field_past',dict(field=2)),('count65',dict(object_count=65)),('count_maximum',dict(object_count=MAX)),('empty_store',dict(object_count=0))]:add(label,error='InvalidState',public=False,**kw)
 for label,kw in [('payload257',dict(payload_length=257)),('payload_max',dict(payload_length=MAX)),('payload_before_field',dict(payload_length=MAX,field=MAX))]:add(label,error='Capacity',public=False,**kw)
 for kind in ['Buffer','String']:
  add('wrong_owner_'+kind,kind=kind,owned=True,error='InvalidState',public=False,extra='store.space.objects[1].value=Value::'+kind+' {'+('buffer_storage:BufferStorage::Owned {buffer_owner:0}'if kind=='Buffer'else'string_storage:StringStorage::Owned {string_owner:0}')+'};')
  add('uninitialized_'+kind,kind=kind,owned=True,error='InvalidState',public=False,extra='store.bytes.blocks[1].initialized=false;')
  add('length_max_'+kind,kind=kind,owned=True,error='Capacity',public=False,extra=f'store.bytes.blocks[1].length={MAX};')
  add('wrong_unit_'+kind,kind=kind,error='Bounds',public=False,unit=8)
  add('source_length_'+kind,kind=kind,error='Capacity',public=False,source_length=MAX)
  add('truncated_'+kind,kind=kind,error='Bounds',public=False,source_length=0)
  add('owned_ignores_source_'+kind,kind=kind,owned=True,source_length=MAX)
 for ref in ['Named','Local','Arg','RefOf','Index','Unresolved']:
  add('backing_'+ref,backing_reference=ref,object_count=3,error='UnsupportedValue'if ref in ['RefOf','Index','Unresolved']else None)
  add('outer_'+ref,top_reference=ref,object_count=3,error='UnsupportedValue')
 add('backing_cycle',error='ReferenceCycle',public=False,extra='store.space.objects[1].value=Value::Reference {kind:ReferenceKind::Named,object_id:1};')
 add('backing_max',error='InvalidState',public=False,extra=f'store.space.objects[0].value=Value::BufferField {{backing_object:{MAX},bit_length:8}};')
 add('backing_integer',error='UnsupportedValue',public=False,extra='store.space.objects[1].value=Value::Integer {number:90};')
 add('direct_integer',error='UnsupportedValue',public=False,extra='store.space.objects[0].value=Value::Integer {number:90};')
 add('bad_backing_before_zero',count=0,error='InvalidState',public=False,extra='store.space.objects[0].value=Value::BufferField {backing_object:64,bit_length:0};')
 add('encoding_before_zero',count=0,data=b'A\0',kind='String',error='Encoding')
 add('virtual_padding',count=2048,data=b'AZ',declared=256,payload=b'BC')
 add('initializer_larger',count=32,data=b'AZBY',declared=1,payload=b'CD')
 assert len({r['name']for r in rows})==len(rows)
 return rows

IMPORTS='''use aml::buffer_field_store::store_value;
use integer_helpers::integers::IntegerSize;
use aml::byte_storage::read_bytes;
use aml::byte_storage::ByteRead;
use aml::byte_storage::ByteResult;
use aml::byte_storage::ByteOutcome;
use aml::model::ObjectStore;
use aml::model::Object;
use aml::model::Value;
use aml::model::Span;
use aml::model::Path;
use aml::model::Namespace;
use aml::model::Entry;
use aml::model::ByteBlock;
use aml::model::BufferStorage;
use aml::model::StringStorage;
use aml::model::ReferenceKind;
'''
HELPERS=store_checks.HELPERS+'''machine write_test_fill_bytes(output:&mut [u8;256],index:u64,count:u64,value:u8)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_ = write_test_byte(output,index,count,value);transition index<count {true -> write_test_fill_bytes(output,index+1,count,value) _ -> (0)}}
machine write_test_byte(output:&mut [u8;256],index:u64,count:u64,value:u8)->u8 {transition index<count && index<256 {true -> put(output,index,value) _ -> (0)}state put(output:&mut [u8;256],index:u64,value:u8)->u8 {output[index]=value;0}}
machine write_test_fill_source(output:&mut [u8;1024],index:u64,count:u64,value:u8)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_ = write_test_source(output,index,count,value);transition index<count {true -> write_test_fill_source(output,index+1,count,value) _ -> (0)}}
machine write_test_source(output:&mut [u8;1024],index:u64,count:u64,value:u8)->u8 {transition index<count && index<1024 {true -> put(output,index,value) _ -> (0)}state put(output:&mut [u8;1024],index:u64,value:u8)->u8 {output[index]=value;0}}
'''
def assignments(place,data,source=False):
 if not data:return ''
 value=collections.Counter(data).most_common(1)[0][0];s=f'_ = write_test_fill_{"source"if source else"bytes"}(&mut {place},0,{len(data)},{value});'
 s+=''.join(f'{place}[{i}]={v};'for i,v in enumerate(data)if v!=value)
 return s

def render(r,control=False,machine='test_result()',constant=False):
 body='let mut input:[u8;1024];let mut store:ObjectStore=ObjectStore {};'+f'store.space.object_count={r["object_count"]};store.space.count=1;store.space.entries[0].has_object=true;store.space.entries[0].object=0;store.space.entries[0].path.absolute=true;store.space.entries[0].path.count=1;store.space.entries[0].path.segments[0]=809782342;'
 body+=f'store.space.entries[31].object={MAX};store.space.entries[31].alias=true;store.space.entries[31].path.segments[15]=4294967295;store.space.objects[0].has_next=true;store.space.objects[0].next=1;store.space.objects[1].has_next=true;store.space.objects[1].next=0;store.space.objects[63].value=Value::OperationRegion {{space:9,base:{MAX},length:17,scope:Path {{absolute:true,count:1}}}};store.space.objects[62].value=Value::Method {{flags:3,body:Span {{unit:77,start:12,end:34}}}};store.bytes.blocks[63].length={MAX};store.bytes.blocks[63].bytes[0]=17;store.bytes.blocks[63].bytes[127]=18;store.bytes.blocks[63].bytes[255]=19;'
 body+=assignments('input',r['data'],True);kind=r['kind'];owner=2 if r['backing_reference']else 1
 if r['owned']:
  storage=f'BufferStorage::Owned {{buffer_owner:{owner}}}'if kind=='Buffer'else f'StringStorage::Owned {{string_owner:{owner}}}'
  body+=f'store.bytes.blocks[{owner}].initialized=true;store.bytes.blocks[{owner}].length={len(r["logical"])};'+assignments(f'store.bytes.blocks[{owner}].bytes',r['data'])
 else:
  span=f'Span {{unit:7,end:{len(r["data"])}}}'
  storage=f'BufferStorage::Source {{declared_size:{r["declared"]},buffer_initializer:{span}}}'if kind=='Buffer'else f'StringStorage::Source {{string_source:{span}}}'
 body+=f'store.space.objects[{owner}].value=Value::{kind} {{{"buffer_storage"if kind=="Buffer"else"string_storage"}:{storage}}};'
 field=f'Value::BufferField {{backing_object:1,bit_offset:{r["start"]},bit_length:{r["count"]}}}'
 body+=f'store.space.objects[0].value={field};'
 if r['backing_reference']:body+=f'store.space.objects[1].value=Value::Reference {{kind:ReferenceKind::{r["backing_reference"]},object_id:2}};'
 if r['top_reference']:body+=f'store.space.objects[2].value={field};store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{r["top_reference"]},object_id:2}};'
 body+=r['extra']+source_setup(r,owner)+'let mut expected:ObjectStore=store;'
 body+=f'let result:ByteResult=store_value(&input,{r["source_length"]},{r["unit"]},&mut store,{r["field"]},{r["source_id"]},IntegerSize::{"FourBytes"if r["bits"]==32 else"EightBytes"});'
 if r['error']is None:
  if not r['owned']:body+=f'expected.bytes.blocks[{owner}]=ByteBlock {{}};'
  body+=f'expected.bytes.blocks[{owner}].initialized=true;expected.bytes.blocks[{owner}].length={len(r["expected"])};'+assignments(f'expected.bytes.blocks[{owner}].bytes',r['expected'])
  payload=f'Value::Buffer {{buffer_storage:BufferStorage::Owned {{buffer_owner:{owner}}}}}'if kind=='Buffer'else f'Value::String {{string_storage:StringStorage::Owned {{string_owner:{owner}}}}}'
  body+=f'expected.space.objects[{owner}].value={payload};'
 if control:body+=f'expected.bytes.blocks[{owner if constant else 63}].bytes[255]=expected.bytes.blocks[{owner if constant else 63}].bytes[255]^1;'
 if constant:body+=f'let bytes_good:bool=fx_bytes_equal(&store.bytes.blocks[{owner}].bytes,&expected.bytes.blocks[{owner}].bytes,0,256,true);let value_good:bool=fx_value(store.space.objects[{owner}].value,expected.space.objects[{owner}].value);let same:bool=bytes_good && value_good && store.space.object_count==expected.space.object_count && store.space.count==expected.space.count;'
 else:body+='let same:bool=fx_store(&store,&expected);'
 body+=f'let good:bool=same && result.outcome==ByteOutcome::{r["error"]or"Success"} && result.object=={0 if r["error"]else owner} && result.length=={0 if r["error"]else len(r["expected"])};'
 return 'machine '+machine+'->i32 {'+body+'transition good {true -> (0) _ -> (1)}}\n'
def source_setup(r,owner):
 if r['alias']:return r['source_extra']
 kind=r['source_kind'];at=3;data=r['source_data'];code=''
 if kind=='Integer':value=f'Integer {{number:{r["number"]}}}'
 elif kind in ['Buffer','String']:
  if r['source_owned']:
   storage=f'BufferStorage::Owned {{buffer_owner:{at}}}'if kind=='Buffer'else f'StringStorage::Owned {{string_owner:{at}}}'
   code+=f'store.bytes.blocks[{at}].initialized=true;store.bytes.blocks[{at}].length={r["source_extent"]};'+assignments(f'store.bytes.blocks[{at}].bytes',data)
  else:
   code+=f'_=write_test_fill_source(&mut input,512,{512+len(data)},{collections.Counter(data).most_common(1)[0][0]});'if data else''
   code+=''.join(f'input[{512+i}]={v};'for i,v in enumerate(data)if v!=collections.Counter(data).most_common(1)[0][0])
   span=f'Span {{unit:7,start:512,end:{512+len(data)}}}'
   storage=f'BufferStorage::Source {{declared_size:{r["source_extent"]},buffer_initializer:{span}}}'if kind=='Buffer'else f'StringStorage::Source {{string_source:{span}}}'
  value=f'{kind} {{{"buffer_storage"if kind=="Buffer"else"string_storage"}:{storage}}}'
 else:value={'Uninitialized':'Uninitialized','Device':'Device','Package':'Package {first:0,count:0}','Method':'Method {flags:0,body:Span {}}','NameReference':'NameReference {name:Path {},scope:Path {}}','BufferField':'BufferField {backing_object:1,bit_length:8}'}[kind]
 return code+f'store.space.objects[{at}].value=Value::{value};'+r['source_extra']

def cases():
 import copy
 base=baseline_cases();rows=[]
 def normalize(r):
  r=copy.deepcopy(r);r.update(source_id=(2 if r['backing_reference']else 1)if r['alias']else 3,source_kind='Buffer',source_data=r['payload'],source_extent=r['payload_length'],source_owned=True,source_extra='',bits=64,number=0)
  if r['object_count']in [2,3]:r['object_count']=4
  if r['name']=='field_past':r['field']=4
  return r
 for r in base:
  if r['name'].startswith('write_')and not(r['count']in [1,9,32,65,2048]and len(r['payload'])in [0,8,256]):continue
  rows.append(normalize(r))
 for original in base:
  if original['name'].startswith('alias_snapshot_'):
   r=normalize(original);r['kind']='String';r['name']='string_'+r['name'];rows.append(r)
 template=normalize(next(r for r in base if r['name']=='write_9_8_True'))
 def add(name,kind='Integer',data=b'',number=0,bits=64,width=9,owned=True,extent=None,error=None,source_extra='',source_id=3,field=0):
  r=copy.deepcopy(template);start=0 if width==2048 else 3;backing=b'\xa5'*((start+width+7)//8)
  r.update(name=name,count=width,start=start,data=list(backing),logical=list(backing),declared=len(backing),source_length=1024,owned=True,source_kind=kind,source_data=list(data),source_extent=len(data)if extent is None else extent,source_owned=owned,number=number,bits=bits,source_extra=source_extra,source_id=source_id,field=field,error=error,payload=[],payload_length=0)
  if not error:
   raw=(number&((1<<bits)-1)).to_bytes(bits//8,'little')if kind=='Integer'else data+b'\0'*max(0,r['source_extent']-len(data))
   original=int.from_bytes(backing,'little');mask=((1<<width)-1)<<start;result=(original&~mask)|((int.from_bytes(raw,'little')<<start)&mask);r['expected']=list(result.to_bytes(len(backing),'little'))
  else:r['expected']=None
  rows.append(r)
 for bits in [32,64]:
  for width in [1,9,32,33,64,65,2048]:
   for number in [0,1<<32,MAX]:add(f'integer_{bits}_{width}_{number}',number=number,bits=bits,width=width)
 for owned in [False,True]:
  for n in [0,1,255,256]:
   for width in [1,8,9,64,2048]:add(f'string_{owned}_{n}_{width}',kind='String',data=b'A'*n,width=width,owned=owned)
  for label,data in [('nul',b'A\0'),('high',b'A\xff'),('late',b'A'*255+b'\0')]:
   add(f'source_string_{owned}_{label}',kind='String',data=data,owned=owned,error='Encoding')
   add(f'source_string_before_field_{owned}_{label}',kind='String',data=data,owned=owned,error='Encoding',field=MAX)
  for kind in ['Buffer','String']:
   add(f'source_{owned}_{kind}_256',kind=kind,data=b'A'*256,width=2048,owned=owned)
 for width in [9,65,2048]:add(f'source_virtual_{width}',kind='Buffer',data=b'AB',extent=256,width=width,owned=False)
 for kind in ['Uninitialized','Device','Package','Method','NameReference','BufferField']:add(f'source_unsupported_{kind}',kind=kind,error='UnsupportedValue')
 for ref in ['Named','Local','Arg','RefOf','Index','Unresolved']:add(f'source_ref_{ref}',error='UnsupportedValue',source_extra=f'store.space.objects[3].value=Value::Reference {{kind:ReferenceKind::{ref},object_id:1}};')
 for index in [4,1<<63,MAX]:add(f'source_id_{index}',source_id=index,error='InvalidState')
 for kind in ['Buffer','String']:
  for label,extra,error in [('owner',f'store.space.objects[3].value=Value::{kind} {{{"buffer_storage:BufferStorage::Owned {buffer_owner:1}"if kind=="Buffer"else"string_storage:StringStorage::Owned {string_owner:1}"}}};','InvalidState'),('uninitialized','store.bytes.blocks[3].initialized=false;','InvalidState'),('length',f'store.bytes.blocks[3].length={MAX};','Capacity')]:add(f'source_owned_{kind}_{label}',kind=kind,data=b'A',error=error,source_extra=extra)
  add(f'source_span_{kind}',kind=kind,data=b'A',owned=False,error='Bounds',source_extra=f'store.space.objects[3].value=Value::{kind} {{{"buffer_storage:BufferStorage::Source {declared_size:1,buffer_initializer:Span {unit:8,start:512,end:513}}"if kind=="Buffer"else"string_storage:StringStorage::Source {string_source:Span {unit:8,start:512,end:513}}"}}};')
 add('source_integer_unused_snapshot',number=MAX,width=65)
 rows[-1]['source_length']=MAX
 # Both source and backing are owned; integer ignores this invalid input geometry.
 assert len(rows)==len({r['name']for r in rows});return rows

def body(row,control,constant=False):
 text=render(row,control,constant=constant);return text.split('->i32 {',1)[1].rsplit('}\n',1)[0]
if __name__=='__main__':
 (HERE/'cases.json').write_text(json.dumps(cases(),indent=2,sort_keys=True)+'\n');print(len(cases()),'direct BufferField source-store pairs')
