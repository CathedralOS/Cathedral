#!/usr/bin/env python3
"""Bounded preloaded-data execution witnesses; primary-profile pin differences explicit."""
import importlib.util,json
import decoder_fixtures
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
spec=importlib.util.spec_from_file_location('integer_fixtures',HERE.parent/'execution/fixtures.py');old=importlib.util.module_from_spec(spec);spec.loader.exec_module(old)
integer,pkg,ret,op,name=old.integer,old.pkg,old.ret,old.op,old.name

def cases():
 rows=[]
 def add(label,body,kind=1,number=0,raw=b'',globals=None,methods=None,after=None,count=None,setup=None,mutate_source=False,bits=64,error='Success',note=''):
  table=b''.join(b'\x08'+name(k)+v for k,v in (globals or{}).items())
  table+=pkg(0x14,name('MAIN')+b'\0'+body)
  table+=b''.join(pkg(0x14,name(k)+bytes([flags])+code)for k,(flags,code)in(methods or{}).items())
  rows.append(dict(name=label,table=table.hex(),kind=kind,number=number,bytes=raw.hex(),after=after or{},object_count=count,error=error,note=note,setup=setup or [],mutate_source=mutate_source,bits=bits))
 buf=lambda b:pkg(0x11,integer(len(b))+b)
 string=lambda b:b'\x0d'+b+b'\0'
 package=lambda *v:pkg(0x12,bytes([len(v)])+b''.join(v))
 for typ,encoded,kind,raw in [('buffer',buf(b'\x11\x22\x33'),2,b'\x11\x22\x33'),('string',string(b'ABC'),3,b'ABC'),('empty_buffer',buf(b''),2,b''),('empty_string',string(b''),3,b'')]:
  add('return_'+typ,ret(name('X')),kind,raw=raw,globals={'X':encoded},count=2)
  add('copy_local_'+typ,op(0x9d,name('X'),b'\x60')+ret(b'\x60'),kind,raw=raw,globals={'X':encoded},count=3)
  add('call_return_'+typ,ret(name('CAL')+name('X')),kind,raw=raw,globals={'X':encoded},methods={'CAL':(1,ret(b'\x68'))},count=3)
  add('copy_named_'+typ,op(0x9d,name('X'),name('Y'))+ret(name('Y')),kind,raw=raw,globals={'X':encoded,'Y':integer(0)},count=3)
 add('package_return',ret(name('X')),4,number=2,globals={'X':package(integer(11),integer(22))},count=4)
 add('package_copy_local',op(0x9d,name('X'),b'\x60')+ret(b'\x60'),4,number=2,globals={'X':package(integer(11),integer(22))},count=5)
 add('package_copy_named',op(0x9d,name('X'),name('Y'))+ret(name('Y')),4,number=2,globals={'X':package(integer(11),integer(22)),'Y':integer(0)},count=5,note='Shallow child identities retained; not a general deep package clone claim.')
 for opcode,label in [(0x70,'store'),(0x9d,'copy')]:
  add(label+'_argument_plain_named',ret(name('CAL')+name('X')),number=22,globals={'X':integer(11)},methods={'CAL':(1,op(opcode,integer(22),b'\x68')+ret(b'\x68'))},after={'X':11},count=3,note='Primary call-by-reference-constant. Public pin changes X to22.')
  add(label+'_argument_plain_local',op(0x70,integer(11),b'\x60')+name('CAL')+b'\x60'+ret(b'\x60'),number=11,methods={'CAL':(1,op(opcode,integer(22),b'\x68')+ret(b'\x68'))},count=2,note='Primary caller binding isolation; public pin returns22.')
  add(label+'_argument_buffer_replace',ret(name('CAL')+name('X')),number=22,globals={'X':buf(b'\x11')},methods={'CAL':(1,op(opcode,integer(22),b'\x68')+ret(b'\x68'))},count=3)
 add('copy_local_byte_then_integer',op(0x9d,name('X'),b'\x60')+op(0x9d,integer(22),b'\x60')+ret(b'\x60'),number=22,globals={'X':buf(b'\x11')},count=3)
 add('copy_local_byte_reuses_cell',op(0x9d,name('X'),b'\x60')+op(0x9d,name('Y'),b'\x60')+ret(b'\x60'),2,raw=b'\x44\x55',globals={'X':buf(b'\x11'),'Y':buf(b'\x44\x55')},count=4)
 add('copy_owned_buffer_independent',op(0x9d,name('X'),b'\x60')+ret(b'\x60'),2,raw=b'\x11\x22',globals={'X':buf(b'\x11\x22')},count=3,setup=['let materialized:ByteResult=materialize(&program.source,program.length,program.unit,&mut program.store,0);'],mutate_source=True)
 add('call_buffer_shared_payload',ret(name('CAL')+name('X')),2,raw=b'\x44\x22',globals={'X':buf(b'\x11\x22')},methods={'CAL':(1,ret(b'\x68'))},count=3,mutate_source=True)
 add('copy_named_self',op(0x9d,name('X'),name('X'))+ret(name('X')),2,raw=b'\x11\x22',globals={'X':buf(b'\x11\x22')},count=2)
 add('late_named_operand',ret(op(0x72,name('X'),name('CAL'),b'\0')),number=23,globals={'X':integer(11)},methods={'CAL':(0,op(0x70,integer(22),name('X'))+ret(integer(1)))},after={'X':22},count=3,note='Retain captured operand identity through retirement; public pin returns23.')
 add('package_local_payload_independence',op(0x9d,name('X'),b'\x60')+op(0x9d,integer(99),name('X'))+ret(b'\x60'),4,number=2,globals={'X':package(integer(11),integer(22))},after={'X':99},count=5)
 add('reference_local_payload_independence',op(0x9d,name('R'),b'\x60')+op(0x9d,integer(99),name('R'))+ret(b'\x60'),5,number=0,globals={'X':integer(11),'R':integer(0)},after={'R':99},count=4,setup=['program.store.space.objects[1].value=Value::Reference {kind:ReferenceKind::RefOf,object_id:0};'])
 add('capacity_copy_package_atomic',op(0x9d,name('X'),b'\x60')+ret(b'\x60'),globals={'X':package(integer(11))},setup=['program.store.space.object_count=64;'],count=64,error='Capacity')
 add('copy_method_destination',op(0x9d,integer(22),name('CAL'))+ret(integer(0)),methods={'CAL':(0,ret(integer(11)))},error='UnsupportedValue')
 for opcode,label in [(0x70,'store'),(0x9d,'copy')]:
  add(label+'_explicit_reference_argument',ret(name('CAL')+name('R')),number=22,globals={'X':integer(11),'R':integer(0)},methods={'CAL':(1,op(opcode,integer(22),b'\x68')+ret(b'\x68'))},after={'X':22},count=4,setup=['program.store.space.objects[1].value=Value::Reference {kind:ReferenceKind::RefOf,object_id:0};'],note='Canonical Reference supplied through owned program metadata; RefOf bytecode dispatch is not claimed.')
  add(label+'_reference_argument_type_replacement',ret(name('CAL')+name('R')),3,raw=b'AB',globals={'X':integer(11),'R':integer(0),'S':string(b'AB')},methods={'CAL':(1,op(opcode,name('S'),b'\x68')+ret(b'\x68'))},count=5,setup=['program.store.space.objects[1].value=Value::Reference {kind:ReferenceKind::RefOf,object_id:0};'],note='Primary Table19.10: reference Arg redirects a copy with no conversion; referenced Integer is replaced by String. Canonical reference metadata is seeded.')
  add(label+'_local_reference_replaced',op(0x9d,name('R'),b'\x60')+op(opcode,integer(22),b'\x60')+ret(b'\x60'),number=22,globals={'X':integer(11),'R':integer(0)},after={'X':11},count=4,setup=['program.store.space.objects[1].value=Value::Reference {kind:ReferenceKind::RefOf,object_id:0};'])
 add('capacity_copy_byte_atomic',op(0x9d,name('X'),b'\x60')+ret(b'\x60'),globals={'X':buf(b'\x11')},setup=['program.store.space.object_count=64;'],count=64,error='Capacity')
 add('source_unit_bad_byte',ret(name('X')),globals={'X':buf(b'\x11')},setup=['program.store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:1,buffer_initializer:Span {unit:8,start:0,end:1}}};'],error='InvalidState')
 add('owned_wrong_owner',ret(name('X')),globals={'X':buf(b'\x11')},setup=['program.store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:1}};'],error='InvalidState')
 add('package_bad_link',ret(name('X')),globals={'X':package(integer(11),integer(22))},setup=['program.store.space.objects[1].has_next=false;'],error='InvalidState')
 add('return_named_width32_carrier',ret(name('X')),number=0xffffffff,globals={'X':integer((1<<64)-1)},count=2,bits=32)
 add('copy_named_width32_carrier',op(0x9d,name('X'),name('Y'))+ret(name('Y')),number=0xffffffff,globals={'X':integer((1<<64)-1),'Y':integer(0)},count=3,bits=32)
 add('argument_width32_carrier',ret(name('CAL')+integer((1<<64)-1)),number=0xffffffff,methods={'CAL':(1,ret(b'\x68'))},count=2,bits=32)
 add('buffer_arithmetic_rejected',ret(op(0x72,name('X'),integer(1),b'\0')),globals={'X':buf(b'\x11')},error='UnsupportedValue')
 return rows+decoder_fixtures.rows()

PREFIX='''// SPDX-License-Identifier: MIT OR Apache-2.0
use aml::model::Value;
use aml::model::Path;
use aml::model::Lookup;
use aml::model::ReferenceKind;
use aml::model::BufferStorage;
use aml::model::Span;
use aml::model::Outcome;
use aml::namespace::get;
use aml::byte_storage::ByteRead;
use aml::byte_storage::ByteResult;
use aml::byte_storage::ByteOutcome;
use aml::byte_storage::materialize;
use aml::byte_storage::create_buffer_field;
use aml::byte_storage::write_field_integer;
use execution::engine::ExecutionResult;
use execution::execution_model::ExecutionOutcome;
use execution::execution_model::Operand;
use execution::execution_model::ValueResult;
use execution::generic_values::read_value;
use execution::generic_values::byte_snapshot;
use pipeline::program::Program;
use pipeline::program::Prepared;
use pipeline::program::prepare_program;
use pipeline::program::run_program;
use integer_helpers::integers::IntegerSize;
data GenericCase [copy] {input:[u8;1024];length:u64;kind:u64;number:u64;bytes:[u8;256];byte_count:u64;has_after:bool;after:u64;after_segment:u32;check_count:bool;object_count:u64;error:ExecutionOutcome;patch:u64;mutate_source:bool;size:IntegerSize;}
machine ge_same_bytes(left:&[u8;256],right:&[u8;256],index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let good:bool=ge_byte(left,right,index,count);transition index<count {true -> ge_same_bytes(left,right,index+1,count,prior && good) _ -> (prior)}}
machine ge_byte(left:&[u8;256],right:&[u8;256],index:u64,count:u64)->bool {transition index<count && index<256 {true -> (left[index]==right[index]) _ -> (true)}}
machine ge_integer(value:Value,number:u64)->bool {transition value {Value::Integer {number as actual} -> (actual==number) _ -> (false)}}
machine ge_transport(value:Operand,kind:u64)->bool {transition value {Operand::Integer {number} -> (kind==1) Operand::Object {object_id} -> (kind>=2 && kind<=5) _ -> (false)}}
machine ge_kind(value:Value,kind:u64,number:u64)->bool {transition value {Value::Integer {number as actual} -> (kind==1 && actual==number) Value::Buffer {buffer_storage} -> (kind==2) Value::String {string_storage} -> (kind==3) Value::Package {first,count} -> (kind==4 && count==number) Value::Reference {kind as reference_kind,object_id} -> (kind==5 && reference_kind==ReferenceKind::RefOf && object_id==number) _ -> (false)}}
machine ge_check(row:GenericCase)->i32 {
 let loaded:Prepared=prepare_program(row.input,row.length,7,128,128);let mut program:Program=loaded.program;
 _=ge_setup(&mut program,row.patch);
 let mut path:Path=Path {absolute:true,count:1};path.segments[0]=1313423693;
 let arguments:[Value;7];let result:ExecutionResult=run_program(&mut program,path,&arguments,0,row.size,256);
 let count:bool=!row.check_count || program.store.space.object_count==row.object_count;
 let mutation:bool=ge_mutate_source(&mut program,row.mutate_source);
 let found:ValueResult=read_value(&program.store,result.operand);
 let matched:bool=ge_kind(found.value,row.kind,row.number);let transport:bool=ge_transport(result.operand,row.kind);
 let bytes:bool=ge_check_bytes(&program,result.operand,row);
 let after:bool=ge_after(&program,row);
 let good:bool=ge_result(loaded.outcome,result.outcome,row.error,result.has_value,matched && transport,bytes,after,count,mutation);
 transition good {true -> (0) _ -> (1)}
}
machine ge_result(loaded:Outcome,actual:ExecutionOutcome,expected:ExecutionOutcome,present:bool,matched:bool,bytes:bool,after:bool,count:bool,mutation:bool)->bool {loaded==Outcome::Success && actual==expected && after && count && mutation && (expected!=ExecutionOutcome::Success || (present && matched && bytes))}
machine ge_check_bytes(program:&Program,operand:Operand,row:GenericCase)->bool {
 transition row.kind==2 || row.kind==3 {true -> bytes(program,operand,row) _ -> (true)}
 state bytes(program:&Program,operand:Operand,row:GenericCase)->bool {let value:ValueResult=read_value(&program.store,operand);let snapshot:ByteRead=byte_snapshot(&program.source,program.length,program.unit,&program.store,value);let equal:bool=ge_same_bytes(&snapshot.bytes,&row.bytes,0,row.byte_count,true);value.outcome==ExecutionOutcome::Success && snapshot.outcome==ByteOutcome::Success && snapshot.length==row.byte_count && equal}
}
machine ge_after(program:&Program,row:GenericCase)->bool {
 transition row.has_after {true -> lookup(program,row) _ -> (true)}
 state lookup(program:&Program,row:GenericCase)->bool {let mut path:Path=Path {absolute:true,count:1};path.segments[0]=row.after_segment;let found:Lookup=get(program.store.space,path);transition found.outcome==Outcome::Success && found.object<64 {true -> value(program,found.object,row.after) _ -> (false)}}
 state value(program:&Program,id:u64,number:u64)->bool {let good:bool=ge_integer(program.store.space.objects[id].value,number);good}
}
machine ge_mutate_source(program:&mut Program,change:bool)->bool {
 transition change {true -> field(program) _ -> (true)}
 state field(program:&mut Program)->bool {let field:ByteResult=create_buffer_field(&program.source,program.length,program.unit,&mut program.store,0,0,8);transition field.outcome==ByteOutcome::Success {true -> write(program,field.object) _ -> (false)}}
 state write(program:&mut Program,id:u64)->bool {let written:ByteResult=write_field_integer(&program.source,program.length,program.unit,&mut program.store,id,IntegerSize::EightBytes,68);written.outcome==ByteOutcome::Success}
}
data Suite {}
'''
def render(rows):
 out=[PREFIX,decoder_fixtures.SOURCE];names=[]
 setup=['machine ge_setup(program:&mut Program,patch:u64)->u8 {transition patch {']
 for i,row in enumerate(rows):
  if row.get('setup'):setup.append(f'{i+1} -> patch_{i+1}(program)')
 setup.append('_ -> (0)}')
 for i,row in enumerate(rows):
  if row.get('setup'):setup.append(f'state patch_{i+1}(program:&mut Program)->u8 {{'+''.join(row['setup'])+'0}')
 setup.append('}')
 out.append('\n'.join(setup))
 for row_index,row in enumerate(rows):
  if 'direct' in row:
   for control in [False,True]:
    body,key=decoder_fixtures.render(row,control);out.append(body);names.append(key)
   continue
  for control in [False,True]:
   key=row['name']+('_control'if control else'_positive');names.append('Suite::'+key+'='+str(int(control)))
   out +=[f'machine Suite::{key}(&mut self)->i32 {{',f'let mut row:GenericCase=GenericCase {{length:{len(bytes.fromhex(row["table"]))},kind:{row["kind"]},number:{row["number"]},byte_count:{len(bytes.fromhex(row["bytes"]))},has_after:{str(bool(row["after"])).lower()},after:{next(iter(row["after"].values()),0)},after_segment:{int.from_bytes(name(next(iter(row["after"]),"X")),"little")},check_count:{str(row["object_count"]is not None).lower()},object_count:{row["object_count"]or 0},error:ExecutionOutcome::{row["error"]},patch:{row_index+1},mutate_source:{str(row["mutate_source"]).lower()},size:IntegerSize::{"FourBytes"if row["bits"]==32 else"EightBytes"}}};']
   out +=[f'row.input[{i}]={v};'for i,v in enumerate(bytes.fromhex(row['table']))if v]
   out +=[f'row.bytes[{i}]={v};'for i,v in enumerate(bytes.fromhex(row['bytes']))if v]
   if control:
    if row['error']!='Success':out +=['row.error=ExecutionOutcome::Success;']
    elif row['kind'] in [1,4,5]:out +=[f'row.number={row["number"]+1};']
    elif row['bytes']:out +=[f'row.bytes[0]={bytes.fromhex(row["bytes"])[0]^1};']
    else:out +=['row.byte_count=1;']
   out +=['let result:i32=ge_check(row);result','}']
 return '\n'.join(out)+'\n',names
if __name__=='__main__':
 rows=cases();(HERE/'cases.json').write_text(json.dumps(rows,indent=2)+'\n');source,names=render(rows);(HERE/'main.omg').write_text(source);(HERE/'selections.json').write_text(json.dumps(names));print(len(rows),'generic cases')
