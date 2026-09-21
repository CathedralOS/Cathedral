#!/usr/bin/env python3
import json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4];MAX=(1<<64)-1

def cases():
 rows=[]
 def add(name,kind,method='integer',data=b'',number=0,expected=None,error=None,bits=64,owned=False,rule='ExplicitTerminated',**kw):
  row=dict(name=name,kind=kind,method=method,data=list(data),number=number,expected=expected,error=error,bits=bits,owned=owned,rule=rule,extra=kw.pop('extra',''),source_length=kw.pop('source_length',len(data)),unit=kw.pop('unit',7),object=kw.pop('object',0),object_count=kw.pop('object_count',1),declared=kw.pop('declared',len(data)));assert not kw;rows.append(row)
 for bits in [32,64]:
  for n in [0,1,0x87654321,0x1234567887654321,MAX]:
   add(f'integer_{bits}_{n}','Integer',number=n,expected=n&((1<<bits)-1),bits=bits)
   add(f'integer_buffer_{bits}_{n}','Integer','buffer',number=n,expected=list((n&((1<<bits)-1)).to_bytes(bits//8,'little')),bits=bits)
  for length in [0,1,3,4,7,8,9,256]:
   data=bytes(i%256+1 if i%256<255 else 0 for i in range(length))
   for owned in [False,True]:add(f'buffer_integer_{bits}_{length}_{owned}','Buffer',data=data,expected=int.from_bytes(data[:bits//8],'little'),error='Empty'if length==0 else None,bits=bits,owned=owned)
 for length in [0,1,255,256]:
  data=bytes(i%256 for i in range(length))
  for owned in [False,True]:add(f'buffer_copy_{length}_{owned}','Buffer','buffer',data=data,expected=list(data),owned=owned)
 for text,expected,error in [('',0,'Empty'),('0',0,None),('42',42,None),('00042',42,None),('0xFF',255,None),('0X10',16,None),('ff',0,'Encoding'),('0x',0,'Encoding'),(' 42',0,'Encoding'),('42 ',0,'Encoding'),('+1',0,'Encoding'),('-1',0,'Encoding'),('12tail',0,'Encoding'),('0x1g',0,'Encoding'),('18446744073709551615',MAX,None),('18446744073709551616',0,'Overflow')]:
  for owned in [False,True]:add('string_integer_'+text.encode().hex()+'_'+str(owned),'String',data=text.encode(),expected=expected,error=error,owned=owned)
 for text,expected,error in [('4294967295',0xffffffff,None),('4294967296',0,'Overflow'),('0xFFFFFFFF',0xffffffff,None),('0x100000000',0,'Overflow')]:add('string_width32_'+text,'String',data=text.encode(),expected=expected,error=error,bits=32)
 for length in [0,1,255,256]:
  for rule in ['ExplicitTerminated','PinnedObjectBytes']:
   for owned in [False,True]:
    data=b'A'*length;out=data+(b'\0'if length and rule=='ExplicitTerminated'else b'')
    add(f'string_buffer_{length}_{rule}_{owned}','String','buffer',data=data,expected=list(out)if len(out)<=256 else None,error='Capacity'if len(out)>256 else None,owned=owned,rule=rule)
 for kind in ['String','Buffer']:
  for method in ['integer','buffer']:
   add(f'owned_dirty_tail_{kind}_{method}',kind,method,data=b'42',owned=True,expected=(42 if kind=='String'else 0x3234)if method=='integer'else list(b'42'+(b'\0'if kind=='String'else b'')),extra='store.bytes.blocks[0].bytes[2]=254;store.bytes.blocks[0].bytes[255]=253;')
   add(f'wrong_owner_{kind}_{method}',kind,method,data=b'42',owned=True,error='InvalidState',extra='store.space.objects[0].value=Value::'+kind+' {'+('string_storage:StringStorage::Owned {string_owner:1}'if kind=='String'else'buffer_storage:BufferStorage::Owned {buffer_owner:1}')+'};')
   add(f'owned_uninitialized_{kind}_{method}',kind,method,data=b'42',owned=True,error='InvalidState',extra='store.bytes.blocks[0].initialized=false;')
   add(f'owned_length_max_{kind}_{method}',kind,method,data=b'42',owned=True,error='Capacity',extra=f'store.bytes.blocks[0].length={MAX};')
   add(f'source_wrong_unit_{kind}_{method}',kind,method,data=b'42',unit=8,error='Bounds')
   add(f'source_length_max_{kind}_{method}',kind,method,data=b'42',source_length=MAX,error='Capacity')
 for data in [b'1\0',b'1\x80',b'12!\xff',b'\xff'+b'A'*255]:
  for method in ['integer','buffer']:add('bad_encoding_'+data.hex()[:8]+'_'+str(len(data))+'_'+method,'String',method,data=data,error='Encoding',owned=True)
 for method in ['integer','buffer']:
  for kind in ['Uninitialized','Package','NameReference','Method','OperationRegion','Device','BufferFieldUnsupported']:
   if kind=='BufferFieldUnsupported'and method=='integer':continue
   add(f'unsupported_{kind}_{method}',kind,method,error='UnsupportedValue')
  for ref in ['Named','RefOf','Local','Arg','Index','Unresolved']:
   add(f'reference_{ref}_{method}','Reference',method,error='UnsupportedValue',object_count=2,extra=f'store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{ref},object_id:1}};store.space.objects[1].value=Value::Integer {{number:42}};')
  for label,kw in [('empty',dict(object_count=0)),('past_count',dict(object=1)),('id_max',dict(object=MAX)),('count65',dict(object_count=65)),('count_max',dict(object_count=MAX))]:add('invalid_'+label+'_'+method,'Integer',method,number=42,error='InvalidState',**kw)
 for bits in [32,64]:
  for length in [1,4,8,31,32,33,63,64,65]:
   expected=(0xfffedcba9876543210>>3)&((1<<min(length,bits))-1)
   add(f'field_{bits}_{length}','Field',data=(0xfedcba9876543210).to_bytes(8,'little')+b'\xff',expected=expected,error='UnsupportedValue'if length>bits else None,bits=bits,object_count=2,extra=f'store.space.objects[0].value=Value::BufferField {{backing_object:1,bit_offset:3,bit_length:{length}}};')
 for label,extra,error in [('zero','bit_offset:0,bit_length:0','Bounds'),('offset_max',f'bit_offset:{MAX},bit_length:1','Bounds'),('length_max',f'bit_offset:0,bit_length:{MAX}','Bounds'),('outside','bit_offset:16,bit_length:1','Bounds')]:add('field_'+label,'Field',data=b'12',error=error,object_count=2,extra=f'store.space.objects[0].value=Value::BufferField {{backing_object:1,{extra}}};')
 add('field_string_backing','Field',data=b'A',expected=65,object_count=2,extra='store.space.objects[0].value=Value::BufferField {backing_object:1,bit_length:8};store.space.objects[1].value=Value::String {string_storage:StringStorage::Source {string_source:Span {unit:7,end:1}}};')
 add('field_backing_cycle','Field',data=b'A',error='ReferenceCycle',object_count=2,extra='store.space.objects[0].value=Value::BufferField {backing_object:1,bit_length:8};store.space.objects[1].value=Value::Reference {kind:ReferenceKind::Named,object_id:1};')
 add('buffer_source_padding','Buffer','buffer',data=b'AB',declared=4,expected=[65,66,0,0])
 add('buffer_source_initializer_larger','Buffer','buffer',data=b'ABCD',declared=1,expected=[65,66,67,68])
 add('owned_ignores_source','Buffer','buffer',data=b'A',owned=True,source_length=MAX,expected=[65])
 add('integer_ignores_source','Integer',number=42,source_length=MAX,expected=42)
 assert len({r['name']for r in rows})==len(rows)
 return rows
IMPORTS='''use aml::object_conversions::to_integer;
use aml::object_conversions::to_buffer;
use aml::object_conversions::ConversionResult;
use aml::object_conversions::ConversionFailure;
use aml::object_conversions::BufferRule;
use aml::model::ObjectStore;
use aml::model::Value;
use aml::model::Span;
use aml::model::Path;
use aml::model::StringStorage;
use aml::model::BufferStorage;
use aml::model::ReferenceKind;
use integer_helpers::integers::IntegerSize;
'''
HELPERS='''machine conversion_test_bytes(a:&[u8;256],b:&[u8;256],index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let equal:bool=conversion_test_byte(a,b,index);transition index<count {true -> conversion_test_bytes(a,b,index+1,count,prior && equal) _ -> (prior)}}
machine conversion_test_byte(a:&[u8;256],b:&[u8;256],index:u64)->bool {transition index<256 {true -> (a[index]==b[index]) _ -> (true)}}
machine conversion_test_failure(result:ConversionResult,expected:ConversionFailure)->bool {transition result {ConversionResult::Failure {reason} -> (reason==expected) _ -> (false)}}
machine conversion_test_integer(result:ConversionResult,expected:u64)->bool {transition result {ConversionResult::Integer {number} -> (number==expected) _ -> (false)}}
machine conversion_test_buffer(result:ConversionResult,expected_length:u64,expected:&[u8;256])->bool {
 transition result {ConversionResult::Bytes {length,bytes} -> compare(length,bytes,expected_length,expected) _ -> (false)}
 state compare(length:u64,bytes:[u8;256],expected_length:u64,expected:&[u8;256])->bool {let equal:bool=conversion_test_bytes(&bytes,expected,0,256,true);length==expected_length && equal}
}
'''
def render(r,control=False,machine='test_result()'):
 data=r['data'];body='let mut input:[u8;1024];let mut store:ObjectStore=ObjectStore {};\n'+f'store.space.object_count={r["object_count"]};'
 body+=''.join(f'input[{i}]={v};'for i,v in enumerate(data))
 kind=r['kind'];id=1 if kind=='Field'else 0
 if kind in ['String','Buffer','Field']:
  k='Buffer'if kind=='Field'else kind
  if r['owned']:
   val='String {string_storage:StringStorage::Owned {string_owner:0}}'if k=='String'else'Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:0}}'
   body+=f'store.bytes.blocks[0].initialized=true;store.bytes.blocks[0].length={len(data)};'+''.join(f'store.bytes.blocks[0].bytes[{i}]={v};'for i,v in enumerate(data))
  else:val=f'String {{string_storage:StringStorage::Source {{string_source:Span {{unit:7,end:{len(data)}}}}}}}'if k=='String'else f'Buffer {{buffer_storage:BufferStorage::Source {{declared_size:{r["declared"]},buffer_initializer:Span {{unit:7,end:{len(data)}}}}}}}'
  body+=f'store.space.objects[{id}].value=Value::{val};'
 elif kind=='Integer':body+=f'store.space.objects[0].value=Value::Integer {{number:{r["number"]}}};'
 else:
  vals=dict(Uninitialized='Uninitialized',Package='Package {first:0,count:0}',NameReference='NameReference {name:Path {},scope:Path {}}',Method='Method {flags:0,body:Span {}}',OperationRegion='OperationRegion {space:0,base:0,length:0,scope:Path {}}',Device='Device',BufferFieldUnsupported='BufferField {backing_object:0,bit_offset:0,bit_length:0}',Reference='Uninitialized')
  body+='store.space.objects[0].value=Value::'+vals[kind]+';'
 body+=r['extra']+'\n'
 args=f'&input,{r["source_length"]},{r["unit"]},&store,{r["object"]},IntegerSize::{"FourBytes"if r["bits"]==32 else"EightBytes"}'
 if r['method']=='buffer':args+=f',BufferRule::{r["rule"]}'
 body+=f'let result:ConversionResult=to_{r["method"]}({args});\n'
 if r['error']:
  error='InvalidState'if r['error']!='InvalidState'else'UnsupportedValue';expected=error if control else r['error']
  body+=f'let good:bool=conversion_test_failure(result,ConversionFailure::{expected});'
 elif r['method']=='integer':
  expected=r['expected']^int(control);body+=f'let good:bool=conversion_test_integer(result,{expected});'
 else:
  expected=r['expected'];body+='let mut expected:[u8;256];'+''.join(f'expected[{i}]={v};'for i,v in enumerate(expected))
  body+=f'let good:bool=conversion_test_buffer(result,{len(expected)+int(control)},&expected);'
 return f'machine {machine}->i32 {{\n'+body+'transition good {true -> (0) _ -> (1)}\n}\n'
def main():
 import argparse
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();text=json.dumps(cases(),indent=2)+'\n';path=HERE/'cases.json'
 if a.check:assert path.read_text()==text
 else:path.write_text(text)
 print(len(cases()),'conversion preflight cases')
if __name__=='__main__':main()
