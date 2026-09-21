#!/usr/bin/env python3
"""Independent graph-policy cases and actual public Rust unwrap observations."""
import argparse,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
KINDS=['Named','RefOf','Local','Arg','Index','Unresolved']
IMPORTS='''use aml::model::Outcome;
use aml::model::Value;
use aml::model::Object;
use aml::model::Namespace;
use aml::model::NamespaceResult;
use aml::model::ReferenceKind;
use aml::model::ReferenceState;
use aml::model::Path;
use aml::model::Span;
use aml::object_references;
use aml::namespace;
'''
def seed(count):return f'let mut space:Namespace=Namespace {{}};space.object_count={count};\n'
def integer(at,number=9):return f'space.objects[{at}].value=Value::Integer {{number:{number}}};\n'
def ref(at,target,kind='Named'):return f'space.objects[{at}].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:{target}}};\n'
def package(at,first,count):return f'space.objects[{at}].value=Value::Package {{first:{first},count:{count}}};\n'
def link(at,to):return f'space.objects[{at}].has_next=true;space.objects[{at}].next={to};\n'
def cases():
 out=[]
 def add(name,body,check,negative,origin='Cathedral bounded graph/identity policy'):
  out.append(dict(name=name,body=body,check=check,mutation=[check,negative],origin=origin))
 def read(name,body,call,outcome='Success',selected=0,extra='',origin='Cathedral bounded graph/identity policy'):
  body+=f'let result:ReferenceState={call};\n';base=f'result.done && result.outcome==Outcome::{outcome} && result.object=={selected}';check=base+extra
  bad=check.replace('result.object=='+str(selected),'result.object=='+str(selected^1),1)
  add(name,body,check,bad,origin)
 observed=json.loads((HERE/'reference-verification.json').read_text())['observations']
 for row in observed:
  if 'chain'not in row:continue
  chain=row['chain'];body=seed(len(chain)+1)
  body+=('space.objects[0].value=Value::NameReference {name:Path {absolute:true},scope:Path {absolute:true}};\n'if row['name_path']else integer(0,(1<<64)-1))
  for i,kind in enumerate(chain):body+=ref(i+1,i,KINDS[kind])
  mode='transparent'if row['transparent']else'all';read('rust_'+row['name']+'_'+mode,body,f'object_references::unwrap_{mode}(&space,{len(chain)},64)',selected=row['selected'],origin='actual pinned public WrappedObject unwrap; pointer identity mapped to stable ordinal')
 for name,body,start,budget,transparent,outcome,selected,visited in [
  ('zero_budget',seed(1)+integer(0),0,0,False,'WorkLimit',0,0),
  ('edge_only_budget',seed(2)+integer(0)+ref(1,0),1,1,False,'WorkLimit',0,2),
  ('budget_max',seed(1)+integer(0),0,(1<<64)-1,False,'InvalidState',0,0),
  ('budget_65',seed(1)+integer(0),0,65,False,'InvalidState',0,0),
  ('empty_arena',seed(0),0,64,False,'InvalidState',0,0),
  ('bad_start',seed(1),64,64,False,'InvalidState',64,0),
  ('bad_count',seed(65),0,64,False,'InvalidState',0,0),
  ('dangling',seed(1)+ref(0,64),0,64,False,'InvalidState',64,1),
  ('dangling_max',seed(1)+ref(0,(1<<64)-1),0,64,False,'InvalidState',(1<<64)-1,1),
  ('opaque_dangling',seed(1)+ref(0,64,'RefOf'),0,1,True,'Success',0,1),
  ('self_cycle',seed(1)+ref(0,0),0,1,False,'ReferenceCycle',0,1),
  ('two_cycle',seed(2)+ref(0,1)+ref(1,0),0,2,False,'ReferenceCycle',0,3),
  ('cycle_short_budget',seed(2)+ref(0,1)+ref(1,0),0,1,False,'WorkLimit',1,1),
  ('opaque_self',seed(1)+ref(0,0,'Index'),0,1,True,'Success',0,1),
  ('uninitialized_terminal',seed(1),0,1,False,'Success',0,1),
 ]:
  mode='transparent'if transparent else'all';read(name,body,f'object_references::unwrap_{mode}(&space,{start},{budget})',outcome,selected,f' && result.visited=={visited}')
 read('object_arena_only_path_count',seed(1)+'space.count=18446744073709551615;\n'+integer(0),'object_references::unwrap_all(&space,0,1)','Success',0)
 fullcycle=seed(64)+''.join(ref(i,(i+1)%64)for i in range(64));read('full_cycle',fullcycle,'object_references::unwrap_all(&space,0,64)','ReferenceCycle',0,' && result.visited==18446744073709551615')
 pkg=seed(4)+integer(0,11)+ref(1,0,'RefOf')+integer(2,22)+link(0,1)+link(1,2)+package(3,0,3)
 for row in observed:
  if not row.get('package_selection'):continue
  read('rust_package_index_'+str(row['index']),pkg,f'object_references::package_element(&space,3,{row["index"]},64)','Success'if row['found']else'MissingObject',row['selected'],origin='actual pinned public Package payload Vec::get selection; bounded linked representation adaptation')
 for name,body,at,index,budget,outcome,selected in [
  ('package_empty',seed(1)+package(0,0,0),0,0,0,'MissingObject',0),
  ('package_missing_root',seed(0),0,0,64,'InvalidState',0),
  ('package_wrong_type',seed(1)+integer(0),0,0,64,'InvalidState',0),
  ('package_capacity',seed(1)+package(0,0,65),0,0,64,'Capacity',0),
  ('package_count_max',seed(1)+package(0,0,(1<<64)-1),0,0,64,'Capacity',0),
  ('package_budget_max',pkg,3,0,(1<<64)-1,'InvalidState',0),
  ('package_low_budget',pkg,3,0,2,'WorkLimit',0),
  ('package_dangling',seed(1)+package(0,64,1),0,0,64,'InvalidState',0),
  ('package_early_end',seed(2)+package(0,1,2)+integer(1),0,0,64,'BadEncoding',0),
  ('package_extra_link',seed(2)+package(0,1,1)+integer(1)+link(1,1),0,0,64,'BadEncoding',0),
  ('package_cycle',seed(3)+package(0,1,3)+integer(1)+integer(2)+link(1,2)+link(2,1),0,0,64,'ReferenceCycle',0),
  ('package_bad_tail_after_selection',seed(3)+package(0,1,2)+integer(1)+link(1,64),0,0,64,'InvalidState',0),
  ('package_name_is_not_resolved',seed(2)+package(0,1,1)+'space.objects[1].value=Value::NameReference {name:Path {absolute:true},scope:Path {absolute:true}};\n',0,0,64,'Success',1),
 ]:read(name,body,f'object_references::package_element(&space,{at},{index},{budget})',outcome,selected)
 longpkg=seed(64)+package(0,1,63)+''.join(integer(i,i)+(link(i,i+1)if i<63 else'')for i in range(1,64));read('package_maximum',longpkg,'object_references::package_element(&space,0,62,64)','Success',63)
 for kind in KINDS:
  body=seed(1)+integer(0)+f'let allocated:NamespaceResult=object_references::allocate_reference(space,ReferenceKind::{kind},0);\nlet followed:ReferenceState=object_references::unwrap_all(&allocated.space,allocated.object,64);\n'
  body+=f'let kind_ok:bool=oref_reference(&allocated.space,allocated.object,ReferenceKind::{kind},0);\n'
  check='kind_ok && allocated.outcome==Outcome::Success && allocated.object==1 && allocated.space.object_count==2 && followed.object==0 && followed.outcome==Outcome::Success'
  add('allocate_'+kind.lower(),body,check,check.replace('allocated.object==1','allocated.object==0'),'actual public Reference construction; fresh stable identity adaptation')
 for name,count,target,outcome in [('allocation_full',64,0,'Capacity'),('allocation_bad_target',1,64,'InvalidState'),('allocation_target_max',1,(1<<64)-1,'InvalidState'),('allocation_empty',0,0,'InvalidState'),('allocation_bad_count',65,0,'InvalidState')]:
  body=seed(count)+integer(0,77)+link(0,21)+f'let result:NamespaceResult=object_references::allocate_reference(space,ReferenceKind::Named,{target});\n'
  check=f'result.outcome==Outcome::{outcome} && result.space.object_count=={count} && result.space.objects[0].has_next && result.space.objects[0].next==21';add(name,body,check,check.replace('next==21','next==22'))
 # Copy tests assert destination identity and both source/destination link metadata.
 for label,payload,payload_check in [
  ('integer','Value::Integer {number:18446744073709551615}','oref_integer(&result.space,1,18446744073709551615)'),
  ('reference','Value::Reference {kind:ReferenceKind::RefOf,object_id:0}','oref_reference(&result.space,1,ReferenceKind::RefOf,0)'),
  ('package','Value::Package {first:2,count:1}','oref_package(&result.space,1,2,1)'),
  ('method','Value::Method {flags:2,body:Span {unit:4,start:10,end:20}}','oref_method(&result.space,1,2,4,10,20)'),
  ('uninitialized','Value::Uninitialized','oref_uninitialized(&result.space,1)'),
 ]:
  body=seed(3)+f'space.objects[0].value={payload};\n'+integer(1,6)+integer(2,7)+link(0,11)+link(1,22)+'let result:NamespaceResult=object_references::copy_value(space,1,0);\n'+f'let payload_ok:bool={payload_check};\n'
  check='result.outcome==Outcome::Success && result.object==1 && result.space.object_count==3 && payload_ok && result.space.objects[0].has_next && result.space.objects[0].next==11 && result.space.objects[1].has_next && result.space.objects[1].next==22';add('copy_'+label,body,check,check.replace('next==22','next==23'),'public Object::clone value/identity observations; namespace link-preservation policy')
 for label,source,destination,count in [('copy_self',0,0,1),('copy_bad_source',64,0,1),('copy_bad_destination',0,64,1),('copy_bad_count',0,0,65)]:
  body=seed(count)+integer(0,44)+link(0,33)+f'let result:NamespaceResult=object_references::copy_value(space,{destination},{source});\nlet payload_ok:bool=oref_integer(&result.space,0,44);\n';expected='Success'if label=='copy_self'else'InvalidState';check=f'result.outcome==Outcome::{expected} && result.space.object_count=={count} && payload_ok && result.space.objects[0].has_next && result.space.objects[0].next==33';add(label,body,check,check.replace('next==33','next==34'))
 body=pkg+'space.object_count=5;\nlet copied:NamespaceResult=object_references::copy_value(space,4,3);\nlet mut shared:Namespace=copied.space;shared.objects[0].value=Value::Integer {number:99};\nlet first:ReferenceState=object_references::package_element(&shared,3,0,64);\nlet second:ReferenceState=object_references::package_element(&shared,4,0,64);\nlet shared_value:bool=oref_integer(&shared,second.object,99);\n'
 check='copied.outcome==Outcome::Success && first.outcome==Outcome::Success && second.outcome==Outcome::Success && first.object==second.object && shared_value'
 add('copy_package_shares_elements',body,check,check.replace('first.object==second.object','first.object!=second.object'),'public Object::Package clone retains child identity')
 # Bind two names to one old identity, then rebind the original name and copy by ID.
 body='let initial:Namespace=namespace::empty();\nlet mut first:Path=Path {absolute:true,count:1};first.segments[0]=1145258561;\nlet mut alias:Path=Path {absolute:true,count:1};alias.segments[0]=1212630597;\nlet inserted:NamespaceResult=namespace::insert(initial,first,Value::Integer {number:11});\nlet bound:NamespaceResult=namespace::bind(inserted.space,alias,inserted.object);\nlet rebound:NamespaceResult=namespace::insert(bound.space,first,Value::Integer {number:22});\nlet copied:NamespaceResult=object_references::copy_value(rebound.space,inserted.object,rebound.object);\nlet old:aml::model::Lookup=namespace::get(copied.space,alias);\nlet current:aml::model::Lookup=namespace::get(copied.space,first);\nlet old_value:bool=oref_integer(&copied.space,old.object,22);\n'
 check='old.outcome==Outcome::Success && current.outcome==Outcome::Success && old.object==inserted.object && current.object==rebound.object && old.object!=current.object && old_value';add('alias_rebinding_stable_ids',body,check,check.replace('old.object!=current.object','old.object==current.object'))
 return out
HELPERS='''machine oref_integer(space:&Namespace,index:u64,expected:u64)->bool {transition index<64 {true -> check(space.objects[index].value,expected) _ -> (false)} state check(value:Value,expected:u64)->bool {transition value {Value::Integer {number} -> (number==expected) _ -> (false)}}}
machine oref_reference(space:&Namespace,index:u64,expected_kind:ReferenceKind,expected_id:u64)->bool {transition index<64 {true -> check(space.objects[index].value,expected_kind,expected_id) _ -> (false)} state check(value:Value,expected_kind:ReferenceKind,expected_id:u64)->bool {transition value {Value::Reference {kind,object_id} -> (kind==expected_kind && object_id==expected_id) _ -> (false)}}}
machine oref_package(space:&Namespace,index:u64,expected_first:u64,expected_count:u64)->bool {transition index<64 {true -> check(space.objects[index].value,expected_first,expected_count) _ -> (false)} state check(value:Value,expected_first:u64,expected_count:u64)->bool {transition value {Value::Package {first,count} -> (first==expected_first && count==expected_count) _ -> (false)}}}
machine oref_uninitialized(space:&Namespace,index:u64)->bool {transition index<64 {true -> check(space.objects[index].value) _ -> (false)} state check(value:Value)->bool {transition value {Value::Uninitialized -> (true) _ -> (false)}}}
machine oref_method(space:&Namespace,index:u64,expected_flags:u8,unit:u64,start:u64,end:u64)->bool {transition index<64 {true -> check(space.objects[index].value,expected_flags,unit,start,end) _ -> (false)} state check(value:Value,expected_flags:u8,unit:u64,start:u64,end:u64)->bool {transition value {Value::Method {flags,body} -> (flags==expected_flags && body.unit==unit && body.start==start && body.end==end) _ -> (false)}}}
'''
def render(row,control=False,machine='test_result'):
 check=row['mutation'][1]if control else row['check'];return 'machine '+machine+('(&mut self)'if'::'in machine else'()')+'->i32 {\n'+row['body']+'transition '+check+' {true -> (0) _ -> (1)}\n}\n'
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();rows=cases();text=json.dumps(rows,indent=2)+'\n';out=HERE/'cases.json'
 if a.check:assert out.read_text()==text
 else:out.write_text(text)
 print(len(rows),'object reference cases', 'verified'if a.check else'generated')
if __name__=='__main__':main()
