from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
BASE=[('',True,False,0),('AAAA',True,True,0),('AAAA.BBBB',True,True,1),('AAAA.BBBB.DATA',False,True,2),('AAAA.KEEP',False,True,3),('OTHR',True,True,4),('OTHR.DATA',False,True,5),('ONLY',False,True,6),('ALIA',False,True,2)]
def path(text):
 parts=text.split('.')if text else[]
 lines=[f'let mut p:Path=Path {{absolute:true,count:{len(parts)}}};']
 lines +=[f'p.segments[{i}]={int.from_bytes(s.encode(),"little")};'for i,s in enumerate(parts)]
 return ''.join(lines)+'p'
def cases():
 rows=[dict(name='subtree_preserves_same_path_object',target='AAAA',kept=[0,1,5,6,7,8],clear=1),dict(name='nested_subtree',target='AAAA.BBBB',kept=[0,1,2,4,5,6,7,8],clear=2),dict(name='other_subtree',target='OTHR',kept=[0,1,2,3,4,5,7,8],clear=5)]
 rows +=[dict(name=n,target=p)for n,p in [('root_noop',''),('object_only_noop','ONLY'),('leaf_object_noop','AAAA.KEEP'),('missing_child_noop','NONE'),('alias_only_noop','ALIA')]]
 rows.append(dict(name='missing_parent',target='NONE.SUB_',outcome='MissingLevel'))
 rows.append(dict(name='leaf_level_only',target='AAAA.KEEP',mutate='s.entries[4].has_object=false;s.entries[4].has_level=true;',kept=[0,1,2,3,5,6,7,8]))
 rows.append(dict(name='root_child_level_only',target='OTHR',mutate='s.entries[5].has_object=false;',kept=[0,1,2,3,4,7,8]))
 for field,values in [('count',[0,33,1<<63,(1<<64)-1]),('object_count',[65,1<<63,(1<<64)-1])]:
  for v in values:rows.append(dict(name=f'bad_{field}_{v}',target='AAAA',mutate=f's.{field}={v};',outcome='InvalidState'))
 for field,values in [('count',[17,1<<63,(1<<64)-1]),('parents',[1,17,1<<63,(1<<64)-1])]:
  for v in values:rows.append(dict(name=f'bad_path_{field}_{v}',target='AAAA',path_mutate=f'p.{field}={v};',outcome='InvalidName'))
 rows.append(dict(name='relative_path',target='AAAA',path_mutate='p.absolute=false;',outcome='NotAbsolute'))
 rows.append(dict(name='bad_target_segment',target='AAAA',path_mutate='p.segments[0]=0;',outcome='InvalidName'))
 for name,mutation in [('entry_relative','s.entries[8].path.absolute=false;'),('entry_bad_segment','s.entries[8].path.segments[0]=0;'),('entry_count_max','s.entries[8].path.count=18446744073709551615;'),('entry_parents_max','s.entries[8].path.parents=18446744073709551615;'),('entry_id_max','s.entries[8].object=18446744073709551615;'),('entry_id_count','s.entries[8].object=7;'),('entry_empty','s.entries[8].has_object=false;'),('duplicate_path','s.entries[8].path=s.entries[7].path;'),('missing_root','s.entries[0].has_level=false;'),('orphan','s.entries[1].has_level=false;')]:rows.append(dict(name=name,target='AAAA',mutate=mutation,outcome='InvalidState'))
 rows.append(dict(name='unrelated_invalid_entry_before_noop',target='',mutate='s.entries[8].object=18446744073709551615;',outcome='InvalidState'))
 rows += [dict(name='full_subtree',target='AAAA',seed='full',kept=[0,1],clear=1),dict(name='full_level_only',target='AAAA',seed='full',mutate='s.entries[1].has_object=false;',kept=[0]),dict(name='full_invalid_last_object',target='AAAA',seed='full',mutate='s.entries[31].object=64;',outcome='InvalidState'),dict(name='full_root_noop',target='',seed='full')]
 deep=['%sAAA'%chr(65+i)for i in range(16)]
 rows += [dict(name='deep_subtree',target='.'.join(deep[:8]),seed='deep',kept=list(range(9)),clear=8),dict(name='deep_leaf',target='.'.join(deep),seed='deep',kept=list(range(17)),clear=16)]
 for row in rows:row.setdefault('outcome','Success')
 return rows
IMPORTS='''use aml::model::Namespace;
use aml::model::Object;
use aml::model::Entry;
use aml::model::Path;
use aml::model::LevelKind;
use aml::model::Outcome;
use aml::model::Value;
use aml::model::BufferStorage;
use aml::namespace_removal::remove_level;
'''
def helpers():
 text=''
 paths=sorted(set([r['target']for r in cases()]+[x[0]for x in BASE]))
 for i,p in enumerate(paths):text+=f'machine path_{i}()->Path{{'+path(p)+'}\n'
 text+='machine base()->Namespace{let mut s:Namespace=Namespace {count:9,object_count:7};seed_objects(&mut s,0,64);\n'
 for i,(p,level,obj,objid)in enumerate(BASE):
  text+=f'let p{i}:Path=path_{paths.index(p)}();s.entries[{i}]=Entry {{path:p{i},has_level:{str(level).lower()},has_object:{str(obj).lower()},object:{objid},alias:{str(i==8).lower()},level:LevelKind::Device}};\n'
 text+='s.entries[31]=Entry {path:Path {parents:18446744073709551615,count:18446744073709551615},object:18446744073709551615,alias:true};s}\n'
 text+='''machine base_full()->Namespace {let mut s:Namespace=base();s.count=32;s.object_count=64;full_entries(&mut s,2,32);s}
machine full_entries(s:&mut Namespace,index:u64,count:u64)
terminates by(index,count)->Nat::BoundedDistance;
{full_one(s,index);transition index<count {true -> full_entries(s,index+1,count) _ -> {}}}
machine full_one(s:&mut Namespace,index:u64){transition index>=2 && index<32 {true -> put(s,index) _ -> {}}
state put(s:&mut Namespace,index:u64){let mut p:Path=Path {absolute:true,count:2};p.segments[0]=0x41414141;p.segments[1]=(0x41414141+((index-2)%26)*256+((index-2)/26)*65536) as u32;s.entries[index]=Entry {path:p,has_object:true,object:index};}}
machine base_deep()->Namespace {let mut s:Namespace=base();s.count=17;s.object_count=64;deep_entries(&mut s,1,17,Path {absolute:true});s}
machine deep_entries(s:&mut Namespace,index:u64,count:u64,prior:Path)
terminates by(index,count)->Nat::BoundedDistance;
{let next:Path=deep_one(s,index,prior);transition index<count {true -> deep_entries(s,index+1,count,next) _ -> {}}}
machine deep_one(s:&mut Namespace,index:u64,prior:Path)->Path {transition index>0 && index<=16 {true -> slot(s,index,prior) _ -> (prior)}
state slot(s:&mut Namespace,index:u64,prior:Path)->Path {let slot:u64=index-1;transition slot<16 {true -> put(s,index,prior,slot) _ -> (prior)}}
state put(s:&mut Namespace,index:u64,prior:Path,slot:u64)->Path{let mut p:Path=prior;p.count=index;p.segments[slot]=(0x41414141+index-1) as u32;s.entries[index]=Entry {path:p,has_level:true,has_object:true,object:index};p}}
machine seed_objects(s:&mut Namespace,index:u64,count:u64)
terminates by(index,count)->Nat::BoundedDistance;
{seed_one(s,index);transition index<count {true -> seed_objects(s,index+1,count) _ -> {}}}
machine seed_one(s:&mut Namespace,index:u64){transition index<64 {true -> put(s,index) _ -> {}} state put(s:&mut Namespace,index:u64){s.objects[index]=Object {value:Value::Integer {number:1000+index},has_next:true,next:18446744073709551615-index};}}
machine path_same(a:Path,b:Path)->bool {let same:bool=segments_same(a,b,0,16,true);a.absolute==b.absolute && a.count==b.count && a.parents==b.parents && same}
machine segments_same(a:Path,b:Path,index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let same:bool=segment_same(a,b,index);transition index<count {true -> segments_same(a,b,index+1,count,prior && same) _ -> (prior)}}
machine segment_same(a:Path,b:Path,index:u64)->bool {transition index<16 {true -> (a.segments[index]==b.segments[index]) _ -> (true)}}
machine entry_same(a:Entry,b:Entry)->bool {let same:bool=path_same(a.path,b.path);same && a.has_level==b.has_level && a.has_object==b.has_object && a.level==b.level && a.object==b.object && a.alias==b.alias}
machine entries_same(a:&Namespace,b:&Namespace,index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let same:bool=entry_at_same(a,b,index);transition index<count {true -> entries_same(a,b,index+1,count,prior && same) _ -> (prior)}}
machine entry_at_same(a:&Namespace,b:&Namespace,index:u64)->bool {transition index<32 {true -> compare(a.entries[index],b.entries[index]) _ -> (true)} state compare(a:Entry,b:Entry)->bool {let same:bool=entry_same(a,b);same}}
machine value_word(v:Value)->u64 {transition v {Value::Integer {number} -> (number) _ -> (0)}}
machine object_same(a:Object,b:Object)->bool {let x:u64=value_word(a.value);let y:u64=value_word(b.value);x==y && a.has_next==b.has_next && a.next==b.next}
machine objects_same(a:&Namespace,b:&Namespace,index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let same:bool=object_at_same(a,b,index);transition index<count {true -> objects_same(a,b,index+1,count,prior && same) _ -> (prior)}}
machine object_at_same(a:&Namespace,b:&Namespace,index:u64)->bool {transition index<64 {true -> compare(a.objects[index],b.objects[index]) _ -> (true)} state compare(a:Object,b:Object)->bool {let same:bool=object_same(a,b);same}}
machine check_result(a:&Namespace,b:&Namespace,outcome:Outcome,expected:Outcome)->i32 {let entries:bool=entries_same(a,b,0,32,true);let objects:bool=objects_same(a,b,0,64,true);transition entries && objects && a.count==b.count && a.object_count==b.object_count && outcome==expected {true -> (0) _ -> (1)}}
'''
 return text,paths
def body(row,control=False):
 _,paths=helpers();text='let mut s:Namespace='+('base_'+row['seed']if 'seed'in row else'base')+'();'+row.get('mutate','')+'let mut expected:Namespace=Namespace {count:s.count,object_count:s.object_count,entries:s.entries,objects:s.objects};\n'
 if 'kept'in row:
  text+='let mut entries:[Entry;32];\n'
  for i,k in enumerate(row['kept']):
   text+=f'entries[{i}]=s.entries[{k}];\n'
   if k==row.get('clear'):text+=f'entries[{i}].has_level=false;entries[{i}].level=LevelKind::Scope;\n'
  text+=f'expected.entries=entries;expected.count={len(row["kept"])};\n'
 text+=f'let mut p:Path=path_{paths.index(row["target"])}();'+row.get('path_mutate','')+'let outcome:Outcome=remove_level(&mut s,p);\n'
 if control:text+='expected.objects[63].next=0;\n'
 text+=f'let result:i32=check_result(&s,&expected,outcome,Outcome::{row["outcome"]});result\n';return text
def render(rows):
 shared,_=helpers();text=IMPORTS+shared+'data Suite{}\n';names=[]
 for row in rows:
  for control in [False,True]:
   name='Suite::'+row['name']+('_control'if control else'_positive');names.append(name+'='+str(int(control)));text+='machine '+name+'(&mut self)->i32{'+body(row,control)+'}\n'
 return text,names
