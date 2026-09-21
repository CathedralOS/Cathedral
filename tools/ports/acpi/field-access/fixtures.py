#!/usr/bin/env python3
"""Actual Omega plan/chunk fixtures; complete initialized arrays are compared."""
import json
from pathlib import Path
import vectors
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[3]
HEAD='''use access::geometry;
use access::chunks;
use access::model::Plan;
use access::model::PlanResult;
use access::model::Chunk;
use access::model::ReadShape;
use access::model::LockRequirement;
use access::model::Previous;
use access::model::WordResult;
use access::model::Error;
use fields::field_model::Field;
use fields::field_model::AccessResult;
use fields::field_model::AccessType;
use fields::field_model::UpdateRule;
use fields::field_model::DeclarationKind;
use fields::field_model::Connection;
use fields::flags;
use aml::model::Path;
use integers::integers::IntegerSize;
machine same_chunk(a:Chunk,b:Chunk)->bool {a.offset==b.offset && a.width==b.width && a.field_bit==b.field_bit && a.native_bit==b.native_bit && a.bit_count==b.bit_count && a.mask==b.mask && a.partial==b.partial}
machine same_chunks(a:&[Chunk;257],b:&[Chunk;257],index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let good:bool=chunk_at(a,b,index);transition index<count {true -> same_chunks(a,b,index+1,count,prior && good) _ -> (prior)}}
machine chunk_at(a:&[Chunk;257],b:&[Chunk;257],index:u64)->bool {transition index<257 {true -> compare(a[index],b[index]) _ -> (true)} state compare(a:Chunk,b:Chunk)->bool {let same:bool=same_chunk(a,b);same}}
machine same_shape(a:ReadShape,b:ReadShape)->bool {
 transition a {ReadShape::Integer {size} -> integer(size,b) ReadShape::Buffer {bytes} -> buffer(bytes,b)}
 state integer(expected:IntegerSize,other:ReadShape)->bool {transition other {ReadShape::Integer {size} -> (size==expected) _ -> (false)}}
 state buffer(expected:u64,other:ReadShape)->bool {transition other {ReadShape::Buffer {bytes} -> (bytes==expected) _ -> (false)}}
}
machine good_plan(result:PlanResult,expected:&Plan)->bool {
 transition result {PlanResult::Ready {plan} -> compare(&plan,expected) _ -> (false)}
 state compare(plan:&Plan,expected:&Plan)->bool {
  let rows:bool=same_chunks(&plan.chunks,&expected.chunks,0,257,true);let shape:bool=same_shape(plan.shape,expected.shape);
  plan.access==expected.access && plan.update==expected.update && plan.lock==expected.lock && shape && plan.bit_offset==expected.bit_offset && plan.bit_length==expected.bit_length && plan.region_bytes==expected.region_bytes && plan.start==expected.start && plan.end==expected.end && plan.width==expected.width && plan.count==expected.count && plan.preserve_reads==expected.preserve_reads && rows
 }
}
machine bad_plan(result:PlanResult,expected:Error)->bool {transition result {PlanResult::Failure {error} -> (error==expected) _ -> (false)}}
machine good_word(result:WordResult,expected:u64)->bool {transition result {WordResult::Word {value} -> (value==expected) _ -> (false)}}
machine bad_word(result:WordResult,expected:Error)->bool {transition result {WordResult::Failure {error} -> (error==expected) _ -> (false)}}
'''
def chunk_text(chunk):return 'Chunk {'+','.join(key+':'+(str(value).lower()if isinstance(value,bool)else str(value))for key,value in chunk.items())+'}'
def cases():
 rows=[dict(row,kind='plan')for row in vectors.cases()]
 for width in [1,2,4,8]:
  bits=width*8
  for start,count in sorted({(0,bits),(0,1),(bits-1,1),(3,min(5,bits-3))}):
   chunk=dict(offset=0,width=width,field_bit=0,native_bit=start,bit_count=count,mask=((1<<count)-1)<<start,partial=count!=bits)
   for raw in [vectors.MAX,0x123456789abcdef0]:rows.append(dict(name=f'extract_{width}_{start}_{count}_{raw}',kind='extract',chunk=chunk,raw=raw,expected=(raw>>start)&((1<<count)-1)))
   for rule in range(3):
    for old in [None,0xffff0000aa55aa55]:
     for value in [0,vectors.MAX]:
      expected=dict(error='NeedsPrevious')if rule==0 and chunk['partial'] and old is None else dict(value=((((old or 0)if rule==0 else vectors.MAX if rule==1 else 0)&~chunk['mask'])|((value&((1<<count)-1))<<start))&((1<<bits)-1))
      rows.append(dict(name=f'merge_{width}_{start}_{count}_{rule}_{int(old is not None)}_{int(value!=0)}',kind='merge',chunk=chunk,rule=rule,old=old,value=value,expected=expected))
 valid=dict(offset=0,width=2,field_bit=0,native_bit=3,bit_count=11,mask=((1<<11)-1)<<3,partial=True)
 for field,values in [('width',[0,3,9,1<<63,vectors.MAX]),('offset',[1,vectors.MAX]),('field_bit',[2048,1<<63,vectors.MAX]),('native_bit',[16,1<<63,vectors.MAX]),('bit_count',[0,14,1<<63,vectors.MAX]),('mask',[0,vectors.MAX]),('partial',[False])]:
  for value in values:
   bad=dict(valid);bad[field]=value;rows.append(dict(name='invalid_'+field+'_'+str(value),kind='bad_chunk',chunk=bad))
 for value in [0,1,8,32,63,64,65,1<<63,vectors.MAX]:rows.append(dict(name='mask_'+str(value),kind='mask',bits=value,expected=(1<<value)-1 if value<=64 else 0))
 return rows

def body(row,control=False):
 kind=row['kind']
 if kind=='plan':
  size='FourBytes'if row['size']==4 else'EightBytes';setup=f'let access:AccessResult=flags::decode_flags({row["flags"]});let mut field:Field=Field {{bit_offset:{row["offset"]},bit_length:{row["length"]},access:access.access}};field.access.flags={row["flags"]};let mut kind:DeclarationKind=DeclarationKind::Field;'+row['patch']+f'let result:PlanResult=geometry::plan(kind,&field,{row["region"]},IntegerSize::{size});'
  expected=row['expected']
  if 'error'in expected:check='bad_plan(result,Error::'+('InvalidChunk'if control else expected['error'])+')'
  else:
   access=['Any','Byte','Word','DWord','QWord'][row['flags']&15];rule=['Preserve','WriteAsOnes','WriteAsZeros'][(row['flags']>>5)&3];lock='UnmetGlobalLock'if row['flags']&16 else'NotRequested';shape=f'ReadShape::Integer {{size:IntegerSize::{size}}}'if expected['shape']=='Integer'else f'ReadShape::Buffer {{bytes:{expected["bytes"]}}}'
   setup+=f'let mut expected:Plan=Plan {{access:AccessType::{access},update:UpdateRule::{rule},lock:LockRequirement::{lock},shape:{shape},bit_offset:{row["offset"]},bit_length:{row["length"]},region_bytes:{row["region"]},start:{expected["start"]},end:{expected["end"]},width:{expected["width"]},count:{expected["count"]},preserve_reads:{expected["preserve_reads"]}}};'
   for index,chunk in enumerate(expected['chunks']):setup+=f'expected.chunks[{index}]='+chunk_text(chunk)+';'
   # Slot 256 mutation witnesses zero-tail comparison (or final live chunk at capacity).
   if control:setup+=f'expected.chunks[256].mask={(expected["chunks"][256]["mask"]if expected["count"]==257 else 0)^1};'
   check='good_plan(result,&expected)'
 elif kind in ['extract','merge','bad_chunk']:
  setup='let chunk:Chunk='+chunk_text(row['chunk'])+';'
  if kind=='extract':setup+=f'let result:WordResult=chunks::extract(chunk,{row["raw"]});';check=f'good_word(result,{row["expected"]^int(control)})'
  elif kind=='bad_chunk':setup+='let a:WordResult=chunks::extract(chunk,0);let b:WordResult=chunks::merge(chunk,UpdateRule::Preserve,Previous::Absent,0);';check='bad_word(a,Error::'+('NeedsPrevious'if control else'InvalidChunk')+') && bad_word(b,Error::InvalidChunk)'
  else:
   rule=['Preserve','WriteAsOnes','WriteAsZeros'][row['rule']];old='Previous::Absent'if row['old']is None else f'Previous::Value {{word:{row["old"]}}}';setup+=f'let result:WordResult=chunks::merge(chunk,UpdateRule::{rule},{old},{row["value"]});'
   check='bad_word(result,Error::'+('InvalidChunk'if control else row['expected']['error'])+')'if 'error'in row['expected']else f'good_word(result,{row["expected"]["value"]^int(control)})'
 else:setup=f'let value:u64=chunks::low_mask({row["bits"]});';check=f'value=={row["expected"]^int(control)}'
 return setup+'\nlet good:bool='+check+';transition good {true -> (0) _ -> (1)}\n'
def render(rows):
 text=HEAD+'data Suite {}\n';selections=[]
 for row in rows:
  for control in [False,True]:
   name='Suite::'+row['name']+('_control'if control else'_positive');text+='machine '+name+'(&mut self)->i32 {\n'+body(row,control)+'}\n';selections.append(name+'='+str(int(control)))
 return text,selections
if __name__=='__main__':
 rows=cases();(HERE/'cases.json').write_text(json.dumps(rows,indent=2,sort_keys=True)+'\n');print(len(rows),'field-access pairs')
