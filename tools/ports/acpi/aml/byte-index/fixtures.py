"""Canonical byte-index allocation transitions and complete store controls."""
from pathlib import Path
import store_checks
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4];MAX=(1<<64)-1

def cases():
 rows=[]
 def add(name,kind='Buffer',owned=False,size=3,index=0,error=None,**kw):
  r=dict(name=name,kind=kind,owned=owned,size=size,index=index,error=error,source=kw.pop('source',0),at=kw.pop('at',0),count=kw.pop('count',4),repeat=kw.pop('repeat',1),length=kw.pop('length',256),unit=kw.pop('unit',7),declared=kw.pop('declared',size),mutation=kw.pop('mutation',''),resolved=kw.pop('resolved',0));assert not kw;rows.append(r)
 for kind in ['Buffer','String']:
  for owned in [False,True]:
   for size in [0,1,8,255,256]:
    for index in sorted({0,max(0,size-1),size}):add(f'{kind}_{owned}_{size}_{index}',kind,owned,size,index,None if index<size else'Bounds')
   for index in [1<<63,MAX]:add(f'max_index_{kind}_{owned}_{index}',kind,owned,index=index,error='Bounds')
   add(f'late_encoding_{kind}_{owned}',kind,owned,size=256,error=None if kind=='Buffer'else'Encoding',mutation=('s.bytes.blocks[0].bytes[255]=0;'if owned else'input[255]=0;'))
   if owned:
    add('wrong_owner_'+kind,kind,True,error='InvalidState',mutation=f's.space.objects[0].value=Value::{kind} {{'+('buffer_storage:BufferStorage::Owned {buffer_owner:1}'if kind=='Buffer'else'string_storage:StringStorage::Owned {string_owner:1}')+'};')
    add('uninitialized_'+kind,kind,True,error='InvalidState',mutation='s.bytes.blocks[0].initialized=false;')
    for n in [257,1<<63,MAX]:add(f'owned_length_{kind}_{n}',kind,True,error='Capacity',mutation=f's.bytes.blocks[0].length={n};')
    add('owned_ignores_input_'+kind,kind,True,length=MAX,unit=MAX)
   else:
    add('wrong_unit_'+kind,kind,unit=8,error='Bounds')
    add('short_input_'+kind,kind,length=2,error='Bounds')
    add('max_input_'+kind,kind,length=MAX,error='Capacity')
 add('source_padding',size=1,index=255,declared=256)
 add('initializer_larger',size=8,index=7,declared=1)
 add('last_two_slots',count=62)
 add('source_last_allocated',count=62,source=61,at=61,owned=True)
 add('one_slot_partial_rollback',count=63,error='Capacity')
 add('full',count=64,error='Capacity')
 add('repeat',repeat=2)
 add('repeat_second_fails',count=62,repeat=2,error='Capacity')
 for n in [0,65,1<<63,MAX]:add('count_'+str(n),count=n,error='InvalidState')
 for n in [4,64,1<<63,MAX]:add('source_'+str(n),source=n,error='InvalidState')
 for n in [0,32,33,1<<63,MAX]:add('namespace_'+str(n),mutation=f's.space.count={n};',error=None)
 add('index_before_capacity',index=3,mutation=f's.space.count={MAX};',error='Bounds')
 add('encoding_before_index',kind='String',size=256,index=MAX,mutation='input[255]=0;',error='Encoding')
 add('encoding_before_capacity',kind='String',size=256,count=63,mutation='input[255]=0;',error='Encoding')
 for kind in ['Named','Local','Arg','RefOf','Index','Unresolved']:add('outer_'+kind,error='UnsupportedValue',mutation=f's.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:1}};')
 for kind,value in [('integer','Value::Integer {number:3}'),('field','Value::BufferField {backing_object:1,bit_length:8}'),('package','Value::Package {first:1,count:1}'),('uninitialized','Value::Uninitialized')]:add('unsupported_'+kind,error='UnsupportedValue',mutation=f's.space.objects[0].value={value};')
 for kind in ['Named','Local','Arg']:
  add('transparent_'+kind,resolved=1,mutation=f's.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:1}};s.space.objects[1].value=Value::Buffer {{buffer_storage:BufferStorage::Owned {{buffer_owner:1}}}};s.bytes.blocks[1]=ByteBlock {{initialized:true,length:3}};')
 add('transparent_cycle',error='ReferenceCycle',mutation='s.space.objects[0].value=Value::Reference {kind:ReferenceKind::Named,object_id:0};')
 add('transparent_dangling',error='InvalidState',mutation='s.space.objects[0].value=Value::Reference {kind:ReferenceKind::Arg,object_id:18446744073709551615};')
 assert len({r['name']for r in rows})==len(rows)
 return rows

IMPORTS='''use aml::byte_storage::make_byte_index;
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
HELPERS=store_checks.HELPERS+'''machine bi_fill(input:&mut [u8;1024],bytes:&mut [u8;256],i:u64,n:u64)
terminates by(i,n)->Nat::BoundedDistance;
{bi_fill_at(input,bytes,i,n);transition i<n {true -> bi_fill(input,bytes,i+1,n) _ -> {}}}
machine bi_fill_at(input:&mut [u8;1024],bytes:&mut [u8;256],i:u64,n:u64){transition i<n && i<256 {true -> put(input,bytes,i) _ -> {}} state put(input:&mut [u8;1024],bytes:&mut [u8;256],i:u64){input[i]=65;bytes[i]=65;}}
machine bi_result(result:ByteResult,expected:ByteResult)->bool {result.outcome==expected.outcome && result.object==expected.object && result.length==expected.length && result.value==expected.value}
machine bi_check(input:&[u8;1024],length:u64,unit:u64,store:ObjectStore,expected:ObjectStore,source:u64,index:u64,repeat:u64,wanted:ByteResult)->i32 {
 let mut actual:ObjectStore=store;let result:ByteResult=make_byte_index(input,length,unit,&mut actual,source,index);
 transition repeat==2 {true -> twice(input,length,unit,actual,expected,source,index,wanted,result,store.space.object_count) _ -> compare(actual,expected,result,wanted)}
 state twice(input:&[u8;1024],length:u64,unit:u64,store:ObjectStore,expected:ObjectStore,source:u64,index:u64,wanted:ByteResult,first:ByteResult,count:u64)->i32 {
  let first_ok:bool=bi_result(first,ByteResult {object:count+1,length:8});
  let mut actual:ObjectStore=store;let result:ByteResult=make_byte_index(input,length,unit,&mut actual,source,index);
  transition first_ok {true -> compare(actual,expected,result,wanted) _ -> (1)}
 }
 state compare(actual:ObjectStore,expected:ObjectStore,result:ByteResult,wanted:ByteResult)->i32 {
  let good:bool=bi_result(result,wanted);let e:bool=entries(&actual,&expected,0,32,true);let o:bool=objects(&actual,&expected,0,64,true);
  transition good && e && o && actual.space.count==expected.space.count && actual.space.object_count==expected.space.object_count {true -> (0) _ -> (1)}
 }
}
machine bi_const_check(input:&[u8;1024],length:u64,unit:u64,store:ObjectStore,expected:ObjectStore,source:u64,index:u64,repeat:u64,wanted:ByteResult)->i32 {
 let mut actual:ObjectStore=store;let result:ByteResult=make_byte_index(input,length,unit,&mut actual,source,index);
 let good:bool=bi_result(result,wanted);let e:bool=entries(&actual,&expected,0,32,true);let o:bool=const_slots(&actual,&expected,0,64,true);
 transition repeat==1 && good && e && o && actual.space.count==expected.space.count && actual.space.object_count==expected.space.object_count {true -> (0) _ -> (1)}
}
'''
def body(row,control=False):
 r=row;a=r['at'];kind=r['kind'];size=r['size']
 t=f'let mut s:ObjectStore=seed();s.space.object_count={r["count"]};let mut input:[u8;1024];let mut bytes:[u8;256];bi_fill(&mut input,&mut bytes,0,{size});'
 if r['owned']:
  payload=f'Value::{kind} {{'+('buffer_storage:BufferStorage::Owned {buffer_owner:'+str(a)+'}'if kind=='Buffer'else'string_storage:StringStorage::Owned {string_owner:'+str(a)+'}')+'}'
  t+=f's.bytes.blocks[{a}]=ByteBlock {{initialized:true,length:{size},bytes:bytes}};s.bytes.blocks[{a}].bytes[255]=65;'
 else:payload=f'Value::{kind} {{'+(f'buffer_storage:BufferStorage::Source {{declared_size:{r["declared"]},buffer_initializer:Span {{unit:7,start:0,end:{size}}}}}'if kind=='Buffer'else f'string_storage:StringStorage::Source {{string_source:Span {{unit:7,start:0,end:{size}}}}}')+'}'
 t+=f's.space.objects[{a}].value={payload};'+r['mutation']+'let mut expected:ObjectStore=s;'
 allocations=r['repeat']if r['error']is None else(1 if r['name']=='repeat_second_fails'else 0)
 for j in range(allocations):
  at=r['count']+2*j
  t+=f'expected.space.objects[{at}]=Object {{value:Value::BufferField {{backing_object:{r["resolved"]or r["source"]},bit_offset:{r["index"]*8},bit_length:8}}}};expected.space.objects[{at+1}]=Object {{value:Value::Reference {{kind:ReferenceKind::RefOf,object_id:{at}}}}};expected.space.object_count={at+2};'
 wanted=f'ByteResult {{outcome:ByteOutcome::{r["error"]}}}'if r['error']else f'ByteResult {{object:{r["count"]+2*allocations-1},length:8}}'
 if control:
  mode=sum(r['name'].encode())%8
  if mode==0:t+='expected.bytes.blocks[63].bytes[255]=expected.bytes.blocks[63].bytes[255]^1;'
  elif mode==1:t+='expected.space.objects[63].next=expected.space.objects[63].next^1;'
  elif mode==2:t+='expected.space.entries[31].path.segments[15]=0;'
  elif mode==3:t+='expected.space.object_count=expected.space.object_count^1;'
  elif mode>=4 and mode<=6 and allocations:
   at=r['count']+2*(allocations-1)
   t+=f'expected.space.objects[{at}].value=Value::BufferField {{backing_object:{(r["resolved"]or r["source"])^(1 if mode==4 else 0)},bit_offset:{r["index"]*8^(8 if mode==5 else 0)},bit_length:{9 if mode==6 else 8}}};'
  else:wanted='ByteResult {outcome:ByteOutcome::WorkLimit}'
 t+=f'let answer:i32=bi_check(&input,{r["length"]},{r["unit"]},s,expected,{r["source"]},{r["index"]},{r["repeat"]},{wanted});answer'
 return t
