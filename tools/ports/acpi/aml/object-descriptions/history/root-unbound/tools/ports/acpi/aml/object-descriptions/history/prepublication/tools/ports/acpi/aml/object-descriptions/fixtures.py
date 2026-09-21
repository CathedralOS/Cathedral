#!/usr/bin/env python3
import json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4];MAX=(1<<64)-1
LABELS={'Uninitialized':'[Uninitialized Object]','Package':'[Package]','BufferField':'[Buffer Field]','Device':'[Device]','Event':'[Event]','Method':'[Control Method]','Mutex':'[Mutex]','OperationRegion':'[Operation Region]','PowerResource':'[Power Resource]','Processor':'[Processor]','ThermalZone':'[Thermal Zone]'}
def cases():
 rows=[]
 def add(name,kind,slot=0,poison=False,error=None,**kw):rows.append(dict(name=name,kind=kind,slot=slot,poison=poison,error=error,object=kw.get('object',slot),count=kw.get('count',slot+1),expected=LABELS.get(kind),extra=kw.get('extra','')))
 for k in LABELS:
  for slot in [0,31,63]:
   for poison in [False,True]:add(f'{k}_{slot}_{poison}',k,slot,poison)
 for k in ['Integer','StringSource','StringOwned','BufferSource','BufferOwned','NameReference']:
  for poison in [False,True]:add(f'unsupported_{k}_{poison}',k,63,poison,'UnsupportedValue')
 for ref in ['Named','RefOf','Local','Arg','Index','Unresolved']:
  for target in [0,1,MAX]:add(f'reference_{ref}_{target}','Reference',0,False,'UnsupportedValue',count=2,extra=f'store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{ref},object_id:{target}}};store.space.objects[1].value=Value::Device;')
 for k in ['Uninitialized','Device','StringOwned']:
  for label,kw in [('empty',dict(count=0)),('past',dict(count=1,object=1)),('id64',dict(count=64,object=64)),('id_max',dict(count=64,object=MAX)),('count65',dict(count=65)),('count_max',dict(count=MAX))]:add(f'invalid_{k}_{label}',k,error='InvalidState',**kw)
 add('default','Default',error='InvalidState')
 assert len({r['name']for r in rows})==len(rows)
 return rows
IMPORTS='''use aml::object_descriptions::describe;
use aml::object_descriptions::Description;
use aml::object_conversions::ConversionFailure;
use aml::model::ObjectStore;
use aml::model::Value;
use aml::model::Span;
use aml::model::Path;
use aml::model::StringStorage;
use aml::model::BufferStorage;
use aml::model::ReferenceKind;
'''
HELPERS='''machine od_test_bytes(a:&[u8;256],b:&[u8;256],index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let equal:bool=od_test_byte(a,b,index);transition index<count {true -> od_test_bytes(a,b,index+1,count,prior && equal) _ -> (prior)}}
machine od_test_byte(a:&[u8;256],b:&[u8;256],index:u64)->bool {transition index<256 {true -> (a[index]==b[index]) _ -> (true)}}
machine od_test_failure(result:Description,expected:ConversionFailure)->bool {transition result {Description::Failure {reason} -> (reason==expected) _ -> (false)}}
machine od_test_string(result:Description,expected_length:u64,expected:&[u8;256])->bool {
 transition result {Description::String {length,bytes} -> compare(length,bytes,expected_length,expected) _ -> (false)}
 state compare(length:u64,bytes:[u8;256],expected_length:u64,expected:&[u8;256])->bool {let equal:bool=od_test_bytes(&bytes,expected,0,256,true);length==expected_length && equal}
}
'''
def render(r,control=False,machine='test_result()'):
 poison=r['poison'];m=MAX if poison else 0;p8=255 if poison else 0;p16=65535 if poison else 0;p32=4294967295 if poison else 0
 span=f'Span {{unit:{m},start:{m},end:0}}';path=f'Path {{absolute:true,parents:{m},count:{m}}}'
 vals=dict(Uninitialized='Uninitialized',Package=f'Package {{first:{m},count:{m}}}',BufferField=f'BufferField {{backing_object:{m},bit_offset:{m},bit_length:{m}}}',Device='Device',Event='Event',Method=f'Method {{flags:{p8},body:{span}}}',Mutex=f'Mutex {{sync_level:{p8}}}',OperationRegion=f'OperationRegion {{space:{p8},base:{m},length:{m},scope:{path}}}',PowerResource=f'PowerResource {{system_level:{p8},order:{p16}}}',Processor=f'Processor {{id:{p8},address:{p32},length:{p8}}}',ThermalZone='ThermalZone',Integer=f'Integer {{number:{m}}}',StringSource=f'String {{string_storage:StringStorage::Source {{string_source:{span}}}}}',StringOwned=f'String {{string_storage:StringStorage::Owned {{string_owner:{m}}}}}',BufferSource=f'Buffer {{buffer_storage:BufferStorage::Source {{declared_size:{m},buffer_initializer:{span}}}}}',BufferOwned=f'Buffer {{buffer_storage:BufferStorage::Owned {{buffer_owner:{m}}}}}',NameReference=f'NameReference {{name:{path},scope:{path}}}',Reference='Uninitialized',Default='Uninitialized')
 slot=r['slot'];body=f'let mut store:ObjectStore=ObjectStore {{}};store.space.object_count={r["count"]};store.space.objects[{slot}].value=Value::{vals[r["kind"]]};'
 if poison:body+=f'store.space.count={MAX};store.space.objects[{slot}].has_next=true;store.space.objects[{slot}].next={MAX};store.bytes.blocks[{slot}].length={MAX};store.bytes.blocks[{slot}].initialized=false;store.bytes.blocks[{slot}].bytes[0]=255;store.bytes.blocks[{slot}].bytes[255]=254;'
 body+=r['extra']
 if r['kind']=='Default':body+='let result:Description;'
 else:body+=f'let result:Description=describe(&store,{r["object"]});'
 if r['error']:
  expected=('InvalidState'if r['error']!='InvalidState'else'UnsupportedValue')if control else r['error'];body+=f'let good:bool=od_test_failure(result,ConversionFailure::{expected});'
 else:
  expected=r['expected'].encode();body+='let mut expected:[u8;256];'+''.join(f'expected[{i}]={b};'for i,b in enumerate(expected))
  if control:body+='expected[255]=1;'
  body+=f'let good:bool=od_test_string(result,{len(expected)},&expected);'
 return f'machine {machine}->i32 {{\n'+body+'transition good {true -> (0) _ -> (1)}\n}\n'
def main():
 import argparse
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();s=json.dumps(cases(),indent=2)+'\n';p=HERE/'cases.json'
 if a.check:assert p.read_text()==s
 else:p.write_text(s)
 print(len(cases()),'object description cases')
if __name__=='__main__':main()
