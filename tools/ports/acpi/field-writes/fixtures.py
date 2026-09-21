#!/usr/bin/env python3
"""Evaluate write bodies and every initialized output record; mutate expected bodies."""
import json
from pathlib import Path
import vectors
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[3]
HEAD='''use writes::write;
use writes::model::WriteResult;
use writes::model::WriteError;
use writes::model::NativeWrite;
use access::model::Error;
use access::model::LockRequirement;
use access::model::Previous;
use fields::field_model::Field;
use fields::field_model::AccessResult;
use fields::field_model::AccessType;
use fields::field_model::UpdateRule;
use fields::field_model::DeclarationKind;
use fields::field_model::Connection;
use fields::flags;
use aml::model::Path;
use integers::integers::IntegerSize;
data DefaultResult [copy] {result:WriteResult;}
data Old [copy] {words:[Previous;257];}
data Bytes [copy] {bytes:[u8;256];}
data Records [copy] {records:[NativeWrite;257];}
machine poison(bytes:&mut [u8;256],index:u64,count:u64)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let wrote:bool=poison_at(bytes,index);transition index<count {true -> poison(bytes,index+1,count) _ -> (true)}}
machine poison_at(bytes:&mut [u8;256],index:u64)->bool {transition index<256 {true -> write(bytes,index) _ -> (true)} state write(bytes:&mut [u8;256],index:u64)->bool{bytes[index]=255;true}}
machine same_records(a:&[NativeWrite;257],b:&[NativeWrite;257],index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let same:bool=record_at(a,b,index);transition index<count {true -> same_records(a,b,index+1,count,prior && same) _ -> (prior)}}
machine record_at(a:&[NativeWrite;257],b:&[NativeWrite;257],index:u64)->bool {transition index<257 {true -> compare(a[index],b[index]) _ -> (true)} state compare(a:NativeWrite,b:NativeWrite)->bool {a.offset==b.offset && a.width==b.width && a.value==b.value}}
machine good_writes(result:WriteResult,expected:&[NativeWrite;257],expected_count:u64,expected_lock:LockRequirement)->bool {
 transition result {WriteResult::Writes {count,records,lock} -> compare(&records,count,lock,expected,expected_count,expected_lock) _ -> (false)}
 state compare(records:&[NativeWrite;257],count:u64,lock:LockRequirement,expected:&[NativeWrite;257],expected_count:u64,expected_lock:LockRequirement)->bool {let same:bool=same_records(records,expected,0,257,true);same && count==expected_count && lock==expected_lock}
}
machine geometry_failure(result:WriteResult,expected:Error)->bool {
 transition result {WriteResult::Failure {error} -> cause(error,expected) _ -> (false)}
 state cause(error:WriteError,expected:Error)->bool {transition error {WriteError::Geometry {cause} -> (cause==expected) _ -> (false)}}
}
machine count_failure(result:WriteResult)->bool {transition result {WriteResult::Failure {error} -> count(error) _ -> (false)} state count(error:WriteError)->bool {transition error {WriteError::CountMismatch -> (true) _ -> (false)}}}
machine payload_failure(result:WriteResult)->bool {transition result {WriteResult::Failure {error} -> payload(error) _ -> (false)} state payload(error:WriteError)->bool {transition error {WriteError::PayloadLength -> (true) _ -> (false)}}}
machine chunk_failure(result:WriteResult,expected_index:u64,expected_cause:Error)->bool {transition result {WriteResult::Failure {error} -> chunk(error,expected_index,expected_cause) _ -> (false)} state chunk(error:WriteError,expected_index:u64,expected_cause:Error)->bool {transition error {WriteError::Chunk {index,cause} -> (index==expected_index && cause==expected_cause) _ -> (false)}}}
'''
def cases():return vectors.cases()+[dict(name='default_failure',kind='default')]
def body(row,control=False):
 if row.get('kind')=='default':return 'let initial:DefaultResult=DefaultResult {};let good:bool=geometry_failure(initial.result,Error::'+('Overflow'if control else'InvalidFlags')+');transition good {true -> (0) _ -> (1)}\n'
 size='FourBytes'if row['size']==4 else'EightBytes';lock='UnmetGlobalLock'if row['flags']&16 else'NotRequested';expected=row['expected']
 setup=f'let access:AccessResult=flags::decode_flags({row["flags"]});let mut field:Field=Field {{bit_offset:{row["offset"]},bit_length:{row["length"]},access:access.access}};field.access.flags={row["flags"]};let mut kind:DeclarationKind=DeclarationKind::Field;'+row['patch']+'let mut input:Bytes=Bytes {};let poisoned:bool=poison(&mut input.bytes,0,256);let mut old:Old=Old {};'
 for index,value in enumerate(row['payload']):setup+=f'input.bytes[{index}]={value};'
 for index,value in enumerate(row['previous']):
  if value is not None:setup+=f'old.words[{index}]=Previous::Value {{word:{value}}};'
 setup+=f'let result:WriteResult=write::assemble(kind,&field,{row["region"]},IntegerSize::{size},&input.bytes,{row["payload_length"]},&old.words,{row["supplied"]});'
 if 'error'in expected:
  if control:check='chunk_failure(result,0,Error::InvalidChunk)'
  elif expected['error']=='Geometry':check=f'geometry_failure(result,Error::{expected["cause"]})'
  elif expected['error']=='Chunk':check=f'chunk_failure(result,{expected["index"]},Error::{expected["cause"]})'
  else:check=('count_failure'if expected['error']=='CountMismatch'else'payload_failure')+'(result)'
 else:
  setup+='let mut expected:Records=Records {};'
  for index,record in enumerate(expected['records']):setup+=f'expected.records[{index}]=NativeWrite {{offset:{record["offset"]},width:{record["width"]},value:{record["value"]}}};'
  if control:setup+=f'expected.records[256].value={(expected["records"][256]["value"]if len(expected["records"])==257 else 0)^1};'
  check=f'good_writes(result,&expected.records,{expected["count"]},LockRequirement::{lock})'
 return setup+'let good:bool='+check+';transition good {true -> (0) _ -> (1)}\n'
def render(rows):
 text=HEAD+'data Suite{}\n';selections=[]
 for row in rows:
  for control in [False,True]:
   name='Suite::'+row['name']+('_control'if control else'_positive');text+='machine '+name+'(&mut self)->i32 {\n'+body(row,control)+'}\n';selections.append(name+'='+str(int(control)))
 return text,selections
if __name__=='__main__':
 rows=cases();(HERE/'cases.json').write_text(json.dumps(rows,indent=2,sort_keys=True)+'\n');print(len(rows),'write-assembly pairs')
