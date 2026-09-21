#!/usr/bin/env python3
import argparse,collections,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4];MAX=(1<<64)-1

def cases():
 rows=[]
 def add(name,count=8,start=0,data=b'Z',bits=64,owned=False,kind='Buffer',error=None,**kw):
  r=dict(name=name,count=count,start=start,data=list(data),bits=bits,owned=owned,kind=kind,error=error,declared=kw.pop('declared',len(data)),source_length=kw.pop('source_length',len(data)),unit=kw.pop('unit',7),field=kw.pop('field',0),object_count=kw.pop('object_count',2),extra=kw.pop('extra',''),backing_reference=kw.pop('backing_reference',None),top_reference=kw.pop('top_reference',None),public=kw.pop('public',True));assert not kw
  if error is None:
   material=data+b'\0'*max(0,r['declared']-len(data))
   n=(int.from_bytes(material,'little')>>start)&((1<<count)-1)
   r['expected_integer']=n if count<=bits else None;r['expected_buffer']=list(n.to_bytes((count+7)//8,'little'))if count>bits else None
  else:r['expected_integer']=r['expected_buffer']=None
  rows.append(r)
 for bits in [32,64]:
  for owned in [False,True]:
   for count in [1,4,5,8,9,31,32,33,63,64,65,127,128,129,2047,2048]:
    for start in [0,1,7]:
     if start+count>2048:continue
     add(f'bits_{bits}_{count}_at_{start}_{owned}',count,start,b'\xa5'*((start+count+7)//8),bits,owned)
 for bits in [32,64]:
  for owned in [False,True]:
   for count in [8,33,64,65,127]:add(f'string_{bits}_{count}_{owned}',count,data=b'A'*16,bits=bits,owned=owned,kind='String')
 for bits in [32,64]:
  for owned in [False,True]:
   for count in [32,64,65]:
    for start in [3,7]:add(f'nonuniform_{bits}_{count}_{start}_{owned}',count,start,bytes.fromhex('1032547698badcfe80'),bits,owned)
 add('owned_wide_dirty_tail',count=65,data=b'\xa5'*9,owned=True,extra='store.bytes.blocks[1].bytes[9]=255;store.bytes.blocks[1].bytes[255]=255;')
 add('source_last_byte',source_length=1024,public=False,extra='input[1023]=90;store.space.objects[1].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:1,buffer_initializer:Span {unit:7,start:1023,end:1024}}};')
 add('source_max_span',error='Bounds',public=False,extra=f'store.space.objects[1].value=Value::Buffer {{buffer_storage:BufferStorage::Source {{buffer_initializer:Span {{unit:7,start:{MAX},end:{MAX}}}}}}};')
 for label,count,start,error in [('zero',0,0,'Bounds'),('past_end',1,8,'Bounds'),('past_extent',9,0,'Bounds'),('start_max',1,MAX,'Bounds'),('count_max',MAX,0,'Bounds')]:add(label,count,start,error=error,public=label!='count_max')
 for label,kw in [('empty_store',dict(object_count=0)),('count65',dict(object_count=65)),('count_maximum',dict(object_count=MAX)),('field_max',dict(field=MAX)),('field_outside',dict(field=2))]:add(label,error='InvalidState',public=False,**kw)
 for kind in ['Buffer','String']:
  for owned in [False,True]:
   stem=kind+'_'+str(owned)
   if owned:
    add('wrong_owner_'+stem,kind=kind,owned=True,error='InvalidState',public=False,extra='store.space.objects[1].value=Value::'+kind+' {'+('buffer_storage:BufferStorage::Owned {buffer_owner:0}'if kind=='Buffer'else'string_storage:StringStorage::Owned {string_owner:0}')+'};')
    add('uninitialized_'+stem,kind=kind,owned=True,error='InvalidState',public=False,extra='store.bytes.blocks[1].initialized=false;')
    add('length_max_'+stem,kind=kind,owned=True,error='Capacity',public=False,extra=f'store.bytes.blocks[1].length={MAX};')
    add('dirty_tail_'+stem,kind=kind,owned=True,extra='store.bytes.blocks[1].bytes[1]=255;store.bytes.blocks[1].bytes[255]=255;')
    add('unused_source_'+stem,kind=kind,owned=True,source_length=MAX,unit=8)
   else:
    add('source_unit_'+stem,kind=kind,error='Bounds',public=False,unit=8)
    add('source_length_'+stem,kind=kind,error='Capacity',public=False,source_length=MAX)
    add('truncated_'+stem,kind=kind,error='Bounds',public=False,source_length=0)
    add('reversed_'+stem,kind=kind,error='Bounds',public=False,extra='store.space.objects[1].value=Value::'+kind+' {'+('buffer_storage:BufferStorage::Source {declared_size:1,buffer_initializer:Span {unit:7,start:1,end:0}}'if kind=='Buffer'else'string_storage:StringStorage::Source {string_source:Span {unit:7,start:1,end:0}}')+'};')
 for owned in [False,True]:
  for label,data in [('nul',b'A\0'),('nonascii',b'A\xc2\x80'),('late_nul',b'A'*255+b'\0')]:add('encoding_'+label+'_'+str(owned),data=data,owned=owned,kind='String',error='Encoding')
 for ref in ['Named','Local','Arg','RefOf','Index','Unresolved']:
  add('backing_'+ref,backing_reference=ref,object_count=3,error='UnsupportedValue'if ref in ['RefOf','Index','Unresolved']else None)
  add('outer_'+ref,top_reference=ref,object_count=3,error='UnsupportedValue')
 add('backing_cycle',error='ReferenceCycle',public=False,extra='store.space.objects[1].value=Value::Reference {kind:ReferenceKind::Named,object_id:1};')
 add('backing_missing',error='InvalidState',public=False,extra='store.space.objects[0].value=Value::BufferField {backing_object:64,bit_length:8};')
 add('backing_integer',error='UnsupportedValue',public=False,extra='store.space.objects[1].value=Value::Integer {number:90};')
 add('direct_integer',error='UnsupportedValue',public=False,extra='store.space.objects[0].value=Value::Integer {number:90};')
 add('virtual_zero_padding',count=2048,data=b'AZ',declared=256)
 add('initializer_larger',count=32,data=b'AZBY',declared=1)
 add('encoding_before_range',count=0,data=b'A\0',kind='String',error='Encoding')
 add('bad_owner_before_range',count=0,owned=True,error='InvalidState',public=False,extra='store.bytes.blocks[1].initialized=false;')
 assert len({r['name']for r in rows})==len(rows)
 return rows

IMPORTS='''use aml::buffer_field_values::read;
use aml::buffer_field_values::FieldValueRead;
use aml::buffer_field_values::FieldReadFailure;
use aml::model::ObjectStore;
use aml::model::Value;
use aml::model::Span;
use aml::model::BufferStorage;
use aml::model::StringStorage;
use aml::model::ReferenceKind;
use integer_helpers::integers::IntegerSize;
'''
HELPERS='''machine field_test_fill_bytes(output:&mut [u8;256],index:u64,count:u64,value:u8)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_ = field_test_write_byte(output,index,count,value);transition index<count {true -> field_test_fill_bytes(output,index+1,count,value) _ -> (0)}}
machine field_test_write_byte(output:&mut [u8;256],index:u64,count:u64,value:u8)->u8 {transition index<count && index<256 {true -> write(output,index,value) _ -> (0)}state write(output:&mut [u8;256],index:u64,value:u8)->u8 {output[index]=value;0}}
machine field_test_fill_source(output:&mut [u8;1024],index:u64,count:u64,value:u8)
terminates by(index,count)->Nat::BoundedDistance;
->u8 {_ = field_test_write_source(output,index,count,value);transition index<count {true -> field_test_fill_source(output,index+1,count,value) _ -> (0)}}
machine field_test_write_source(output:&mut [u8;1024],index:u64,count:u64,value:u8)->u8 {transition index<count && index<1024 {true -> write(output,index,value) _ -> (0)}state write(output:&mut [u8;1024],index:u64,value:u8)->u8 {output[index]=value;0}}
machine field_test_equal(a:&[u8;256],b:&[u8;256],index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let equal:bool=field_test_equal_byte(a,b,index);transition index<count {true -> field_test_equal(a,b,index+1,count,prior && equal) _ -> (prior)}}
machine field_test_equal_byte(a:&[u8;256],b:&[u8;256],index:u64)->bool {transition index<256 {true -> (a[index]==b[index]) _ -> (true)}}
machine field_test_failure(result:FieldValueRead,expected:FieldReadFailure)->bool {transition result {FieldValueRead::Failure {reason} -> (reason==expected) _ -> (false)}}
machine field_test_integer(result:FieldValueRead,expected:u64)->bool {transition result {FieldValueRead::Integer {number} -> (number==expected) _ -> (false)}}
machine field_test_buffer(result:FieldValueRead,expected_length:u64,expected:&[u8;256])->bool {transition result {FieldValueRead::Buffer {length,bytes} -> compare(length,bytes,expected_length,expected) _ -> (false)}state compare(length:u64,bytes:[u8;256],expected_length:u64,expected:&[u8;256])->bool {let equal:bool=field_test_equal(&bytes,expected,0,256,true);length==expected_length && equal}}
'''
def assignments(place,data,source=False):
 if not data:return ''
 value=collections.Counter(data).most_common(1)[0][0];s=''
 if value:s=f'_ = field_test_fill_{"source"if source else"bytes"}(&mut {place},0,{len(data)},{value});'
 s+=''.join(f'{place}[{i}]={v};'for i,v in enumerate(data)if v!=value)
 return s

def render(r,control=False,machine='test_result()'):
 body='let mut input:[u8;1024];let mut store:ObjectStore=ObjectStore {};'+f'store.space.object_count={r["object_count"]};'
 body+=assignments('input',r['data'],True)
 kind=r['kind'];owner=2 if r['backing_reference']else 1
 if r['owned']:
  storage=f'BufferStorage::Owned {{buffer_owner:{owner}}}'if kind=='Buffer'else f'StringStorage::Owned {{string_owner:{owner}}}'
  body+=f'store.bytes.blocks[{owner}].initialized=true;store.bytes.blocks[{owner}].length={max(len(r["data"]),r["declared"])};'+assignments(f'store.bytes.blocks[{owner}].bytes',r['data'])
 else:
  span=f'Span {{unit:7,end:{len(r["data"])}}}'
  storage=f'BufferStorage::Source {{declared_size:{r["declared"]},buffer_initializer:{span}}}'if kind=='Buffer'else f'StringStorage::Source {{string_source:{span}}}'
 body+=f'store.space.objects[{owner}].value=Value::{kind} {{{"buffer_storage"if kind=="Buffer"else"string_storage"}:{storage}}};'
 field=f'Value::BufferField {{backing_object:1,bit_offset:{r["start"]},bit_length:{r["count"]}}}'
 body+=f'store.space.objects[0].value={field};'
 if r['backing_reference']:body+=f'store.space.objects[1].value=Value::Reference {{kind:ReferenceKind::{r["backing_reference"]},object_id:2}};'
 if r['top_reference']:body+=f'store.space.objects[2].value={field};store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{r["top_reference"]},object_id:2}};'
 body+=r['extra']
 body+=f'let result:FieldValueRead=read(&input,{r["source_length"]},{r["unit"]},&store,{r["field"]},IntegerSize::{"FourBytes"if r["bits"]==32 else"EightBytes"});'
 if r['error']:
  error=('Bounds'if r['error']!='Bounds'else'InvalidState')if control else r['error'];body+=f'let good:bool=field_test_failure(result,FieldReadFailure::{error});'
 elif r['expected_integer']is not None:
  value=r['expected_integer']^(1 if control else 0);body+=f'let good:bool=field_test_integer(result,{value});'
 else:
  data=r['expected_buffer'];body+='let mut expected:[u8;256];'+assignments('expected',data)
  if control:body+='expected[255]=expected[255]^1;'
  body+=f'let good:bool=field_test_buffer(result,{len(data)},&expected);'
 return 'machine '+machine+'->i32 {'+body+'transition good {true -> (0) _ -> (1)}}\n'
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();text=json.dumps(cases(),indent=2,sort_keys=True)+'\n';out=HERE/'cases.json'
 if a.check:assert out.read_text()==text
 else:out.write_text(text)
 print(len(cases()),'detached field-read cases')
