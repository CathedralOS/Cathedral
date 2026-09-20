#!/usr/bin/env python3
"""Actual AML bytecode scenarios for the bounded integer execution profile."""
import argparse,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
def integer(value):
 if value==0:return b'\x00'
 if value==1:return b'\x01'
 if value==(1<<64)-1:return b'\xff'
 width=1 if value<=255 else 2 if value<=65535 else 4 if value<=0xffffffff else 8
 return bytes([{1:10,2:11,4:12,8:14}[width]])+value.to_bytes(width,'little')
def pkg(op,payload):
 for width in range(1,5):
  length=len(payload)+width
  if length<(1<<(6 if width==1 else 4+8*(width-1))):
   if width==1:encoded=bytes([length])
   else:encoded=bytes([((width-1)<<6)|(length&15)])+(length>>4).to_bytes(width-1,'little')
   return bytes([op])+encoded+payload
 raise ValueError('package too large')
def ret(value):return b'\xa4'+value
def op(code,*args):return (code.to_bytes(2,'big')if code>255 else bytes([code]))+b''.join(args)
def name(s):return s.encode('ascii').ljust(4,b'_')
def cases():
 rows=[]
 def add(label,code,expected=None,error='Success',globals=None,methods=None,flags=0,args=(),after=None,budget=128,origin='Original synthetic AML; primary ACPI6.6 chapters19/20',bits=32,aliases=None):
  rows.append(dict(name=label,main=code.hex(),expected=expected,error=error,globals=globals or{},methods={k:{'flags':v[0],'bytes':v[1].hex()}for k,v in(methods or{}).items()},flags=flags,args=list(args),after=after or{},budget=budget,origin=origin,bits=bits,aliases=aliases or{}))
 add('return_add',ret(op(0x72,integer(40),integer(2),b'\0')),42,budget=16)
 add('return_literal',ret(integer(42)),42,budget=8)
 add('empty_method',b'',None,budget=4)
 add('logical_not',ret(op(0x92,integer(1))),0,origin='tests/logical_not.asl::MAIN complete method body')
 add('local_store',op(0x70,integer(17),b'\x60')+ret(b'\x60'),17)
 add('argument_store',op(0x70,integer(17),b'\x68')+ret(b'\x68'),17,flags=1,args=(9,))
 add('argument_copy',op(0x9d,integer(31),b'\x6e')+ret(b'\x6e'),31,flags=7,args=(1,2,3,4,5,6,7))
 add('named_store',op(0x70,integer(42),name('X'))+ret(name('X')),42,globals={'X':5},after={'X':42})
 add('named_alias_store',op(0x70,integer(42),name('ALIS'))+ret(name('X')),42,globals={'X':5},aliases={'ALIS':'X'},after={'X':42})
 add('incdec',op(0x75,name('X'))*3+op(0x70,name('X'),name('Y'))+op(0x76,name('X'))+ret(name('X')),2,globals={'X':0,'Y':0},after={'X':2,'Y':3},origin='tests/incdec.asl statement body wrapped in synthetic MAIN; Name declarations seeded in namespace')
 for truth in [0,1]:
  code=pkg(0xa0,integer(truth)+ret(integer(11)))+pkg(0xa1,ret(integer(22)))+ret(integer(33))
  add('if_else_'+str(truth),code,11 if truth else 22)
  add('if_no_else_'+str(truth),pkg(0xa0,integer(truth)+ret(integer(11)))+ret(integer(33)),11 if truth else 33)
 for x in [0,5]:
  foo=pkg(0xa0,op(0x94,name('X'),integer(1))+b'\xa3')+pkg(0xa1,ret(integer(0x55)))+ret(integer(0x3f))
  add('method_'+str(x),op(0x70,name('FOO'),name('Y'))+ret(name('Y')),0x3f if x>1 else 0x55,globals={'X':x,'Y':0},methods={'FOO':(0,foo)},after={'Y':0x3f if x>1 else 0x55},origin='tests/method.asl complete FOO body and invocation/store, wrapped top-level tail; x=0 is additional else coverage')
 add('call_argument',ret(name('FOO')+integer(41)),42,methods={'FOO':(1,ret(op(0x72,b'\x68',integer(1),b'\0')))})
 add('call_no_return_statement',name('FOO')+ret(integer(42)),42,methods={'FOO':(0,b'\xa3')})
 add('call_no_return_operand',ret(name('FOO')),error='Uninitialized',methods={'FOO':(0,b'\xa3')})
 add('while_increment',pkg(0xa2,op(0x95,name('X'),integer(5))+op(0x75,name('X')))+ret(name('X')),5,globals={'X':0},after={'X':5},origin='tests/while.asl first While body; declarations seeded and final read explicit')
 loop=op(0x95,name('Y'),integer(10))+pkg(0xa0,op(0x9295,name('Y'),integer(5))+b'\xa5')+op(0x75,name('Y'))
 add('while_break',pkg(0xa2,loop)+ret(name('Y')),5,globals={'Y':0},after={'Y':5},budget=256,origin='tests/while.asl DefBreak scenario')
 loop=op(0x95,name('CNT'),integer(5))+op(0x75,name('CNT'))+b'\x9f'+op(0x75,name('Z'))
 add('while_continue',pkg(0xa2,loop)+ret(name('Z')),0,globals={'CNT':0,'Z':0},after={'CNT':5,'Z':0},budget=192,origin='tests/while.asl DefContinue scenario')
 add('while_decrement_predicate',op(0x70,integer(5),b'\x60')+pkg(0xa2,op(0x76,b'\x60')+b'\x9f')+ret(b'\x60'),0,origin='tests/while.asl decrement-in-predicate scenario')
 add('divide_targets',op(0x78,integer(23),integer(5),b'\x60',b'\x61')+ret(op(0x72,op(0x77,b'\x60',integer(10),b'\0'),b'\x61',b'\0')),34)
 for code,label in [(0x80,'not'),(0x81,'highest'),(0x82,'lowest')]:add('unary_'+label,ret(op(code,integer(0x80),b'\0')),{0x80:0xffffff7f,0x81:8,0x82:8}[code])
 add('zero_division',ret(op(0x78,integer(1),integer(0),b'\0',b'\0')),error='DivideByZero')
 add('uninitialized_local',ret(b'\x60'),error='Uninitialized')
 add('uninitialized_argument',ret(b'\x68'),error='Uninitialized')
 add('argument_count',ret(integer(1)),error='ArgumentCount',flags=1)
 add('serialized',ret(integer(1)),error='UnresolvedSynchronization',flags=8)
 for code,label in [(b'\x5b\x33','timer'),(b'\x5b\x22','sleep'),(b'\xcc','breakpoint')]:add('unresolved_'+label,code,error='UnresolvedService')
 add('debug_store',op(0x70,integer(1),b'\x5b\x31'),error='UnresolvedService')
 add('copy_null',op(0x9d,integer(1),b'\0'),error='InvalidTarget')
 add('store_null',op(0x70,integer(1),b'\0'),error='InvalidTarget')
 add('break_outside',b'\xa5',error='ControlOutsideLoop')
 add('continue_outside',b'\x9f',error='ControlOutsideLoop')
 add('orphan_else',pkg(0xa1,b'\xa3'),error='BadEncoding')
 add('truncated_constant',ret(b'\x0c\x12'),error='Truncated')
 add('truncated_expression',ret(b'\x72\x01'),error='Truncated')
 add('malformed_package',b'\xa0\x3f\x01',error='BadEncoding')
 add('unsupported_buffer',ret(pkg(0x11,integer(1)+b'\x00')),error='UnsupportedOpcode')
 add('region_read',ret(name('REGN')),error='UnresolvedRegion',globals={'REGN':{'kind':'region'}})
 add('missing_name',ret(name('MISS')),error='MissingObject')
 add('noop_operand',ret(b'\xa3'),error='BadEncoding')
 add('over_budget',ret(integer(1)),error='Capacity',budget=1025)
 nested=ret(integer(42))
 for i in range(9):nested=pkg(0xa0,integer(1)+nested)
 add('block_depth',nested,error='Depth')
 add('wrap64',ret(op(0x72,integer((1<<64)-1),integer(1),b'\0')),0,bits=64)
 add('zero_budget',ret(integer(42)),error='WorkLimit',budget=0)
 add('loop_budget',pkg(0xa2,integer(1)+b'\xa3'),error='WorkLimit',budget=16)
 add('recursive_depth',ret(name('MAIN')),error='Depth',budget=64)
 deep=integer(1)
 for i in range(17):deep=op(0x92,deep)
 add('expression_depth',ret(deep),error='Depth')
 add('qword_literal',ret(integer(0x100000000)),0x100000000,bits=64)
 add('from_bcd',ret(op(0x5b28,integer(0x1234),b'\0')),1234)
 add('to_bcd',ret(op(0x5b29,integer(1234),b'\0')),0x1234)
 add('invalid_bcd',ret(op(0x5b28,integer(0xa),b'\0')),error='BadEncoding')
 add('region_store',op(0x70,integer(1),name('REGN')),error='UnresolvedRegion',globals={'REGN':{'kind':'region'}})
 add('store_then_limit',op(0x70,integer(42),name('X'))+pkg(0xa2,integer(1)+b'\xa3'),error='WorkLimit',budget=16,globals={'X':0},after={'X':42})
 for label,patch,error in [('source_unit',{'span_unit':8},'SourceUnit'),('source_max_start',{'span_start':(1<<64)-1},'Truncated'),('source_max_end',{'span_end':(1<<64)-1},'Truncated'),('source_length',{'length':1},'Truncated'),('source_inverted',{'span_start':2,'span_end':1},'Truncated'),('source_capacity',{'length':1025},'Capacity'),('namespace_capacity',{'entry_count':33},'Capacity'),('object_capacity',{'object_count':65},'Capacity')]:
  add(label,ret(integer(42)),error=error);rows[-1]['profile_patch']=patch
 # Cross-scope aliases retain the method definition observation, independent
 # of current namespace lookup paths and original-name rebinding/removal.
 setup=[
  'let mut d0: Path = Path { absolute: true, count: 1 };',
  f'd0.segments[0] = {int.from_bytes(name("D0"),"little")};',
  'let mut d1: Path = Path { absolute: true, count: 1 };',
  f'd1.segments[0] = {int.from_bytes(name("D1"),"little")};',
  'let mut original: Path = d0; original.count = 2;',
  f'original.segments[1] = {int.from_bytes(name("FOO"),"little")};',
  'space.entries[2].path = original; definitions[1].scope = original;',
  'let mut x0: Path = d0; x0.count = 2;',
  f'x0.segments[1] = {int.from_bytes(name("X"),"little")};',
  'space.entries[3].path = x0;',
  'let mut x1: Path = d1; x1.count = 2;',
  f'x1.segments[1] = {int.from_bytes(name("X"),"little")};',
  'space.entries[4].path = x1;',
  'let mut alias: Path = d1; alias.count = 2;',
  f'alias.segments[1] = {int.from_bytes(name("ALIS"),"little")};',
  'space.entries[5].path = alias;',
  'space.entries[6] = Entry { path: d0, has_level: true };',
  'space.entries[7] = Entry { path: d1, has_level: true }; space.count = 8;']
 for label,tail,error in [
  ('method_alias_scope',[],'Success'),
  ('method_alias_rebound',['space.objects[4] = Object { value: Value::Integer { number: 99 } };','space.object_count = 5; space.entries[2].object = 4;'],'Success'),
  ('method_alias_removed',['space.entries[2].has_object = false;'],'Success'),
  ('method_bound_removed',['space.entries[2].has_object = false; space.entries[5].alias = false;'],'Success'),
  ('method_alias_unobserved',['definitions[1].present = false;'],'MethodDefinition'),
  ('method_bound_unobserved',['space.entries[2].has_object = false; space.entries[5].alias = false; definitions[1].present = false;'],'MethodDefinition')]:
  add(label,ret(b'\\\x2e'+name('D1')+name('ALIS')),11 if error=='Success'else None,error=error,methods={'FOO':(0,ret(name('X')))},globals={'X':11,'Y':22},aliases={'ALIS':'FOO'})
  rows[-1]['omega_setup']=setup+tail
 for label,assignment in [('definition_missing','definitions[0].present = false;'),('definition_identity','definitions[0].object_id = 1;'),('definition_span','definitions[0].body.end = 0;'),('definition_flags','definitions[0].flags = 1;')]:
  add(label,ret(integer(42)),error='MethodDefinition');rows[-1]['omega_setup']=[assignment]
 return rows

def render(row,evaluate=True):
 patch=row.get('profile_patch',{})
 methods={'MAIN':{'flags':row['flags'],'bytes':row['main']},**row['methods']};data=bytearray();spans={}
 for key,value in methods.items():start=len(data);data.extend(bytes.fromhex(value['bytes']));spans[key]=(start,len(data))
 objects={key:i for i,key in enumerate([*methods,*row['globals']])}
 entries=[*objects,*row['aliases']]
 out=['// SPDX-License-Identifier: MIT OR Apache-2.0','// '+row['origin'],
 'use aml::model::Value;','use aml::model::Path;','use aml::model::Span;','use aml::model::Namespace;','use aml::model::Object;','use aml::model::Entry;',
 'use integer_helpers::integers::IntegerSize;','use execution::engine::run_method;','use execution::engine::ExecutionResult;','use execution::execution_model::ExecutionOutcome;', 'use execution::execution_model::MethodDefinition;',
 'machine namespace_integer(value: Value, expected: u64) -> bool { transition value { Value::Integer { number } -> (number == expected) _ -> (false) } }',
 'machine test_result() -> i32 {','    let mut input: [u8; 1024];']
 out +=[f'    input[{i}] = {v};'for i,v in enumerate(data)if v]
 out +=['    let mut definitions: [MethodDefinition; 64];','    let mut arguments: [Value; 7];']+[f'    arguments[{i}] = Value::Integer {{ number: {v} }};'for i,v in enumerate(row['args'])]
 out +=[f'    let mut space: Namespace = Namespace {{ count: {patch.get("entry_count",len(entries)+1)}, object_count: {patch.get("object_count",len(objects))} }};', '    space.entries[0] = Entry { path: Path { absolute: true }, has_level: true };']
 for i,key in enumerate(entries):
  packed=int.from_bytes(name(key),'little');out +=[f'    let mut path_{i}: Path = Path {{ absolute: true, count: 1 }};',f'    path_{i}.segments[0] = {packed};',f'    space.entries[{i+1}] = Entry {{ path: path_{i}, has_object: true, object: {objects[row["aliases"].get(key,key)]}, alias: {str(key in row["aliases"]).lower()} }};']
 for key,index in objects.items():
  if key in methods:
   a,b=spans[key];a=patch.get('span_start',a)if key=='MAIN'else a;b=patch.get('span_end',b)if key=='MAIN'else b;v=f'Value::Method {{ flags: {methods[key]["flags"]}, body: Span {{ unit: {patch.get("span_unit",7)}, start: {a}, end: {b} }} }}'
  elif isinstance(row['globals'][key],dict):v='Value::OperationRegion { space: 0, base: 4096, length: 8, scope: Path { absolute: true } }'
  else:v=f'Value::Integer {{ number: {row["globals"][key]} }}'
  out +=[f'    space.objects[{index}] = Object {{ value: {v} }};']
  if key in methods:
   out +=[f'    definitions[{index}] = MethodDefinition {{ present: true, object_id: {index}, scope: path_{entries.index(key)}, flags: {methods[key]["flags"]}, body: Span {{ unit: {patch.get("span_unit",7)}, start: {a}, end: {b} }} }};']
 for text in row.get('omega_setup',[]):out +=['    '+text]

 out +=[f'    let result: ExecutionResult = run_method(&input, {patch.get("length",len(data))}, 7, &mut space, &definitions, path_0, &arguments, {len(row["args"])}, IntegerSize::{"FourBytes"if row["bits"]==32 else"EightBytes"}, {row["budget"]});']
 checks=[f'result.outcome == ExecutionOutcome::{row["error"]}']
 if row['error']=='Success':checks +=[f'result.value.initialized == {str(row["expected"]is not None).lower()}']+([f'result.value.number == {row["expected"]}']if row['expected']is not None else[])
 for i,(key,value)in enumerate(row['after'].items()):
  out +=[f'    let changed_{i}: bool = namespace_integer(space.objects[{objects[key]}].value, {value});'];checks +=[f'changed_{i}']
 out +=['    transition '+' && '.join(checks)+' { true -> (0) _ -> (1) }','}']
 if evaluate:out +=['const TEST_RESULT: i32 = test_result();','machine require_success(value: i32) requires value == 0; {}','data Main {}','machine Main::main(&mut self) { require_success(TEST_RESULT); }']
 else:out +=['data Main {}','machine Main::main(&mut self) {}']
 return '\n'.join(out)+'\n'
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--check',action='store_true');args=p.parse_args();rows=cases();outputs={HERE/'cases.json':json.dumps({'format':'cathedral-acpi-execution-cases-v1','cases':rows},indent=2)+'\n',HERE/'main.omg':render(rows[0])}
 for path,text in outputs.items():
  if args.check:
   if not path.exists()or path.read_text()!=text:raise SystemExit('Fixture differs: '+str(path))
  else:path.write_text(text)
 print(len(rows),'bytecode cases '+('verified'if args.check else'generated'))
if __name__=='__main__':main()
