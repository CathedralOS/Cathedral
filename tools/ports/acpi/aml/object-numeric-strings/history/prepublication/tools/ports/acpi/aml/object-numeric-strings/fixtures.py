#!/usr/bin/env python3
import json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4];MAX=(1<<64)-1

def cases():
 rows=[]
 def add(name,kind,format='Decimal',data=b'',number=0,bits=64,owned=False,error=None,**kw):
  declared=kw.pop('declared',len(data));logical=data+bytes(max(0,declared-len(data)))if kind=='Buffer'and declared<=256 else data
  if kind=='Integer':expected=str(number&((1<<bits)-1))if format=='Decimal'else'0x'+format_number(number&((1<<bits)-1))
  elif kind=='String':expected=data.decode('ascii',errors='replace')
  else:expected=','.join(str(v)if format=='Decimal'else f'0x{v:02X}'for v in logical)
  if len(expected)>256 and error is None:error='Capacity'
  rows.append(dict(name=name,kind=kind,format=format,data=list(data),number=number,bits=bits,owned=owned,error=error,expected=list(expected.encode('ascii',errors='replace'))if error is None else None,declared=declared,extra=kw.pop('extra',''),source_length=kw.pop('source_length',len(data)),unit=kw.pop('unit',7),object=kw.pop('object',0),object_count=kw.pop('object_count',1)));assert not kw
 for fmt in ['Decimal','Hexadecimal']:
  for bits in [32,64]:
   for n in [0,1,9,10,15,16,255,0xffffffff,0x100000000,0x1234567887654321,MAX]:add(f'integer_{fmt}_{bits}_{n}','Integer',fmt,number=n,bits=bits)
  for owned in [False,True]:
   for length in [0,1,2,50,51,52,63,64,65,85,86,128,129,255,256]:add(f'buffer_{fmt}_{length}_{owned}','Buffer',fmt,data=b'\xff'*length,owned=owned)
   for data in [b'\0',b'\x09\x0a\x0f\x10\x63\x64\xff',bytes(range(16)),b'\0'*128,b'\0'*129,b'\0'*127+b'\x0a']:
    add(f'buffer_pattern_{fmt}_{len(data)}_{data[-1]if data else 0}_{owned}','Buffer',fmt,data=data,owned=owned)
   for data in [b'',b'0',b'0xnot-a-number',b' \t42 ! ',b'A'*255,b'A'*256]:add(f'string_{fmt}_{len(data)}_{owned}','String',fmt,data=data,owned=owned)
   for kind in ['String','Buffer']:
    add(f'tail_{kind}_{fmt}_{owned}',kind,fmt,data=b'42',owned=owned,extra='store.bytes.blocks[0].bytes[2]=255;store.bytes.blocks[0].bytes[255]=254;input[2]=255;input[1023]=254;')
    for name,extra,err in [('wrong_owner','OWNER','InvalidState'),('uninitialized','store.bytes.blocks[0].initialized=false;','InvalidState'),('length257','store.bytes.blocks[0].length=257;','Capacity'),('length_max',f'store.bytes.blocks[0].length={MAX};','Capacity')]:
     if owned:
      if extra=='OWNER':extra='store.space.objects[0].value=Value::'+kind+' {'+('string_storage:StringStorage::Owned {string_owner:1}'if kind=='String'else'buffer_storage:BufferStorage::Owned {buffer_owner:1}')+'};'
      add(f'{name}_{kind}_{fmt}',kind,fmt,data=b'42',owned=True,error=err,extra=extra)
    if not owned:
     for label,kw in [('unit',dict(unit=8,error='Bounds')),('source_max',dict(source_length=MAX,error='Capacity')),('source_short',dict(source_length=1,error='Bounds')),('source_1025',dict(source_length=1025,error='Capacity'))]:add(f'{label}_{kind}_{fmt}',kind,fmt,data=b'42',**kw)
   for bad in [b'\0',b'\x80',b'A'*255+b'\0',b'A'*255+b'\xff']:add(f'encoding_{fmt}_{len(bad)}_{bad[-1]}_{owned}','String',fmt,data=bad,owned=owned,error='Encoding')
  add(f'padding_{fmt}','Buffer',fmt,data=b'A',declared=4)
  add(f'long_initializer_{fmt}','Buffer',fmt,data=b'ABCD',declared=1)
  add(f'declared_max_{fmt}','Buffer',fmt,declared=MAX,error='Capacity')
  add(f'owned_unused_source_{fmt}','String',fmt,data=b'A',owned=True,source_length=MAX,unit=MAX)
  add(f'integer_unused_source_{fmt}','Integer',fmt,number=42,source_length=MAX,unit=MAX)
  for kind in ['Uninitialized','Package','NameReference','Method','OperationRegion','Device','BufferField']:add(f'unsupported_{fmt}_{kind}',kind,fmt,error='UnsupportedValue')
  for ref in ['Named','RefOf','Local','Arg','Index','Unresolved']:add(f'reference_{fmt}_{ref}','Reference',fmt,error='UnsupportedValue',object_count=2,extra=f'store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{ref},object_id:1}};store.space.objects[1].value=Value::Integer {{number:42}};')
  for label,kw in [('empty',dict(object_count=0)),('past',dict(object=1)),('max',dict(object=MAX)),('count65',dict(object_count=65)),('count_max',dict(object_count=MAX))]:add(f'invalid_{fmt}_{label}','Integer',fmt,error='InvalidState',**kw)
 add('default','Default',error='InvalidState')
 assert len({r['name']for r in rows})==len(rows)
 return rows

def format_number(n):return f'{n:X}'
IMPORTS='''use aml::object_numeric_strings::convert;
use aml::object_numeric_strings::NumericStringResult;
use aml::object_conversions::ConversionFailure;
use aml::model::ObjectStore;
use aml::model::Value;
use aml::model::Span;
use aml::model::Path;
use aml::model::StringStorage;
use aml::model::BufferStorage;
use aml::model::ReferenceKind;
use integer_helpers::integers::IntegerSize;
use integer_helpers::string_numbers::NumberFormat;
'''
HELPERS='''machine ns_test_equal(a:&[u8;256],b:&[u8;256],index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let same:bool=ns_test_byte(a,b,index);transition index<count {true -> ns_test_equal(a,b,index+1,count,prior && same) _ -> (prior)}}
machine ns_test_byte(a:&[u8;256],b:&[u8;256],index:u64)->bool {transition index<256 {true -> (a[index]==b[index]) _ -> (true)}}
machine ns_test_failure(result:NumericStringResult,expected:ConversionFailure)->bool {transition result {NumericStringResult::Failure {reason} -> (reason==expected) _ -> (false)}}
machine ns_test_string(result:NumericStringResult,expected_length:u64,expected:&[u8;256])->bool {
 transition result {NumericStringResult::String {length,bytes} -> compare(length,bytes,expected_length,expected) _ -> (false)}
 state compare(length:u64,bytes:[u8;256],expected_length:u64,expected:&[u8;256])->bool {let same:bool=ns_test_equal(&bytes,expected,0,256,true);length==expected_length && same}
}
machine ns_test_fill(a:&mut[u8;256],value:u8,index:u64,count:u64)
terminates by(index,count)->Nat::BoundedDistance;
{transition index<count && index<256 {true -> put(a,value,index,count) _ -> {}}
 state put(a:&mut[u8;256],value:u8,index:u64,count:u64){a[index]=value;ns_test_fill(a,value,index+1,count);}}
machine ns_test_input(a:&mut[u8;1024],b:&[u8;256],index:u64,count:u64)
terminates by(index,count)->Nat::BoundedDistance;
{transition index<count && index<256 {true -> put(a,b,index,count) _ -> {}}
 state put(a:&mut[u8;1024],b:&[u8;256],index:u64,count:u64){a[index]=b[index];ns_test_input(a,b,index+1,count);}}
'''
def writes(name,data):
 if not data:return ''
 mode=max(set(data),key=data.count);s=f'ns_test_fill(&mut {name},{mode},0,{len(data)});'
 return s+''.join(f'{name}[{i}]={v};'for i,v in enumerate(data)if v!=mode)
def render(r,control=False,machine='test_result()'):
 data=r['data'];body='let mut input:[u8;1024];let mut store:ObjectStore=ObjectStore {};\n'+f'store.space.object_count={r["object_count"]};let mut raw:[u8;256];'+writes('raw',data)+f'ns_test_input(&mut input,&raw,0,{len(data)});'
 k=r['kind']
 if k in ['String','Buffer']:
  if r['owned']:
   val='String {string_storage:StringStorage::Owned {string_owner:0}}'if k=='String'else'Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:0}}'
   body+=f'store.bytes.blocks[0].initialized=true;store.bytes.blocks[0].length={len(data)};store.bytes.blocks[0].bytes=raw;'
  else:val=f'String {{string_storage:StringStorage::Source {{string_source:Span {{unit:7,end:{len(data)}}}}}}}'if k=='String'else f'Buffer {{buffer_storage:BufferStorage::Source {{declared_size:{r["declared"]},buffer_initializer:Span {{unit:7,end:{len(data)}}}}}}}'
  body+='store.space.objects[0].value=Value::'+val+';'
 elif k=='Integer':body+=f'store.space.objects[0].value=Value::Integer {{number:{r["number"]}}};'
 else:
  vals=dict(Uninitialized='Uninitialized',Package='Package {first:0,count:0}',NameReference='NameReference {name:Path {},scope:Path {}}',Method='Method {flags:0,body:Span {}}',OperationRegion='OperationRegion {space:0,base:0,length:0,scope:Path {}}',Device='Device',BufferField='BufferField {backing_object:0,bit_offset:0,bit_length:0}',Reference='Uninitialized',Default='Uninitialized')
  body+='store.space.objects[0].value=Value::'+vals[k]+';'
 body+=r['extra']+'\n'
 if k=='Default':body+='let result:NumericStringResult;'
 else:body+=f'let result:NumericStringResult=convert(&input,{r["source_length"]},{r["unit"]},&store,{r["object"]},IntegerSize::{"FourBytes"if r["bits"]==32 else"EightBytes"},NumberFormat::{r["format"]});\n'
 if r['error']:
  expected=('InvalidState'if r['error']!='InvalidState'else'UnsupportedValue')if control else r['error'];body+=f'let good:bool=ns_test_failure(result,ConversionFailure::{expected});'
 else:
  expected=r['expected'];body+='let mut expected:[u8;256];'+writes('expected',expected)
  if control:body+=f'expected[255]={(expected[255]if len(expected)==256 else 0)^1};'
  body+=f'let good:bool=ns_test_string(result,{len(expected)},&expected);'
 return f'machine {machine}->i32 {{\n'+body+'transition good {true -> (0) _ -> (1)}\n}\n'
def main():
 import argparse
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();text=json.dumps(cases(),indent=2)+'\n';path=HERE/'cases.json'
 if a.check:assert path.read_text()==text
 else:path.write_text(text)
 print(len(cases()),'numeric-string cases')
if __name__=='__main__':main()
