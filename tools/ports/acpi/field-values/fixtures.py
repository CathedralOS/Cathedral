#!/usr/bin/env python3
"""Actual assembly body fixtures; every output byte and semantic alternative checked."""
import json
from pathlib import Path
import vectors
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[3]
HEAD='''use values::read;
use values::model::ReadResult;
use values::model::ReadError;
use access::model::Error;
use access::model::LockRequirement;
use aml::field_model::Field;
use aml::field_model::AccessResult;
use aml::field_model::AccessType;
use aml::field_model::UpdateRule;
use aml::field_model::DeclarationKind;
use aml::field_model::Connection;
use fields::flags;
use aml::model::Path;
use integers::integers::IntegerSize;
data DefaultResult [copy] {result:ReadResult;}
data Words [copy] {words:[u64;257];}
data Bytes [copy] {bytes:[u8;256];}
machine poison(words:&mut [u64;257],index:u64,count:u64)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let wrote:bool=poison_at(words,index);transition index<count {true -> poison(words,index+1,count) _ -> (true)}}
machine poison_at(words:&mut [u64;257],index:u64)->bool {transition index<257 {true -> write(words,index) _ -> (true)} state write(words:&mut [u64;257],index:u64)->bool{words[index]=18446744073709551615;true}}
machine same_bytes(a:&[u8;256],b:&[u8;256],index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let same:bool=byte_at(a,b,index);transition index<count {true -> same_bytes(a,b,index+1,count,prior && same) _ -> (prior)}}
machine byte_at(a:&[u8;256],b:&[u8;256],index:u64)->bool {transition index<256 {true -> (a[index]==b[index]) _ -> (true)}}
machine good_integer(result:ReadResult,expected:u64,expected_size:IntegerSize,expected_lock:LockRequirement)->bool {
 transition result {ReadResult::Integer {value,size,lock} -> (value==expected && size==expected_size && lock==expected_lock) _ -> (false)}
}
machine good_buffer(result:ReadResult,expected:&[u8;256],expected_length:u64,expected_lock:LockRequirement)->bool {
 transition result {ReadResult::Buffer {length,bytes,lock} -> compare(&bytes,length,lock,expected,expected_length,expected_lock) _ -> (false)}
 state compare(bytes:&[u8;256],length:u64,lock:LockRequirement,expected:&[u8;256],expected_length:u64,expected_lock:LockRequirement)->bool {let same:bool=same_bytes(bytes,expected,0,256,true);same && length==expected_length && lock==expected_lock}
}
machine geometry_failure(result:ReadResult,expected:Error)->bool {
 transition result {ReadResult::Failure {error} -> cause(error,expected) _ -> (false)}
 state cause(error:ReadError,expected:Error)->bool {transition error {ReadError::Geometry {cause} -> (cause==expected) _ -> (false)}}
}
machine count_failure(result:ReadResult)->bool {transition result {ReadResult::Failure {error} -> count(error) _ -> (false)} state count(error:ReadError)->bool {transition error {ReadError::CountMismatch -> (true) _ -> (false)}}}
machine internal_failure(result:ReadResult)->bool {transition result {ReadResult::Failure {error} -> internal(error) _ -> (false)} state internal(error:ReadError)->bool {transition error {ReadError::InternalChunk -> (true) _ -> (false)}}}
'''
def cases():return vectors.cases()+[dict(name="default_failure",kind="default")]
def body(row,control=False):
 if row.get("kind")=="default":return "let initial:DefaultResult=DefaultResult {};let good:bool=geometry_failure(initial.result,Error::"+("Overflow"if control else"InvalidFlags")+");transition good {true -> (0) _ -> (1)}\n"
 size='FourBytes'if row['size']==4 else'EightBytes';lock='UnmetGlobalLock'if row['flags']&16 else'NotRequested';expected=row['expected']
 setup=f'let access:AccessResult=flags::decode_flags({row["flags"]});let mut field:Field=Field {{bit_offset:{row["offset"]},bit_length:{row["length"]},access:access.access}};field.access.flags={row["flags"]};let mut kind:DeclarationKind=DeclarationKind::Field;'+row['patch']+'let mut input:Words=Words {};let poisoned:bool=poison(&mut input.words,0,257);'
 for index,value in enumerate(row['words']):setup+=f'input.words[{index}]={value};'
 setup+=f'let result:ReadResult=read::assemble(kind,&field,{row["region"]},IntegerSize::{size},&input.words,{row["supplied"]});'
 if 'error'in expected:check='internal_failure(result)'if control else'count_failure(result)'if expected['error']=='CountMismatch'else f'geometry_failure(result,Error::{expected["cause"]})'
 elif expected['shape']=='Integer':check=f'good_integer(result,{expected["value"]^int(control)},IntegerSize::{size},LockRequirement::{lock})'
 else:
  setup+='let mut expected:Bytes=Bytes {};'
  for index,value in enumerate(expected['bytes']):
   if value:setup+=f'expected.bytes[{index}]={value};'
  if control:setup+=f'expected.bytes[255]={expected["bytes"][255]^1};'
  check=f'good_buffer(result,&expected.bytes,{expected["length"]},LockRequirement::{lock})'
 return setup+'let good:bool='+check+';transition good {true -> (0) _ -> (1)}\n'
def render(rows):
 text=HEAD+'data Suite{}\n';selections=[]
 for row in rows:
  for control in [False,True]:
   name='Suite::'+row['name']+('_control'if control else'_positive');text+='machine '+name+'(&mut self)->i32 {\n'+body(row,control)+'}\n';selections.append(name+'='+str(int(control)))
 return text,selections
if __name__=='__main__':
 rows=cases();(HERE/'cases.json').write_text(json.dumps(rows,indent=2,sort_keys=True)+'\n');print(len(rows),'read-assembly pairs')
