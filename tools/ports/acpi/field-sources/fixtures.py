#!/usr/bin/env python3
import json
from pathlib import Path
import vectors
from fixture_common import HELPERS,initialized
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[3]
HEAD='''use sources::sequence::payload_at;
use sources::model::PayloadResult;
use aml::object_conversions::ConversionFailure;
use aml::model::ObjectStore;use aml::model::Value;use aml::model::Span;use aml::model::Path;
use aml::model::StringStorage;use aml::model::BufferStorage;use aml::model::ReferenceKind;
use integers::integers::IntegerSize;
data DefaultResult [copy] {value:PayloadResult;}
'''+HELPERS+'''
machine expected_failure(value:PayloadResult,error:ConversionFailure)->bool {transition value {PayloadResult::Failure {reason} -> (reason==error) _ -> (false)}}
machine expected_end(value:PayloadResult,count:u64)->bool {transition value {PayloadResult::End {total} -> (total==count) _ -> (false)}}
machine expected_payload(value:PayloadResult,expected_count:u64,expected_ordinal:u64,expected_length:u64,expected:&[u8;256])->bool {
 transition value {PayloadResult::Payload {total,ordinal,length,bytes} -> compare(&bytes,expected,total,expected_count,ordinal,expected_ordinal,length,expected_length) _ -> (false)}
 state compare(actual:&[u8;256],expected:&[u8;256],total:u64,expected_count:u64,ordinal:u64,expected_ordinal:u64,length:u64,expected_length:u64)->bool {let same:bool=equal_bytes(actual,expected,0,256,true);same && total==expected_count && ordinal==expected_ordinal && length==expected_length}
}
'''
def cases():return vectors.cases()+[dict(name='default_failure',kind='default')]
def body(r,control=False):
 if r['kind']=='default':return 'let initial:DefaultResult=DefaultResult {};let good:bool=expected_failure(initial.value,ConversionFailure::'+('Bounds'if control else'InvalidState')+');transition good {true -> (0) _ -> (1)}'
 data=r['data'];kind=r['kind'];code='let mut input:[u8;1024];let mut store:ObjectStore=ObjectStore {};'+initialized('data',data)+f'_=put_input(&mut input,&data,0,{len(data)});store.space.object_count={r["object_count"]};'
 if kind in ['String','Buffer']:
  if r['owned']:
   value='String {string_storage:StringStorage::Owned {string_owner:0}}'if kind=='String'else'Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:0}}';code+=f'store.bytes.blocks[0].initialized=true;store.bytes.blocks[0].length={len(data)};store.bytes.blocks[0].bytes=data;'
  else:value=f'String {{string_storage:StringStorage::Source {{string_source:Span {{unit:7,end:{len(data)}}}}}}}'if kind=='String'else f'Buffer {{buffer_storage:BufferStorage::Source {{declared_size:{r["declared"]},buffer_initializer:Span {{unit:7,end:{len(data)}}}}}}}'
 elif kind=='Integer':value=f'Integer {{number:{r["number"]}}}'
 else:value={'Uninitialized':'Uninitialized','Package':'Package {first:0,count:0}','Method':'Method {flags:0,body:Span {}}','OperationRegion':'OperationRegion {space:0,base:0,length:0,scope:Path {}}','NameReference':'NameReference {name:Path {},scope:Path {}}','BufferField':'BufferField {backing_object:0,bit_offset:0,bit_length:0}','Device':'Device','Reference':'Reference {kind:ReferenceKind::Named,object_id:0}'}[kind]
 code+='store.space.objects[0].value=Value::'+value+';'+r['extra'];size='FourBytes'if r['bits']==32 else'EightBytes'
 code+=f'let value:PayloadResult=payload_at(&input,{r["source_length"]},{r["unit"]},&store,{r["object"]},IntegerSize::{size},{r["width"]},{r["ordinal"]});';expected=r['expected']
 if 'error'in expected:
  error=('Bounds'if expected['error']=='InvalidState'else'InvalidState')if control else expected['error'];code+=f'let good:bool=expected_failure(value,ConversionFailure::{error});'
 elif expected.get('end'):code+=f'let good:bool=expected_end(value,{expected["total"]^int(control)});'
 else:
  out=list(expected['bytes']);out[-1]^=int(control);code+=initialized('expected',out)+f'let good:bool=expected_payload(value,{expected["total"]},{expected["ordinal"]},{expected["length"]},&expected);'
 return code+'transition good {true -> (0) _ -> (1)}'
def render(rows):
 text=HEAD+'data Suite{}\n';selections=[]
 for row in rows:
  for control in [False,True]:
   name='Suite::'+row['name']+('_control'if control else'_positive');text+='machine '+name+'(&mut self)->i32{'+body(row,control)+'}\n';selections.append(name+'='+str(int(control)))
 return text,selections
if __name__=='__main__':
 rows=cases();(HERE/'cases.json').write_text(json.dumps(rows,indent=2,sort_keys=True)+'\n');print(len(rows),'payload sequencing pairs')
