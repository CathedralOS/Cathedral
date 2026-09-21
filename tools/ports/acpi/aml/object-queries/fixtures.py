#!/usr/bin/env python3
"""Original bounded mixed-reference and canonical object-query cases."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4];MAX=2**64-1
IMPORTS='''use aml::namespace;
use aml::model::Lookup;
use aml::model::Entry;
use aml::model::Outcome;
use aml::model::Value;
use aml::model::Namespace;
use aml::model::ObjectStore;
use aml::model::ReferenceKind;
use aml::model::ReferenceState;
use aml::model::Path;
use aml::model::Span;
use aml::model::StringStorage;
use aml::model::BufferStorage;
use aml::model::ByteBlock;
use aml::byte_storage::ByteOutcome;
use aml::object_queries::QueryResult;
use aml::object_queries::resolve_object;
use aml::object_queries::object_type;
use aml::object_queries::size_of;
'''
HELPERS='''machine query_equal(left:QueryResult,right:QueryResult)->bool {
 transition left {
  QueryResult::Value {object,number} -> value(right,object,number)
  QueryResult::ResolutionFailure {resolution_error} -> resolution(right,resolution_error)
  QueryResult::UnsupportedValue -> unsupported(right)
  QueryResult::InvalidStorage {storage_error} -> storage(right,storage_error)
 }
 state value(right:QueryResult,object:u64,number:u64)->bool {transition right {QueryResult::Value {object,number} -> same(right,object,number) _ -> (false)}}
 state same(right:QueryResult,expected_object:u64,expected_number:u64)->bool {transition right {QueryResult::Value {object,number} -> (object==expected_object && number==expected_number) _ -> (false)}}
 state resolution(right:QueryResult,error:Outcome)->bool {transition right {QueryResult::ResolutionFailure {resolution_error} -> (resolution_error==error) _ -> (false)}}
 state unsupported(right:QueryResult)->bool {transition right {QueryResult::UnsupportedValue -> (true) _ -> (false)}}
 state storage(right:QueryResult,error:ByteOutcome)->bool {transition right {QueryResult::InvalidStorage {storage_error} -> (storage_error==error) _ -> (false)}}
}
'''
# Keep left payload fields distinct from pattern bindings in right.
HELPERS=HELPERS.replace('state value(right:QueryResult,object:u64,number:u64)->bool {transition right {QueryResult::Value {object,number} -> same(right,object,number) _ -> (false)}}','state value(right:QueryResult,object:u64,number:u64)->bool {let equal:bool=query_numbers(right,object,number);equal}')
HELPERS+='''machine query_numbers(right:QueryResult,expected_object:u64,expected_number:u64)->bool {transition right {QueryResult::Value {object,number} -> (object==expected_object && number==expected_number) _ -> (false)}}\n'''
def seed(count=1):return f'let mut store:ObjectStore=ObjectStore {{}};store.space.object_count={count};let mut input:[u8;1024];\n'
def integer(i,n=42):return f'store.space.objects[{i}].value=Value::Integer {{number:{n}}};\n'
def ref(i,j,kind='Named'):return f'store.space.objects[{i}].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:{j}}};\n'
def bind(slot,target,segment=0x5f5f5f58):return f'store.space.entries[{slot}].path=Path {{absolute:true,count:1}};store.space.entries[{slot}].path.segments[0]={segment};store.space.entries[{slot}].has_object=true;store.space.entries[{slot}].object={target};\n'
def lexical(i,segment=0x5f5f5f58):return f'let mut path{i}:Path=Path {{absolute:true,count:1}};path{i}.segments[0]={segment};store.space.objects[{i}].value=Value::NameReference {{name:path{i},scope:Path {{absolute:true}}}};\n'
def source(kind='Buffer',declared=4,end=2,unit=7):
 return f'Value::Buffer {{buffer_storage:BufferStorage::Source {{declared_size:{declared},buffer_initializer:Span {{unit:{unit},end:{end}}}}}}}' if kind=='Buffer' else f'Value::String {{string_storage:StringStorage::Source {{string_source:Span {{unit:{unit},end:{end}}}}}}}'
def cases():
 rows=[]
 def resolve(label,body,start=0,budget=64,outcome='Success',object=0,seen=1):
  rows.append(dict(name='resolve_'+label,kind='resolve',body=body,call=f'resolve_object(&store.space,{start},{budget})',outcome=outcome,object=object,seen=seen))
 def query(label,body,call,number=None,object=0,error=None,storage=None):
  expected=f'QueryResult::Value {{object:{object},number:{number}}}'if number is not None else f'QueryResult::ResolutionFailure {{resolution_error:Outcome::{error}}}'if error else f'QueryResult::InvalidStorage {{storage_error:ByteOutcome::{storage}}}'if storage else 'QueryResult::UnsupportedValue'
  rows.append(dict(name=label,kind='query',body=body,call=call,expected=expected))
 for kind in ['Named','RefOf','Local','Arg','Index','Unresolved']:
  resolve(kind,seed(2)+ref(0,1,kind)+integer(1),object=1,seen=3)
  query('type_'+kind,seed(2)+ref(0,1,kind)+integer(1),'object_type(&store.space,0,64)',1,1)
 resolve('terminal',seed()+integer(0))
 resolve('uninitialized',seed())
 resolve('zero_budget',seed()+ref(0,MAX),budget=0,outcome='WorkLimit',seen=0)
 for budget in [65,MAX]:resolve('invalid_budget_'+str(budget),seed(),budget=budget,outcome='InvalidState',seen=0)
 for count,start,label in [(0,0,'empty'),(65,0,'count'),(1,64,'start'),(1,MAX,'max_start')]:resolve(label,seed(count),start=start,outcome='InvalidState',object=start,seen=0)
 resolve('bad_entries',seed()+'store.space.count=33;',outcome='InvalidState',seen=0)
 for target in [64,MAX]:resolve('dangling_'+str(target),seed()+ref(0,target),outcome='InvalidState',object=target)
 resolve('self',seed()+ref(0,0),outcome='ReferenceCycle')
 resolve('two_cycle',seed(2)+ref(0,1)+ref(1,0),outcome='ReferenceCycle',seen=3)
 resolve('short_cycle_budget',seed(2)+ref(0,1)+ref(1,0),budget=1,outcome='WorkLimit',object=1)
 chain=seed(64)+''.join(ref(i,i+1)for i in range(63))+integer(63)
 resolve('full_chain',chain,object=63,seen=MAX)
 resolve('short_chain',chain,budget=63,outcome='WorkLimit',object=63,seen=(1<<63)-1)
 resolve('full_cycle',seed(64)+''.join(ref(i,(i+1)%64)for i in range(64)),outcome='ReferenceCycle',seen=MAX)
 resolve('name',seed(2)+'store.space.count=1;'+bind(0,1)+lexical(0)+integer(1),object=1,seen=3)
 resolve('mixed',seed(4)+'store.space.count=1;'+bind(0,2)+ref(0,1)+lexical(1)+ref(2,3,'RefOf')+integer(3),object=3,seen=15)
 resolve('mixed_cycle',seed(3)+'store.space.count=1;'+bind(0,2)+ref(0,1)+lexical(1)+ref(2,0),outcome='ReferenceCycle',seen=7)
 resolve('lexical_cycle',seed()+'store.space.count=1;'+bind(0,0)+lexical(0),outcome='ReferenceCycle')
 resolve('name_missing',seed()+lexical(0),outcome='MissingObject')
 resolve('bad_lexical_target',seed()+'store.space.count=1;'+bind(0,MAX)+lexical(0),outcome='MissingObject')
 resolve('bad_name',seed()+lexical(0,0),outcome='InvalidName')
 # Scope search is exercised with complete root/child level metadata.
 scoped=seed(3)+'store.space.count=3;store.space.entries[0].path=Path {absolute:true};store.space.entries[0].has_level=true;store.space.entries[1].path=Path {absolute:true,count:1};store.space.entries[1].path.segments[0]=1600073796;store.space.entries[1].has_level=true;'+bind(2,1)+integer(1)+integer(2)+lexical(0)+'path0.absolute=false;let declared:Path=store.space.entries[1].path;store.space.objects[0].value=Value::NameReference {name:path0,scope:declared};'
 resolve('ancestor_search',scoped,object=1,seen=3)
 nearer=scoped+'store.space.count=4;'+bind(3,2)+'store.space.entries[3].path.count=2;store.space.entries[3].path.segments[0]=1600073796;store.space.entries[3].path.segments[1]=1600085848;'
 resolve('nearest_scope',nearer,object=2,seen=5)
 resolve('parent_prefix',nearer+'path0.parents=1;store.space.objects[0].value=Value::NameReference {name:path0,scope:declared};',object=1,seen=3)
 resolve('zero_name_budget',seed()+lexical(0),budget=0,outcome='WorkLimit',seen=0)
 query('mixed_buffer_size',seed(3)+'store.space.count=1;'+bind(0,2)+ref(0,1)+lexical(1)+f'store.space.objects[2].value={source()};','size_of(&input,2,7,&store,0,64)',4,2)
 for n,payload in [(0,'Uninitialized'),(1,'Integer {number:42}'),(2,'String {string_storage:StringStorage::Owned {string_owner:99}}'),(3,'Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:99}}'),(4,'Package {first:99,count:999}'),(6,'Device'),(7,'Event'),(8,'Method {flags:0,body:Span {}}'),(9,'Mutex {sync_level:0}'),(10,'OperationRegion {space:0,base:0,length:0,scope:Path {}}'),(11,'PowerResource {system_level:0,order:0}'),(12,'Processor {id:0,address:0,length:0}'),(13,'ThermalZone'),(14,'BufferField {backing_object:99,bit_offset:999,bit_length:999}')]:
  body=seed()+f'store.space.objects[0].value=Value::{payload};'
  query('payload_type_'+str(n),body,'object_type(&store.space,0,64)',n)
  if n not in [2,3,4]:query('unsupported_size_'+str(n),body,'size_of(&input,2,7,&store,0,64)')
 query('type_zero_budget',seed(),'object_type(&store.space,0,0)',error='WorkLimit')
 query('size_zero_budget',seed(),'size_of(&input,2,7,&store,0,0)',error='WorkLimit')
 for kind in ['Buffer','String']:
  for end,declared in [(0,0),(2,4),(4,2),(256,256)]:
   body=seed()+f'store.space.objects[0].value={source(kind,declared,end)};'+''.join(f'input[{i}]=65;'for i in range(end))
   query('source_size_'+kind+'_'+str(end),body,f'size_of(&input,{end},7,&store,0,64)',max(end,declared)if kind=='Buffer'else end)
  payload=f'Value::{kind} {{{kind.lower()}_storage:{kind}Storage::Owned {{{kind.lower()}_owner:0}}}}'
  body=seed()+f'store.space.objects[0].value={payload};store.bytes.blocks[0]=ByteBlock {{initialized:true,length:2}};store.bytes.blocks[0].bytes[0]=65;store.bytes.blocks[0].bytes[1]=66;'
  query('owned_size_'+kind,body,'size_of(&input,0,999,&store,0,64)',2)
  query('bad_owned_'+kind,seed()+f'store.space.objects[0].value={payload};','size_of(&input,2,7,&store,0,64)',storage='InvalidState')
 query('bad_source_unit',seed()+f'store.space.objects[0].value={source(unit=8)};','size_of(&input,2,7,&store,0,64)',storage='Bounds')
 query('bad_string_encoding',seed()+f'store.space.objects[0].value={source("String")};','size_of(&input,2,7,&store,0,64)',storage='Encoding')
 query('oversized_source',seed()+f'store.space.objects[0].value={source(declared=257)};','size_of(&input,2,7,&store,0,64)',storage='Capacity')
 for count in [0,1,2,63]:
  body=seed(count+1)+f'store.space.objects[0].value=Value::Package {{first:1,count:{count}}};'+''.join(integer(i)+ (f'store.space.objects[{i}].has_next=true;store.space.objects[{i}].next={i+1};'if i<count else '')for i in range(1,count+1))
  query('package_size_'+str(count),body,'size_of(&input,0,0,&store,0,64)',count)
 body=seed(3)+'store.space.objects[0].value=Value::Package {first:1,count:2};store.space.objects[1].has_next=true;store.space.objects[1].next=2;'
 query('package_tail_extra',body+'store.space.objects[2].has_next=true;','size_of(&input,0,0,&store,0,64)',error='BadEncoding')
 query('package_tail_missing',body+'store.space.objects[1].has_next=false;','size_of(&input,0,0,&store,0,64)',error='BadEncoding')
 query('package_cycle',body+'store.space.objects[1].next=1;','size_of(&input,0,0,&store,0,64)',error='ReferenceCycle')
 query('package_dangling',body+f'store.space.objects[1].next={MAX};','size_of(&input,0,0,&store,0,64)',error='InvalidState')
 query('package_capacity',seed()+'store.space.objects[0].value=Value::Package {first:1,count:65};','size_of(&input,0,0,&store,0,64)',error='Capacity')
 for obj,label in [(0,'valid'),(1,'past_count'),(MAX,'maximum')]:
  body=seed()+'store.space=namespace::empty();store.space.object_count=1;'+integer(0)+'let mut target:Entry=Entry {has_object:true,object:'+str(obj)+'};target.path=Path {absolute:true,count:1};target.path.segments[0]=1600085848;'
  expected='Success'if obj==0 else'InvalidState';checks='store.space.count==2 && store.space.entries[1].has_object && store.space.entries[1].object==0'if obj==0 else'store.space.count==1 && !store.space.entries[1].has_object'
  rows.append(dict(name='namespace_bind_'+label,kind='outcome',body=body,call='namespace::bind_mut(&mut store.space,target,false)',outcome=expected,check=checks))
 return rows

def render(row,control=False,machine='test_result'):
 signature=machine+'(&mut self)'if'::'in machine else machine+'()'
 text=f'machine {signature}->i32 {{\n'+row['body']+'\n'
 if row['kind']=='outcome':
  text+=f'let result:Outcome={row["call"]};let good:bool=result{"!="if control else"=="}Outcome::{row["outcome"]} && '+row['check']+';'
 elif row['kind']=='resolve':
  text+=f'let result:ReferenceState={row["call"]};let good:bool=result.done && result.outcome==Outcome::{row["outcome"]} && result.object=={row["object"]^int(control)} && result.visited=={row["seen"]};'
 else:
  expected=('QueryResult::UnsupportedValue'if row['expected']!='QueryResult::UnsupportedValue'else'QueryResult::Value {object:0,number:0}')if control else row['expected']
  text+=f'let result:QueryResult={row["call"]};let good:bool=query_equal(result,{expected});'
 return text+'transition good {true -> (0) _ -> (1)}}\n'
if __name__=='__main__':
 text=json.dumps(cases(),indent=2)+'\n';path=HERE/'cases.json'
 if '--check'in sys.argv:assert path.read_text()==text
 else:path.write_text(text)
 print(len(cases()),'mixed reference and object query scenarios')
