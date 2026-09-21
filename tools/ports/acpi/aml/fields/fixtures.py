#!/usr/bin/env python3
"""Original field-declaration byte fixtures; no external firmware assets."""
import json
from pathlib import Path
HERE=Path(__file__).resolve().parent
COMMON='''// SPDX-License-Identifier: MIT OR Apache-2.0
// Original Cathedral semantic fixture; no firmware or external example content.
use aml::model::Outcome;
use aml::model::Path;
use aml::model::Span;
use aml::model::Read;
use aml::field_model::Access;
use aml::field_model::AccessType;
use aml::field_model::UpdateRule;
use aml::field_model::AccessResult;
use aml::field_model::Connection;
use aml::field_model::ListState;
use aml::field_model::DeclarationKind;
use aml::field_model::DeclarationResult;
use aml::field_model::Pending;
'''
TAIL='''
const TEST_RESULT:u64=test();
machine require_ok(result:u64) requires result==0; {}
data Main {}
machine Main::main(&mut self){require_ok(TEST_RESULT);}
'''
CASES={}
def pkg(n):
 if n<64:return bytes([n])
 for count in range(1,4):
  if n<(1<<(4+8*count)):
   return bytes([(count<<6)|(n&15)])+(n>>4).to_bytes(count,'little')
 raise ValueError(n)
def envelope(body):
 for width in range(1,5):
  size=pkg(len(body)+width)
  if len(size)==width:return size+body
 raise ValueError(len(body))
def wire(op,body):return bytes([0x5b,op])+envelope(body)
def array(data):return 'let mut input:[u8;1024];\n'+''.join(f'input[{i}]={b};\n' for i,b in enumerate(data))
def fixture(name,imports,body,condition,mutation,helpers='',reason=''):
 source=COMMON+imports+'\n'+helpers+'\nmachine test()->u64 {\n'+body+'\n transition '+condition+' {true -> (0) _ -> (1)}\n}\n'+TAIL
 assert source.count(mutation[0])==1,name
 (HERE/'cases'/f'{name}.omg').write_text(source)
 CASES[name]={'mutation':mutation,'behavior':reason or condition}
L='use fields::field_list::parse_list;'
D='use fields::declarations::parse_declaration;'
F='use fields::flags::decode_flags;\nuse fields::flags::change_access;\nuse fields::flags::nominal_width;'
def lists(data,flags=0,limit=1024,budget=32):
 return array(data)+f'let scope:Path=Path {{absolute:true}};let result:ListState=parse_list(&input,{len(data)},0,9,scope,{flags},{limit},{budget});'
def declaration(data):
 return array(data)+f'let scope:Path=Path {{absolute:true}};let result:DeclarationResult=parse_declaration(&input,{len(data)},0,7,scope,1024,32);'
fixture('flags',F,'''let good:AccessResult=decode_flags(0x52);let invalid_access:AccessResult=decode_flags(6);let invalid_update:AccessResult=decode_flags(0x60);let reserved:AccessResult=decode_flags(0x80);let width:Read=nominal_width(4);''','good.outcome==Outcome::Success && good.access.kind==AccessType::Word && good.access.locked && good.access.update==UpdateRule::WriteAsZeros && invalid_access.outcome==Outcome::BadEncoding && invalid_update.outcome==Outcome::BadEncoding && reserved.outcome==Outcome::BadEncoding && width.value==8',('width.value==8','width.value==7'))
fixture('access-attributes',F,'''let initial:AccessResult=decode_flags(0x52);let ordinary:AccessResult=change_access(initial.access,0x85,9,false,0);let extended:AccessResult=change_access(ordinary.access,5,14,true,6);let bad:AccessResult=change_access(initial.access,0x12,0,false,0);let bad_extended:AccessResult=change_access(initial.access,5,12,true,1);''','ordinary.outcome==Outcome::Success && ordinary.access.attribute_mode==2 && ordinary.access.attribute==9 && ordinary.access.flags==0x55 && ordinary.access.locked && extended.access.extended && extended.access.access_length==6 && extended.access.attribute==14 && bad.outcome==Outcome::BadEncoding && bad_extended.outcome==Outcome::BadEncoding',('extended.access.access_length==6','extended.access.access_length==5'))
data=b'AAAA'+pkg(8)+bytes([0])+pkg(3)+bytes([1,0x44,7])+b'BBBB'+pkg(16)+bytes([3,5,14,6])+b'CCCC'+pkg(0)
fixture('list-offsets-access',L,lists(data,0x12),'result.outcome==Outcome::Success && result.count==3 && result.bit_offset==27 && result.fields[0].bit_offset==0 && result.fields[1].bit_offset==11 && result.fields[1].access.kind==AccessType::QWord && result.fields[1].access.attribute_mode==1 && result.fields[2].access.extended && result.fields[2].access.access_length==6 && result.fields[2].access.locked && result.fields[2].bit_length==0',('result.bit_offset==27','result.bit_offset==26'))
data=bytes([2])+b'^CON0'+b'LINK'+pkg(8)+bytes([2,0x11])+envelope(bytes([0x0a,2,0xaa,0xbb]))+b'BUF0'+pkg(16)
helpers='''machine named(connection:Connection)->bool {transition connection {Connection::Name {connection_name} -> check(connection_name) _ -> (false)} state check(path:Path)->bool {path.parents==1 && path.count==1 && path.segments[0]==0x304e4f43}}
machine buffered(connection:Connection)->bool {transition connection {Connection::Buffer {buffer_encoding,size_known,declared_size,initializer} -> check(buffer_encoding,size_known,declared_size,initializer) _ -> (false)} state check(span:Span,known:bool,size:u64,bytes:Span)->bool {span.unit==9 && known && size==2 && bytes.start==16 && bytes.end==18}}
'''
fixture('connections',L,lists(data)+'let one:bool=named(result.fields[0].connection);let two:bool=buffered(result.fields[1].connection);','result.outcome==Outcome::Success && result.count==2 && result.bit_offset==24 && one && two',('result.bit_offset==24','result.bit_offset==23'),helpers)
data=bytes([2,0x11])+envelope(bytes([0x60,0xab]))+b'DYN0'+pkg(8)
helpers='''machine deferred(connection:Connection)->bool {transition connection {Connection::Buffer {buffer_encoding,size_known,declared_size,initializer} -> check(buffer_encoding,size_known,initializer) _ -> (false)} state check(span:Span,known:bool,bytes:Span)->bool {!known && span.start==1 && span.end==5 && bytes.end==0}}
'''
fixture('connection-deferred-buffer',L,lists(data)+'let pending:bool=deferred(result.fields[0].connection);','result.outcome==Outcome::Success && result.count==1 && result.at==10 && pending',('result.at==10','result.at==9'),helpers)
data=b'\x5cBAD0'+pkg(8)
fixture('named-field-rejects-path',L,lists(data),'result.outcome==Outcome::InvalidName && result.count==0',('result.count==0','result.count==1'))
data=b'AAAA'+pkg(8)+b'BBBB'+pkg(9)
fixture('bit-limit',L,lists(data,limit=16),'result.outcome==Outcome::Capacity && result.count==1 && result.bit_offset==8',('result.bit_offset==8','result.bit_offset==7'))
data=b''.join(f'A{i:03d}'.encode()+pkg(1) for i in range(33))
fixture('field-capacity',L,lists(data,budget=40),'result.outcome==Outcome::Capacity && result.count==32 && result.bit_offset==32',('result.count==32','result.count==31'))
data=b'AAAA'+pkg(8)+b'BBBB'+pkg(8)
fixture('field-work-limit',L,lists(data,budget=1),'result.outcome==Outcome::WorkLimit && result.count==1 && result.at==5',('result.at==5','result.at==4'))
data=bytes([0,0xc1,0xff,0xff,0xff])+b'WIDE'+pkg(1)
fixture('large-bit-offset',L,lists(data,limit=0xffffffffffffffff),'result.outcome==Outcome::Success && result.fields[0].bit_offset==0xffffff1 && result.bit_offset==0xffffff2',('result.bit_offset==0xffffff2','result.bit_offset==0xffffff1'))
data=bytes([1,2])
fixture('truncated-access',L,lists(data),'result.outcome==Outcome::Truncated && result.count==0',('result.count==0','result.count==1'))
data=wire(0x81,b'REG0'+bytes([0x31])+b'F000'+pkg(5)+bytes([0])+pkg(3)+b'F001'+pkg(8))
fixture('field-declaration',D,declaration(data),f'result.outcome==Outcome::Success && result.next=={len(data)} && result.declaration.kind==DeclarationKind::Field && result.declaration.primary_name.segments[0]==0x30474552 && result.list.count==2 && result.list.fields[1].bit_offset==8 && result.list.fields[0].access.update==UpdateRule::WriteAsOnes',('result.list.count==2','result.list.count==1'))
data=wire(0x86,b'IDX0DAT0'+bytes([2])+b'IND0'+pkg(16))
fixture('index-declaration',D,declaration(data),'result.outcome==Outcome::Success && result.declaration.kind==DeclarationKind::Index && result.declaration.secondary_name.segments[0]==0x30544144 && result.list.count==1 && result.list.fields[0].bit_length==16',('result.list.fields[0].bit_length==16','result.list.fields[0].bit_length==15'))
data=wire(0x87,b'REG0BNK0'+bytes([0x0b,0x34,0x12,1])+b'BNK1'+pkg(8))
fixture('bank-declaration',D,declaration(data),'result.outcome==Outcome::Success && result.declaration.kind==DeclarationKind::Bank && result.declaration.bank_value==0x1234 && result.list.count==1',('result.declaration.bank_value==0x1234','result.declaration.bank_value==0x1233'))
data=wire(0x87,b'REG0BNK0'+bytes([0x72,0x01,0x01,0x00,1])+b'BNK1'+pkg(8))
helpers='''machine pending_bank(value:Pending)->bool {transition value {Pending::BankTermArgRemainder {unparsed} -> check(unparsed) _ -> (false)} state check(span:Span)->bool {span.unit==7 && span.start==11 && span.end==21}}
'''
fixture('bank-dynamic-pending',D,declaration(data)+'let pending:bool=pending_bank(result.pending);','result.outcome==Outcome::UnsupportedSyntax && result.next==11 && result.list.count==0 && result.declaration.field_list.end==0 && pending',('result.next==11','result.next==12'),helpers)
data=bytes([0x5b,0x81,2,ord('A')])
fixture('declaration-short-envelope',D,declaration(data),'result.outcome==Outcome::Truncated && result.list.count==0',('result.list.count==0','result.list.count==1'))
fixture('empty-list',L,lists(b'',budget=0),'result.outcome==Outcome::Success && result.done && result.count==0 && result.at==0',('result.count==0','result.count==1'))
fixture('invalid-flags-empty',L,lists(b'',flags=0x60,budget=0),'result.outcome==Outcome::BadEncoding && result.count==0',('result.count==0','result.count==1'))
(HERE/'cases.json').write_text(json.dumps(CASES,indent=2,sort_keys=True)+'\n')
print(len(CASES),'field cases written')
