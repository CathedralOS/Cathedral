#!/usr/bin/env python3
"""Original strict Omega body fixtures; public pin differences remain explicit."""
import json
from pathlib import Path
HERE=Path(__file__).resolve().parent
HEAD='''use prt::routes;
use prt::link_resources;
use prt::model::Fault;
use prt::model::Target;
use prt::model::Route;
use prt::model::Routes;
use prt::model::Decode;
use prt::model::Selection;
use prt::model::Request;
use prt::model::ResourceSelection;
use aml::model::Outcome;
use aml::model::Value;
use aml::model::Object;
use aml::model::ObjectStore;
use aml::model::Path;
use aml::model::Entry;
use aml::model::LevelKind;
use aml::model::ReferenceKind;
use aml::model::StringStorage;
use aml::model::BufferStorage;
use aml::model::Span;
use aml::model::ByteBlock;
use aml::byte_storage::ByteOutcome;
use aml::names;
use aml::namespace;
use resources::resource_model;
use resources::resource_irq;
machine path(absolute:bool,parents:u64,a:u32,b:u32,count:u64)->Path {
 let mut result:Path=Path {absolute:absolute,parents:parents,count:count};result.segments[0]=a;result.segments[1]=b;result
}
machine basic(address:u64,pin:u64,source:Value,index:u64)->ObjectStore {
 let mut store:ObjectStore=ObjectStore {};
 store.space.count=1;store.space.entries[0]=Entry {path:Path {absolute:true},has_level:true};
 store.space.object_count=6;
 store.space.objects[0]=Object {value:Value::Package {first:1,count:1}};
 store.space.objects[1]=Object {value:Value::Package {first:2,count:4}};
 store.space.objects[2]=Object {value:Value::Integer {number:address},has_next:true,next:3};
 store.space.objects[3]=Object {value:Value::Integer {number:pin},has_next:true,next:4};
 store.space.objects[4]=Object {value:source,has_next:true,next:5};
 store.space.objects[5]=Object {value:Value::Integer {number:index}};
 store
}
machine levels(store:ObjectStore,local:bool)->ObjectStore {
 let mut out:ObjectStore=store;let pci:Path=path(true,0,0x30494350,0,1);let link:Path=path(true,0,0x414b4e4c,0,1);
 out.space.entries[1]=Entry {path:pci,has_level:true,level:LevelKind::Device};
 out.space.entries[2]=Entry {path:link,has_level:true,level:LevelKind::Device};out.space.count=3;
 transition local {true -> nearest(out,pci) _ -> (out)}
 state nearest(store:ObjectStore,pci:Path)->ObjectStore {
  let mut out:ObjectStore=store;let link:Path=path(true,0,0x30494350,0x414b4e4c,2);
  out.space.entries[3]=Entry {path:link,has_level:true,level:LevelKind::Device};out.space.count=4;out
 }
}
machine empty_route(route:Route)->bool {
 transition route.target {Target::None -> (route.device==0 && route.pin==0) _ -> (false)}
}
machine zero_routes(routes:&Routes,index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let good:bool=zero_slot(routes,index);transition index<count {true -> zero_routes(routes,index+1,count,prior && good) _ -> (prior)}}
machine zero_slot(routes:&Routes,index:u64)->bool {transition index<32 {true -> checked(routes.entries[index]) _ -> (true)} state checked(route:Route)->bool {let good:bool=empty_route(route);good}}
machine failure(store:&ObjectStore,outcome:Outcome,fault:Fault,row:u64,budget:u64)->bool {
 let result:Decode=routes::decode(store,0,budget);let empty:bool=zero_routes(&result.routes,0,32,true);
 result.outcome==outcome && result.fault==fault && result.row==row && result.routes.count==0 && empty
}
machine direct_slot(route:Route,expected:u32,pin:u8)->bool {
 transition route.target {Target::Gsi {number} -> (route.device==3 && route.pin==pin && number==expected) _ -> (false)}
}
machine direct_rows(table:&Routes,index:u64,limit:u64,count:u64,expected:u32,second:u32,pin:u8,prior:bool)
terminates by(index,limit)->Nat::BoundedDistance;
->bool {
 let good:bool=direct_slot_at(table,index,count,expected,second,pin);
 transition index<limit {true -> direct_rows(table,index+1,limit,count,expected,second,pin,prior && good) _ -> (prior)}
}
machine direct_slot_at(table:&Routes,index:u64,count:u64,expected:u32,second:u32,pin:u8)->bool {
 transition index<32 {true -> slot(table.entries[index],index,count,expected,second,pin) _ -> (true)}
 state slot(route:Route,index:u64,count:u64,expected:u32,second:u32,pin:u8)->bool {let good:bool=direct_row(route,index,count,expected,second,pin);good}
}
machine direct_row(route:Route,index:u64,count:u64,expected:u32,second:u32,pin:u8)->bool {
 transition index<count {true -> used(route,index,expected,second,pin) _ -> vacant(route)}
 state used(route:Route,index:u64,expected:u32,second:u32,pin:u8)->bool {
  transition index==1 {true -> selected(route,second,pin) _ -> selected(route,expected,pin)}
 }
 state selected(route:Route,expected:u32,pin:u8)->bool {let good:bool=direct_slot(route,expected,pin);good}
 state vacant(route:Route)->bool {let good:bool=empty_route(route);good}
}
machine direct_all(store:&ObjectStore,count:u64,expected:u32,second:u32,pin:u8)->bool {
 let result:Decode=routes::decode(store,0,64);
 let selected:Selection=routes::select(&result.routes,3,pin);
 let rows:bool=direct_rows(&result.routes,0,32,count,expected,second,pin,true);
 let good:bool=result.outcome==Outcome::Success && result.fault==Fault::None && result.routes.count==count && selected.outcome==Outcome::Success && selected.row==0 && rows;
 transition selected.target {Target::Gsi {number} -> (good && number==expected) _ -> (false)}
}
machine direct(store:&ObjectStore,count:u64,expected:u32,pin:u8)->bool {let good:bool=direct_all(store,count,expected,expected,pin);good}
machine link_check(store:&ObjectStore,expected:Path,index:u32)->bool {
 let result:Decode=routes::decode(store,0,64);let selected:Selection=routes::select(&result.routes,3,0);
 let tail:bool=zero_routes(&result.routes,1,32,true);
 let first:Route=result.routes.entries[0];
 let row_good:bool=link_row(first,expected,index);
 let good:bool=result.outcome==Outcome::Success && result.routes.count==1 && selected.outcome==Outcome::Success && selected.row==0 && tail && row_good;
 transition selected.target {Target::Link {path,source_index} -> link(good,path,source_index,expected,index) _ -> (false)}
 state link(good:bool,path:Path,source_index:u32,expected:Path,index:u32)->bool {
  let same:bool=names::path_equal(path,expected);let request:Request=routes::crs_request(Target::Link {path:path,source_index:source_index});
  let exact:bool=crs_path(request.path,expected);
  good && same && source_index==index && request.outcome==Outcome::Success && request.source_index==index && exact
 }
}
machine link_row(route:Route,expected:Path,index:u32)->bool {
 transition route.target {Target::Link {path,source_index} -> linked(route.device,route.pin,path,source_index,expected,index) _ -> (false)}
 state linked(device:u16,pin:u8,path:Path,source_index:u32,expected:Path,index:u32)->bool {let same:bool=names::path_equal(path,expected);device==3 && pin==0 && same && source_index==index}
}
machine crs_path(actual:Path,parent:Path)->bool {
 let count:u64=parent.count;
 transition count<16 {true -> expected(actual,parent,count) _ -> (false)}
 state expected(actual:Path,parent:Path,count:u64)->bool {
  let mut path:Path=parent;path.segments[count]=0x5352435f;path.count=count+1;
  let same:bool=names::path_equal(actual,path);same
 }
}
machine all_bytes(actual:&[u8;4096],expected:&[u8;4096],length:u64,index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {
 let same:bool=one_byte(actual,expected,length,index);
 transition index<count {true -> all_bytes(actual,expected,length,index+1,count,prior && same) _ -> (prior)}
}
machine one_byte(actual:&[u8;4096],expected:&[u8;4096],length:u64,index:u64)->bool {
 transition index<4096 {true -> live(actual,expected,length,index) _ -> (true)}
 state live(actual:&[u8;4096],expected:&[u8;4096],length:u64,index:u64)->bool {
  transition index<length {true -> (actual[index]==expected[index]) _ -> (actual[index]==0)}
 }
}
machine resource_good(result:ResourceSelection,expected:u64,length:u64)->bool {
 let number:resource_model::NumberResult=resource_irq::irq_at(&result.bytes,result.length,result.irq.interrupts,0);
 result.outcome==Outcome::Success && result.length==length && number.outcome==resource_model::Outcome::Success && number.value==expected
}
machine resource_bad(result:ResourceSelection,outcome:Outcome,fault:Fault)->bool {
 let empty:[u8;4096];let zero:bool=all_bytes(&result.bytes,&empty,0,0,4096,true);
 result.outcome==outcome && result.fault==fault && result.length==0 && zero
}
data PrtSuite {}
'''

def cases():
 rows=[]
 def add(name,setup,check,old,new,origin='Original bounded structural/semantic fixture'):
  body=setup+'\n let good:bool='+check+';\n transition good {true -> (0) _ -> (1)}\n'
  assert body.count(old)==1,(name,old,body)
  rows.append(dict(name=name,body=body,control=body.replace(old,new),origin=origin))
 def baseline(source='Value::Integer {number:0}',address=0x3ffff,pin=0,index=40):return f' let mut store:ObjectStore=basic({address},{pin},{source},{index});'
 def error(name,patch,outcome='BadEncoding',fault='Package',row=0,budget=64,**kw):
  setup=baseline(**kw)+patch;check=f'failure(&store,Outcome::{outcome},Fault::{fault},{row},{budget})';add(name,setup,check,'Outcome::'+outcome,'Outcome::Success')
 for pin in range(4):add('direct_pin_'+str(pin),baseline(pin=pin),f'direct(&store,1,40,{pin})','direct(&store,1,40,','direct(&store,1,41,','Public direct_all_pins')
 for gsi in [0,1,0xffffffff]:add('gsi_'+str(gsi),baseline(index=gsi),f'direct(&store,1,{gsi},0)',f'direct(&store,1,{gsi},',f'direct(&store,1,{gsi^1},','Public GSI boundary')
 error('address_overflow','',fault='Address',address=0x10003ffff)
 error('function_specific','',fault='Address',address=0x30001)
 error('pin_high','',fault='Pin',pin=(1<<64)-1)
 error('source_nonzero','',fault='Source',source='Value::Integer {number:1}')
 error('source_string','',fault='Source',source='Value::String {string_storage:StringStorage::Owned {string_owner:4}}')
 error('index_overflow','',fault='SourceIndex',index=1<<32)
 error('index_wrong_type','store.space.objects[5].value=Value::Uninitialized;',fault='SourceIndex')
 error('address_wrong_type','store.space.objects[2].value=Value::Uninitialized;',fault='Address')
 error('pin_wrong_type','store.space.objects[3].value=Value::Uninitialized;',fault='Pin')
 for count in [0,3,5]:error('row_count_'+str(count),f'store.space.objects[1].value=Value::Package {{first:2,count:{count}}};')
 error('outer_wrong_type','store.space.objects[0].value=Value::Integer {number:0};')
 error('row_wrong_type','store.space.objects[1].value=Value::Integer {number:0};')
 error('outer_capacity','store.space.objects[0].value=Value::Package {first:1,count:33};',outcome='Capacity',fault='Table')
 error('bad_store_capacity','store.space.object_count=65;',outcome='InvalidState')
 error('bad_namespace_capacity','store.space.count=33;',outcome='InvalidState')
 error('bad_root_id','store.space.object_count=0;',outcome='InvalidState')
 error('bad_budget','',outcome='InvalidState',budget=65)
 error('small_budget','',outcome='WorkLimit',budget=3)
 error('outer_extra_tail','store.space.objects[1].has_next=true;store.space.objects[1].next=1;')
 error('outer_cycle','store.space.objects[0].value=Value::Package {first:1,count:2};store.space.objects[1].has_next=true;store.space.objects[1].next=1;',outcome='ReferenceCycle')
 error('field_missing_tail','store.space.objects[3].has_next=false;')
 error('field_extra_tail','store.space.objects[5].has_next=true;store.space.objects[5].next=2;')
 error('field_cycle','store.space.objects[3].next=2;',outcome='ReferenceCycle')
 error('field_missing_object','store.space.objects[3].next=63;',outcome='InvalidState')
 setup=baseline()+'store.space.object_count=7;store.space.objects[4].value=Value::Reference {kind:ReferenceKind::RefOf,object_id:6};store.space.objects[6].value=Value::Integer {number:0};'
 add('source_reference',setup,'direct(&store,1,40,0)','direct(&store,1,40,','direct(&store,1,41,')
 error('source_reference_cycle','store.space.objects[4].value=Value::Reference {kind:ReferenceKind::Index,object_id:4};',outcome='ReferenceCycle',fault='Source')
 error('source_reference_missing','store.space.objects[4].value=Value::Reference {kind:ReferenceKind::Named,object_id:63};',outcome='InvalidState',fault='Source')
 setup=baseline()+'store.space.objects[0].value=Value::Package {first:0,count:0};let result:Decode=routes::decode(&store,0,64);'
 add('empty',setup,'result.outcome==Outcome::Success && result.routes.count==0 && zero_routes(&result.routes,0,32,true)','result.routes.count==0','result.routes.count==1')
 # Typed namespace guards must reject high-bit counts before any scan/allocation.
 for field,values in [('count',[33,2**63,2**64-1]),('object_count',[65,2**63,2**64-1])]:
  for value in values:
   initial=baseline()+f'store.space.{field}={value};'
   add(f'namespace_locate_{field}_{value}',initial+'let result:aml::model::Lookup=namespace::locate(store.space,Path {absolute:true});','result.outcome==Outcome::InvalidState','Outcome::InvalidState','Outcome::Success')
   add(f'namespace_allocate_{field}_{value}',initial+'let result:aml::model::Read=namespace::allocate_mut(&mut store.space,Value::Integer {number:99});',f'result.outcome==Outcome::Capacity && store.space.{field}=={value}','Outcome::Capacity','Outcome::Success')
   add(f'namespace_bind_{field}_{value}',initial+'let candidate:Entry=Entry {path:path(true,0,0x41414141,0,1),has_object:true,object:0};let outcome:Outcome=namespace::bind_mut(&mut store.space,candidate,false);',f'outcome==Outcome::InvalidState && store.space.{field}=={value}','Outcome::InvalidState','Outcome::Success')
   error(f'decode_{field}_{value}',f'store.space.{field}={value};',outcome='InvalidState')
 for field in ['count','parents']:
  for value in [17,2**63,2**64-1]:
   malformed=f'let mut invalid:Path=path(true,0,0x414b4e4c,0,1);invalid.{field}={value};'
   setup=malformed+baseline(source='Value::NameReference {name:invalid,scope:Path {absolute:true}}')+'store=levels(store,false);'
   add(f'link_invalid_{field}_{value}',setup,'failure(&store,Outcome::InvalidName,Fault::LinkLevel,0,64)','Outcome::InvalidName','Outcome::Success')
   setup=malformed+'let mut table:Routes=Routes {count:1};table.entries[0]=Route {target:Target::Link {path:invalid,source_index:0}};let result:Selection=routes::select(&table,0,0);'
   add(f'select_invalid_{field}_{value}',setup,'result.outcome==Outcome::InvalidState','Outcome::InvalidState','Outcome::Success')
   setup=malformed+'let result:Request=routes::crs_request(Target::Link {path:invalid,source_index:0});'
   add(f'crs_invalid_{field}_{value}',setup,'result.outcome==Outcome::InvalidName','Outcome::InvalidName','Outcome::Success')
 # Two complete rows; failure of the second must discard the first candidate.
 second='store.space.object_count=11;store.space.objects[0].value=Value::Package {first:1,count:2};store.space.objects[1].has_next=true;store.space.objects[1].next=6;store.space.objects[6]=Object {value:Value::Package {first:7,count:4}};store.space.objects[7]=Object {value:Value::Integer {number:262143},has_next:true,next:8};store.space.objects[8]=Object {value:Value::Integer {number:0},has_next:true,next:9};store.space.objects[9]=Object {value:Value::Integer {number:0},has_next:true,next:10};store.space.objects[10]=Object {value:Value::Integer {number:41}};'
 add('first_match',baseline()+second,'direct_all(&store,2,40,41,0)','direct_all(&store,2,40,41,','direct_all(&store,2,40,42,','Public first_duplicate_wildcard')
 error('late_failure_atomic',second+'store.space.objects[8].value=Value::Integer {number:4};',fault='Pin',row=1)

 setup=baseline()+'store.space.object_count=37;store.space.objects[0].value=Value::Package {first:1,count:32};store.space.objects[1].has_next=true;store.space.objects[1].next=6;'
 for object in range(6,37):setup+=f'store.space.objects[{object}]=Object {{value:Value::Package {{first:2,count:4}},has_next:{str(object<36).lower()},next:{object+1 if object<36 else 0}}};'
 add('capacity_32_shared_members',setup,'direct(&store,32,40,0)','direct(&store,32,40,','direct(&store,32,41,','Public capacity32; canonical captured rows share initialized field-chain identity')
 error('address_reference_not_unwrapped','store.space.object_count=7;store.space.objects[2].value=Value::Reference {kind:ReferenceKind::Named,object_id:6};store.space.objects[6].value=Value::Integer {number:262143};',fault='Address')
 # Canonical paths and declared scopes. Link input retains the source index.
 for label,absolute,parents,local,expect_count in [('simple',False,0,False,1),('nearest',False,0,True,2),('absolute',True,0,True,1),('parent',False,1,False,1)]:
  prefix=f'let name:Path=path({str(absolute).lower()},{parents},0x414b4e4c,0,1);let scope:Path=path(true,0,0x30494350,0,1);'
  setup=prefix+baseline(source='Value::NameReference {name:name,scope:scope}',index=1)+f'store=levels(store,{str(local).lower()});'
  expected='path(true,0,0x30494350,0x414b4e4c,2)'if expect_count==2 else'path(true,0,0x414b4e4c,0,1)'
  setup+=f'let expected:Path={expected};';add('link_'+label,setup,'link_check(&store,expected,1)','link_check(&store,expected,1)','link_check(&store,expected,2)','Public name path observations; parent resolution is deliberate correction')
 prefix='let name:Path=path(false,0,0x414b4e4c,0,1);let scope:Path=path(true,0,0x30494350,0,1);'
 error('missing_link_level',prefix+'store.space.objects[4].value=Value::NameReference {name:name,scope:scope};',outcome='MissingLevel',fault='LinkLevel')

 for label,name,scope,outcome in [('above_root','path(false,2,0x414b4e4c,0,1)','path(true,0,0x30494350,0,1)','AboveRoot'),('relative_scope','path(false,0,0x414b4e4c,0,1)','path(false,0,0x30494350,0,1)','NotAbsolute'),('invalid_name','path(false,0,1,0,1)','path(true,0,0x30494350,0,1)','InvalidName')]:
  patch=f'let name:Path={name};let scope:Path={scope};store=levels(store,false);store.space.objects[4].value=Value::NameReference {{name:name,scope:scope}};'
  error('link_'+label,patch,outcome=outcome,fault='LinkLevel')
 # Strict captured scope must exist as a level before ancestor search.
 for label,patch in [('absent','store.space.entries[1].has_level=false;'),('object_only','store.space.entries[1].has_level=false;store.space.entries[1].has_object=true;store.space.entries[1].object=0;')]:
  setup='let name:Path=path(false,0,0x414b4e4c,0,1);let scope:Path=path(true,0,0x30494350,0,1);'+baseline(source='Value::NameReference {name:name,scope:scope}')+'store=levels(store,false);'+patch
  add('declared_scope_'+label,setup,'failure(&store,Outcome::MissingLevel,Fault::LinkLevel,0,64)','Outcome::MissingLevel','Outcome::Success','Strict capture validation: missing declared level cannot borrow existing root LNKA')
 # Public selectors also reject fabricated malformed ordinary tables.
 for label,patch in [('none_target','table.count=1;'),('pin','table.count=1;table.entries[0]=Route {pin:4,target:Target::Gsi {number:40}};'),('count','table.count=33;')]:
  add('invalid_table_'+label,'let mut table:Routes=Routes {};'+patch+'let selected:Selection=routes::select(&table,0,0);','selected.outcome==Outcome::InvalidState','selected.outcome==Outcome::InvalidState','selected.outcome==Outcome::Success')
 add('invalid_later_table','let mut table:Routes=Routes {};table.count=2;table.entries[0]=Route {device:3,target:Target::Gsi {number:40}};let selected:Selection=routes::select(&table,3,0);','selected.outcome==Outcome::InvalidState','selected.outcome==Outcome::InvalidState','selected.outcome==Outcome::Success')
 add('no_entry',baseline()+'let result:Decode=routes::decode(&store,0,64);let selected:Selection=routes::select(&result.routes,4,0);','selected.outcome==Outcome::MissingObject','selected.outcome==Outcome::MissingObject','selected.outcome==Outcome::Success')
 # Resource bytes match corresponding retained original host fixtures.
 irq=bytes([0x23,0,4,0x18]);end=b'\x79\0';extended=b'\x89\x06\0\x0d\x01'+(0x12345678).to_bytes(4,'little')
 resource_cases=[('single',irq+end,0,10),('second_irq',bytes([0x23,0,2,0x18])+irq+end,1,10),('vendor_before',b'\x71\xaa'+irq+end,1,10),('dma_before',b'\x2a\x01\0'+irq+end,1,10),('extended',extended+end,0,0x12345678),('selected_vendor',b'\x71\xaa'+irq+end,0,None),('index_missing',irq+end,1,None),('no_end',irq,0,None),('bad_checksum',irq+b'\x79\x01',0,None),('trailing',irq+end+b'\xff',0,None),('two_numbers',b'\x89\x0a\0\x0d\x02'+(9).to_bytes(4,'little')+(10).to_bytes(4,'little')+end,0,None),('two_mask_bits',bytes([0x23,0,6,0x18])+end,0,None),('empty',b'',0,None),('truncated',b'\x23\x01',0,None)]
 for label,data,index,expected in resource_cases:
  setup='let mut input:[u8;4096];input[4095]=199;'+''.join(f'input[{n}]={b};'for n,b in enumerate(data))+f'let result:ResourceSelection=link_resources::select_bytes(&input,{len(data)},{index});'
  if expected is not None:add('resource_'+label,setup,f'resource_good(result,{expected},{len(data)}) && all_bytes(&result.bytes,&input,{len(data)},0,4096,true)',f'resource_good(result,{expected},',f'resource_good(result,{expected+1},','Detached bytes also exercised by public host resource observations')
  else:
   fault='Template'if label in ['no_end','bad_checksum','trailing','empty','truncated']else'Descriptor';outcome='MissingObject'if label=='index_missing'else'BadEncoding'
   add('resource_'+label,setup,f'resource_bad(result,Outcome::{outcome},Fault::{fault})','Outcome::'+outcome,'Outcome::Success')
 # Canonical source and owned byte representations both feed the same selector.
 for owned in [False,True]:
  data=irq+end;setup='let mut input:[u8;1024];let mut store:ObjectStore=ObjectStore {};store.space.object_count=1;'
  if owned:
   setup+='store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:0}};store.bytes.blocks[0]=ByteBlock {initialized:true,length:6};'+''.join(f'store.bytes.blocks[0].bytes[{n}]={b};'for n,b in enumerate(data))
  else:setup+='store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:6,buffer_initializer:Span {unit:1,start:0,end:6}}};'+''.join(f'input[{n}]={b};'for n,b in enumerate(data))
  setup+='let mut expected:[u8;4096];'+''.join(f'expected[{n}]={b};'for n,b in enumerate(data))+'let result:ResourceSelection=link_resources::select_buffer(&input,6,1,&store,0,0);'
  add('buffer_'+('owned'if owned else'source'),setup,'resource_good(result,10,6) && all_bytes(&result.bytes,&expected,6,0,4096,true)','resource_good(result,10,','resource_good(result,11,')

 for label,storage,byte in [('wrong_owner','BufferStorage::Owned {buffer_owner:1}','InvalidState'),('wrong_unit','BufferStorage::Source {declared_size:0,buffer_initializer:Span {unit:2,start:0,end:0}}','Bounds'),('too_large','BufferStorage::Source {declared_size:257,buffer_initializer:Span {unit:1,start:0,end:0}}','Capacity')]:
  setup='let input:[u8;1024];let mut store:ObjectStore=ObjectStore {};store.space.object_count=1;'+f'store.space.objects[0].value=Value::Buffer {{buffer_storage:{storage}}};let result:ResourceSelection=link_resources::select_buffer(&input,0,1,&store,0,0);'
  add('buffer_'+label,setup,f'resource_bad(result,Outcome::InvalidState,Fault::Buffer) && result.byte_outcome==ByteOutcome::{byte}',f'ByteOutcome::{byte}','ByteOutcome::Success')
 return rows

def render(selected=None):
 rows=cases();selected=rows if selected is None else [r for r in rows if r['name']in selected];source=HEAD;selections=[]
 for r in selected:
  for control in [False,True]:
   name='PrtSuite::'+r['name']+('_control'if control else'_positive');source+='machine '+name+'(&mut self)->i32 {\n'+r['control'if control else'body']+'}\n';selections.append(name+'='+str(int(control)))
 return source,selections
if __name__=='__main__':
 rows=cases();source,selections=render();(HERE/'suite.omg').write_text(source);(HERE/'cases.json').write_text(json.dumps(rows,indent=2)+'\n');(HERE/'selections.json').write_text(json.dumps(selections,indent=2)+'\n');print(len(rows),'positive/control pairs')
