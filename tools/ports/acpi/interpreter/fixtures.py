#!/usr/bin/env python3
"""Pinned pure scenario translations and independent AML boundary expectations."""
import argparse,json,re
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[3]
IMPORTS={'integers':['IntegerSize','IntegerResult','Binary','Logical','from_revision','integer_bits','binary','logical','logical_not','bitwise_not','find_set_left','find_set_right'], 'bcd':['from_bcd','to_bcd'], 'buffer_fields':['copy_bits','field_to_integer'], 'conversions':['EmptyPolicy','buffer_to_integer','integer_to_buffer','mid','buffer_to_string','string_to_buffer']}
PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
def cases():
 rows=[]
 def add(name,body,checks,origin='Original boundary scenario; ACPI6.6 chapter19'):
  rows.append(dict(name=name,body=body,checks=checks,origin=origin))
 def array(name,data):return [f'let mut {name}: [u8; 256];']+[f'{name}[{i}] = {v};' for i,v in enumerate(data) if v]
 def size(bits):return 'IntegerSize::'+('FourBytes'if bits==32 else'EightBytes')
 for bits in [32,64]:
  mask=(1<<bits)-1;z=size(bits)
  for op,a,b,expected in [('Add',23,19,42),('Subtract',23,19,4),('Multiply',23,19,437),('Divide',23,19,1),('Mod',23,19,4),('And',23,19,19),('Nand',23,19,mask^19),('Or',23,19,23),('Nor',23,19,mask^23),('Xor',23,19,4),('ShiftLeft',23,2,92),('ShiftRight',23,2,5),('Add',mask,1,0),('Subtract',0,1,mask),('Multiply',mask,2,mask-1),('ShiftLeft',1,bits,0),('ShiftRight',mask,bits,0),('ShiftLeft',1,bits+1,0),('ShiftRight',mask,mask,0),('ShiftLeft',mask,1,mask-1)]:
   add(f'binary_{bits}_{op}_{a}_{b}',[f'let result: IntegerResult = binary({z}, Binary::{op}, {a}, {b});'],['result.error == 0',f'result.value == {expected}']+(['result.remainder == 4']if op=='Divide'else[]))
  for op in ['Divide','Mod']:
   add(f'zero_{bits}_{op}',[f'let result: IntegerResult = binary({z}, Binary::{op}, 12, 0);'],['result.error == 1'])
  for op,predicate in [('And',True),('Or',True),('Equal',False),('NotEqual',True),('Less',True),('LessEqual',True),('Greater',False),('GreaterEqual',False)]:
   add(f'logical_{bits}_{op}',[f'let result: u64 = logical({z}, Logical::{op}, 12, 42);'],[f'result == {mask if predicate else 0}'])
  for value in [0,1,2,0x81,1<<(bits-1),mask]:
   add(f'unary_{bits}_{value}',[f'let a: u64 = bitwise_not({z}, {value});',f'let b: u64 = logical_not({z}, {value});',f'let c: u64 = find_set_left({z}, {value});',f'let d: u64 = find_set_right({z}, {value});'],[f'a == {mask^value}',f'b == {mask if value==0 else 0}',f'c == {value.bit_length()}',f'd == {(value&-value).bit_length()}'])
  for value in [0,1,123,99999999 if bits==32 else 9999999999999999]:
   encoded=int(str(value),16)
   add(f'bcd_{bits}_{value}',[f'let a: IntegerResult = to_bcd({z}, {value});',f'let b: IntegerResult = from_bcd({z}, {encoded});'],['a.error == 0','b.error == 0',f'a.value == {encoded}',f'b.value == {value}'])
  add(f'bcd_overflow_{bits}',[f'let a: IntegerResult = to_bcd({z}, {100000000 if bits==32 else 10000000000000000});'],['a.error == 3'])
  for value in [10,0x1f,0xa<<28]:add(f'bcd_invalid_{bits}_{value}',[f'let a: IntegerResult = from_bcd({z}, {value});'],['a.error == 2'])
 for rev in [0,1,2,255]:add('revision_'+str(rev),[f'let value: IntegerSize = from_revision({rev});','let bits: u64 = integer_bits(value);'],[f'bits == {32 if rev<2 else 64}'],'src/aml/mod.rs::IntegerSize::from_revision')
 # Actual upstream Rust tests, replacing only allocation/reference wrappers.
 add('upstream_copy_bits',array('source',[0xbf,0xf7,0xff,0xff,0xff])+array('destination',[0xe1,0,0,0,0])+['let error: u8 = copy_bits(&source, 5, 0, &mut destination, 5, 2, 15);'],['error == 0']+[f'destination[{i}] == {v}'for i,v in enumerate([0xfd,0xde,1,0,0])],'src/aml/object.rs::tests::test_copy_bits')
 add('upstream_buffer_to_integer',array('source',[0xab,0xcd,0xef,1,0xff])+['let result: IntegerResult = buffer_to_integer(IntegerSize::FourBytes, &source, 5, EmptyPolicy::PinnedZero);'],['result.error == 0','result.value == 32492971'],'src/aml/object.rs::tests::buffer_to_integer')
 for name,data,start,count,bits,expected in [('integer',[255]*5,5,9,32,511),('4_byte_integer',[15,0,0,0,240],4,36,32,0),('8_byte_integer',[15,0,0,0,240,255],4,36,64,0xf00000000)]:
  add('upstream_buffer_field_to_'+name,array('source',data)+[f'let result: IntegerResult = field_to_integer({size(bits)}, &source, {len(data)}, {start}, {count});'],['result.error == 0',f'result.value == {expected}'],'src/aml/object.rs::tests::buffer_field_to_'+name)
 for bits,data,expected in [(64,[0xde,0xad,0xbe,0xef],0xefbeadde),(64,[1],1),(64,[0],0),(64,[0xde,0xad,0xbe,0xef,0xca,0xfe,0xba,0xbe],0xbebafecaefbeadde)]:
  add('asl_to_integer_'+str(expected),array('source',data)+[f'let result: IntegerResult = buffer_to_integer({size(bits)}, &source, {len(data)}, EmptyPolicy::PinnedZero);'],['result.error == 0',f'result.value == {expected}'],'tests/to_integer.asl (buffer expression only; method harness not executed)')
 for bits in [32,64]:
  value=0xdeadbeefcafebabe;expected=value.to_bytes(8,'little')[:bits//8]
  add('integer_to_buffer_'+str(bits),array('output',[99]*10)+[f'let result: IntegerResult = integer_to_buffer({size(bits)}, {value}, &mut output);'],['result.error == 0',f'result.value == {bits//8}']+[f'output[{i}] == {v}'for i,v in enumerate(expected)]+[f'output[{bits//8}] == 99'],'tests/to_x.asl::ToBuffer integer expression plus original width/tail checks')
 for empty in ['Reject','PinnedZero']:
  add('empty_'+empty,array('source',[])+[f'let result: IntegerResult = buffer_to_integer(IntegerSize::EightBytes, &source, 0, EmptyPolicy::{empty});'],[f'result.error == {6 if empty=="Reject" else 0}','result.value == 0'])
 for source_start in [0,7,8,2047,2048,(1<<64)-1]:
  data=[0x81];dst=[255]*3;count=9;expected=int.from_bytes(dst,'little');bits=(int.from_bytes(data,'little')>>source_start)&((1<<count)-1)if source_start<8 else 0;expected=(expected&~(((1<<count)-1)<<3))|(bits<<3)
  add('copy_zero_extend_'+str(source_start),array('source',data)+array('output',dst)+[f'let error: u8 = copy_bits(&source, 1, {source_start}, &mut output, 3, 3, 9);'],['error == 0']+[f'output[{i}] == {v}'for i,v in enumerate(expected.to_bytes(3,'little'))])
 for sl,dl,start,count,error in [(257,1,0,1,4),(1,257,0,1,4),(1,1,8,1,5),(1,1,9,0,5),(1,1,0,(1<<64)-1,5),(1,1,8,0,0),(1,0,0,0,0)]:
  add(f'copy_bounds_{sl}_{dl}_{start}_{count}',array('source',[255])+array('output',[81])+[f'let error: u8 = copy_bits(&source, {sl}, 0, &mut output, {dl}, {start}, {count});'],[f'error == {error}','output[0] == 81'])
 for length,index,requested,expected in [(5,0,3,[1,2,3]),(5,3,10,[4,5]),(5,3,(1<<64)-1,[4,5]),(5,5,1,[]),(5,(1<<64)-1,1,[]),(0,0,1,[]),(5,1,0,[])]:
  add(f'mid_{length}_{index}_{requested}',array('source',[1,2,3,4,5])+array('output',[99]*6)+[f'let result: IntegerResult = mid(&source, {length}, {index}, {requested}, &mut output);'],['result.error == 0',f'result.value == {len(expected)}']+[f'output[{i}] == {v}'for i,v in enumerate(expected)]+[f'output[{len(expected)}] == 99'])
 for explicit in [True,False]:
  for data,error in [(b'Hello',0),(b'',0),(bytes([65,0,66]),7),(bytes([128]),7)]:
   expected=data+(b'\0'if explicit and data else b'')
   add(f'string_buffer_{explicit}_{data.hex() or "empty"}',array('source',data)+array('output',[99]*8)+[f'let result: IntegerResult = string_to_buffer(&source, {len(data)}, {str(explicit).lower()}, &mut output);'],[f'result.error == {error}']+([f'result.value == {len(expected)}']+[f'output[{i}] == {v}'for i,v in enumerate(expected)]+[f'output[{len(expected)}] == 99']if not error else ['output[0] == 99']),'tests/to_x.asl::ToBuffer String expression'if data==b'Hello'and explicit else'Original ASCII conversion boundary')
 for data,maximum,expected,error in [(b'AB\0Z',99,b'AB',0),(b'ABCD',2,b'AB',0),(b'',99,b'',0),(bytes([65,128]),99,b'',7),(bytes([0,128]),99,b'',0),(b'A',0,b'',0)]:
  add(f'buffer_string_{data.hex() or "empty"}_{maximum}',array('source',data)+array('output',[99]*6)+[f'let result: IntegerResult = buffer_to_string(&source, {len(data)}, {maximum}, &mut output);'],[f'result.error == {error}']+([f'result.value == {len(expected)}']+[f'output[{i}] == {v}'for i,v in enumerate(expected)]+[f'output[{len(expected)}] == 99']if not error else ['output[0] == 99']))
 for operation,call in [('mid','mid(&source, 257, 0, 1, &mut output)'),('string_to_buffer','string_to_buffer(&source, 257, false, &mut output)'),('buffer_to_string','buffer_to_string(&source, 257, 1, &mut output)')]:
  add('capacity_'+operation,array('source',[65])+array('output',[99])+['let result: IntegerResult = '+call+';'],['result.error == 4','output[0] == 99'])
 for operation,call in [('field','field_to_integer(IntegerSize::EightBytes, &source, 257, 0, 1)'),('integer','buffer_to_integer(IntegerSize::EightBytes, &source, 257, EmptyPolicy::PinnedZero)')]:
  add('capacity_'+operation,array('source',[65])+['let result: IntegerResult = '+call+';'],['result.error == 4'])
 add('capacity_terminator',array('source',[65]*256)+array('output',[99])+['let result: IntegerResult = string_to_buffer(&source, 256, true, &mut output);'],['result.error == 4','output[0] == 99'])
 add('last_byte_copy',array('source',[])+array('output',[])+['source[255] = 128;','output[255] = 1;','let error: u8 = copy_bits(&source, 256, 2047, &mut output, 256, 2047, 1);'],['error == 0','output[255] == 129','output[254] == 0'])
 add('last_bit_field',array('source',[])+['source[255] = 128;','let result: IntegerResult = field_to_integer(IntegerSize::EightBytes, &source, 256, 2047, 64);'],['result.error == 0','result.value == 1'])
 add('last_byte_mid',array('source',[])+array('output',[99,99])+['source[255] = 42;','let result: IntegerResult = mid(&source, 256, 255, 18446744073709551615, &mut output);'],['result.error == 0','result.value == 1','output[0] == 42','output[1] == 99'])
 add('asl_incdec',['let a: IntegerResult = binary(IntegerSize::FourBytes, Binary::Add, 0, 1);','let b: IntegerResult = binary(IntegerSize::FourBytes, Binary::Add, a.value, 1);','let c: IntegerResult = binary(IntegerSize::FourBytes, Binary::Add, b.value, 1);','let y: u64 = c.value;','let x: IntegerResult = binary(IntegerSize::FourBytes, Binary::Subtract, c.value, 1);'],['x.value == 2','y == 3'],'tests/incdec.asl (pure value chain only; namespace operations not executed)')
 add('asl_logical_not',['let result: u64 = logical_not(IntegerSize::FourBytes, 1);'],['result == 0'],'tests/logical_not.asl (returned expression only; method invocation not executed)')
 return rows

def render(rows,evaluate=True):
 out=['// SPDX-License-Identifier: MIT OR Apache-2.0','// Generated pinned pure-expression translations and original boundary tests.']
 for module,names in IMPORTS.items():out +=[f'use interpreter::{module}::{name};'for name in names]
 for row in rows:out += [f'machine test_{row["name"]}() -> bool {{']+['    '+line for line in row['body']]+['    '+' && '.join(row['checks']),'}']
 if evaluate:
  out += ['machine test_result() -> i32 { transition '+' && '.join('test_'+r['name']+'()'for r in rows)+' { true -> (0) _ -> (1) } }','const TEST_RESULT: i32 = test_result();','machine require_success(value: i32) requires value == 0; {}','data Main {}','machine Main::main(&mut self) { require_success(TEST_RESULT); }']
 else:out+=['data Main {}','machine Main::main(&mut self) {}']
 body='\n'.join(line for line in out if not line.startswith('use '))
 return '\n'.join(line for line in out if not line.startswith('use ')or re.search(r'\b'+line.rsplit('::',1)[1].rstrip(';')+r'\b',body))+'\n'
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--check',action='store_true');args=p.parse_args();rows=cases()
 for path,text in {HERE/'main.omg':render(rows,False),HERE/'cases.json':json.dumps({'format':'cathedral-acpi-interpreter-cases-v1','pin':PIN,'scope':'Pure-expression tests only; no AML opcode execution or method/namespace effects.','cases':rows},indent=2)+'\n'}.items():
  if args.check:
   if not path.exists()or path.read_text()!=text:raise SystemExit('Fixture differs: '+str(path))
  else:path.write_text(text)
 print(len(rows),'interpreter helper cases '+('verified'if args.check else'generated'))
if __name__=='__main__':main()
