#!/usr/bin/env python3
"""Original AML SizeOf and unchanged ObjectType scenarios through the actual owned Program."""
import argparse
import importlib.util
import json
from pathlib import Path

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[4]
MAX=(1<<64)-1
_spec=importlib.util.spec_from_file_location('query_opcode_aml',HERE.parent/'execution/fixtures.py')
encoding=importlib.util.module_from_spec(_spec);_spec.loader.exec_module(encoding)
integer,pkg,ret,op,name=encoding.integer,encoding.pkg,encoding.ret,encoding.op,encoding.name


def namestring(text):
    lead=b''
    if text.startswith('\\'):lead=b'\\';text=text[1:]
    while text.startswith('^'):lead+=b'^';text=text[1:]
    if not text:return lead+b'\0'
    parts=text.split('.')
    return lead+(b'' if len(parts)==1 else b'.' if len(parts)==2 else b'/'+bytes([len(parts)]))+b''.join(name(part) for part in parts)


def method(label,body,flags=0):return pkg(0x14,namestring(label)+bytes([flags])+body)
def named(label,value):return b'\x08'+namestring(label)+value
def string(text):return b'\x0d'+text.encode()+b'\0'
def buffer(data,size=None):return pkg(0x11,integer(len(data) if size is None else size)+bytes(data))
def package(*items):return pkg(0x12,bytes([len(items)])+b''.join(items))
def scope(label,body=b''):return pkg(0x10,namestring(label)+body)
def extended(code,payload):return b'\x5b'+pkg(code,payload)
def region():return b'\x5b\x80REG0\0'+integer(0x1000)+integer(16)
def field():return b'\x5b'+pkg(0x81,b'REG0\x11FLD0\x08')
def array(data):return 'let mut input:[u8;1024];\n'+''.join(f'input[{i}]={v};\n' for i,v in enumerate(data) if v)
def path(variable,text):
    parts=text.lstrip('\\').split('.')
    return f'let mut {variable}:Path=Path {{absolute:true,count:{len(parts)}}};'+''.join(f'{variable}.segments[{i}]={int.from_bytes(name(part),"little")};' for i,part in enumerate(parts))+'\n'

IMPORTS='''// SPDX-License-Identifier: MIT OR Apache-2.0
// Original initialized AML; actual Program preparation and execution.
use aml::model::Outcome;
use aml::model::Value;
use aml::model::Object;
use aml::model::ObjectStore;
use aml::model::ReferenceKind;
use aml::model::Path;
use aml::model::Span;
use aml::model::Entry;
use aml::model::LevelKind;
use aml::model::NamespaceResult;
use aml::model::StringStorage;
use aml::model::BufferStorage;
use aml::model::ByteBlock;
use aml::namespace::add_level;
use pipeline::program::Program;
use pipeline::program::Prepared;
use pipeline::program::prepare_program;
use pipeline::program::run_program;
use execution::engine::ExecutionResult;
use execution::execution_model::ExecutionOutcome;
use execution::execution_model::Frame;
use execution::execution_model::Binding;
use execution::execution_model::Operation;
use execution::execution_model::Operand;
use execution::decode_execution::decode_term;
use aml::model::FieldBinding;
use integer_helpers::integers::IntegerSize;
'''
HELPERS='''machine qe_integer(value:Value,n:u64)->bool {transition value {Value::Integer {number} -> (number==n) _ -> (false)}}
machine qe_field(value:Value,n:u64)->bool {transition value {Value::FieldUnit {binding,declaration,field} -> field(binding,field.bit_offset,field.bit_length,n) _ -> (false)} state field(binding:FieldBinding,offset:u64,length:u64,n:u64)->bool {transition binding {FieldBinding::Region {region_object} -> (region_object==n && offset==0 && length==8) _ -> (false)}}}
machine qe_operand(value:Operand,n:u64)->bool {transition value {Operand::Integer {number} -> (number==n) _ -> (false)}}
'''


def cases():
    rows=[]
    def add(label,prefix,body,expected=None,error='Success',patch='',after='',extra='',flags=0,args=(),budget=128,unit=7,bits=64,public=True,public_after=None,note=''):
        data=prefix+method('MAIN',body,flags)
        source=array(data)+f'let prepared:Prepared=prepare_program(input,{len(data)},{unit},128,128);let mut program:Program=prepared.program;\n'+patch+path('main_path','MAIN')+'let mut arguments:[Value;7];'
        source+=''.join(f'arguments[{i}]={value};' for i,value in enumerate(args))
        source+=f'let result:ExecutionResult=run_program(&mut program,main_path,&arguments,{len(args)},IntegerSize::{"EightBytes" if bits==64 else "FourBytes"},{budget});\n'+after
        check=f'prepared.outcome==Outcome::Success && result.outcome==ExecutionOutcome::{error}'
        if expected is not None:
            check+=f' && result.has_value && result.value.initialized && result.value.number=={expected}'
            old=f'result.value.number=={expected}';new=f'result.value.number=={expected^1}'
        else:
            check+=' && !result.has_value';old=f'result.outcome==ExecutionOutcome::{error}';new='result.outcome==ExecutionOutcome::Success'
        if extra:check+=' && '+extra
        assert check.count(old)==1
        rows.append(dict(name=label,body=source,check=check,mutation=[old,new],aml_hex=data.hex(),expected=expected,error=error,
                         public=public and not patch and not args and budget==128,public_after=public_after or {},revision=2 if bits==64 else 1,note=note))
    for label,value,typ,size in [('integer',integer(42),1,None),('string',string('ABC'),2,3),('buffer',buffer([1,2]),3,2),('padded_buffer',buffer([1,2],5),3,5),('empty_string',string(''),2,0),('empty_buffer',buffer([]),3,0),('package',package(integer(1),integer(2)),4,2),('empty_package',package(),4,0),('package_uninitialized',pkg(0x12,b'\x03'),4,3)]:
        prefix=named('OBJ0',value)
        add('type_'+label,prefix,ret(op(0x8e,b'OBJ0')),typ)
        add('size_'+label,prefix,ret(op(0x87,b'OBJ0')),size,error='Success' if size is not None else 'UnsupportedValue')
    for label,prefix,typ in [('device',extended(0x82,b'OBJ0'),6),('event',b'\x5b\x02OBJ0',7),('mutex',b'\x5b\x01OBJ0\0',9),('power',extended(0x84,b'OBJ0\0\0\0'),11),('processor',extended(0x83,b'OBJ0'+b'\0'*6),12),('thermal',extended(0x85,b'OBJ0'),13),('region',region(),10)]:
        target=b'REG0' if label=='region' else b'OBJ0'
        add('type_'+label,prefix,ret(op(0x8e,target)),typ)
        add('size_'+label,prefix,ret(op(0x87,target)),error='UnsupportedValue')
    for opcode,label,expected,error in [(0x8e,'type',8,'Success'),(0x87,'size',None,'UnsupportedValue')]:
        prefix=named('CNT0',integer(0))+method('SIDE',op(0x70,integer(99),b'CNT0')+ret(integer(5)),7)
        add(label+'_method_no_invocation',prefix,ret(op(opcode,b'SIDE')),expected,error,
            after='let untouched:bool=qe_integer(program.store.space.objects[0].value,0);',extra='untouched',public_after={'CNT0':'integer:0'})
    for opcode,label,expected,error in [(0x8e,'type',8,'Success'),(0x87,'size',None,'UnsupportedValue')]:
        prefix=named('CNT0',integer(0))+method('SIDE',op(0x70,integer(99),b'CNT0')+ret(integer(5)))
        add(label+'_method_zero_arity_no_effect',prefix,ret(op(opcode,b'SIDE')),expected,error,
            after='let untouched:bool=qe_integer(program.store.space.objects[0].value,0);',extra='untouched',public_after={'CNT0':'integer:0'})
    prefix=region()+field()
    for opcode,label,expected,error in [(0x8e,'type',5,'Success'),(0x87,'size',None,'UnsupportedValue')]:
        add(label+'_installed_field',prefix,ret(op(opcode,b'FLD0')),expected,error,
            after='let unchanged:bool=qe_field(program.store.space.objects[1].value,0);',extra='unchanged')
    registers=b'\x5b'+pkg(0x81,b'REG0\x01SEL0\x08DTA0\x08')
    for kind,definition in [('bank',b'\x5b'+pkg(0x87,b'REG0SEL0'+integer(2)+b'\x01OBJ0\x08')),('index',b'\x5b'+pkg(0x86,b'SEL0DTA0\x01OBJ0\x08'))]:
        for opcode,label,expected,error in [(0x8e,'type',5,'Success'),(0x87,'size',None,'UnsupportedValue')]:
            add(label+'_installed_'+kind,region()+registers+definition,ret(op(opcode,b'OBJ0')),expected,error)
    for opcode,label,expected in [(0x8e,'type',16),(0x87,'size',None)]:
        add(label+'_debug',b'',ret(op(opcode,b'\x5b\x31')),expected,error='Success' if expected is not None else 'UnsupportedValue')
    for opcode,label,expected in [(0x8e,'type',0),(0x87,'size',None)]:
        for target,kind in [(b'\x60','local0'),(b'\x67','local7'),(b'\x68','arg0'),(b'\x6e','arg6')]:
            add(label+'_uninitialized_'+kind,b'',ret(op(opcode,target)),expected,error='Success' if expected is not None else 'UnsupportedValue')
    for opcode,label,expected in [(0x8e,'type',1),(0x87,'size',None)]:
        add(label+'_inline_local',b'',op(0x70,integer(42),b'\x60')+ret(op(opcode,b'\x60')),expected,error='Success' if expected is not None else 'UnsupportedValue')
        add(label+'_inline_arg',b'',ret(op(opcode,b'\x68')),expected,error='Success' if expected is not None else 'UnsupportedValue',flags=1,args=('Value::Integer {number:42}',))
    for opcode,label,expected in [(0x8e,'type',2),(0x87,'size',3)]:
        add(label+'_owned_local',named('OBJ0',string('ABC')),op(0x70,b'OBJ0',b'\x60')+ret(op(opcode,b'\x60')),expected)
        add(label+'_alias',named('OBJ0',string('ABC'))+b'\x06OBJ0ALIS',ret(op(opcode,b'ALIS')),expected)
    add('type_created_scope',scope('SCP0'),ret(op(0x8e,b'SCP0')),0)
    add('size_scope',scope('SCP0'),ret(op(0x87,b'SCP0')),error='UnsupportedValue')
    add('type_root',b'',ret(op(0x8e,b'\\\0')),0)
    add('type_predefined_scope',b'',ret(op(0x8e,b'\\_SB_')),0,
        patch=path('sb','_SB')+'let added:NamespaceResult=add_level(program.store.space,sb,LevelKind::Scope);program.store.space=added.space;')
    prefix=named('OBJ0',integer(4))+scope('D0',scope('OBJ0')+method('TEST',ret(op(0x8e,b'OBJ0'))))
    add('nearest_scope_wins',prefix,ret(namestring('\\D0.TEST')),0)
    prefix=named('OBJ0',integer(4))+scope('D0',named('OBJ0',string('AB'))+method('TEST',ret(op(0x8e,b'OBJ0'))))
    add('nearest_object_wins',prefix,ret(namestring('\\D0.TEST')),2)
    prefix=named('OBJ0',buffer([1,2,3]))+scope('D0',method('TEST',ret(op(0x87,namestring('^^OBJ0')))))
    add('parent_name_lookup',prefix,ret(namestring('\\D0.TEST')),3)
    add('nested_query_arithmetic',named('OBJ0',string('ABC')),ret(op(0x72,op(0x8e,b'OBJ0'),op(0x87,b'OBJ0'),b'\0')),5)
    add('standalone_query',named('OBJ0',string('ABC')),op(0x8e,b'OBJ0')+ret(integer(42)),42)
    add('query_result_store',named('OUT0',integer(0))+named('OBJ0',buffer([1,2,3])),op(0x70,op(0x87,b'OBJ0'),b'OUT0')+ret(b'OUT0'),3,
        after='let stored:bool=qe_integer(program.store.space.objects[0].value,3);',extra='stored',public_after={'OUT0':'integer:3'})
    for opcode,label in [(0x8e,'type'),(0x87,'size')]:
        add(label+'_missing',b'',ret(op(opcode,b'MISS')),error='MissingObject')
        add(label+'_truncated',b'',ret(op(opcode)),error='Truncated')
        add(label+'_truncated_name',b'',ret(op(opcode,b'AB')),error='Truncated')
        add(label+'_null_target',b'',ret(op(opcode,b'\0')),error='InvalidTarget',public=False)
        add(label+'_literal_operand',b'',ret(op(opcode,integer(42))),error='UnsupportedOpcode',public=False)
        for nested,suffix in [(0x71,'ref_of'),(0x83,'deref_of'),(0x88,'index')]:
            add(label+'_unsupported_'+suffix,named('OBJ0',buffer([1,2,3])),ret(op(opcode,op(nested,b'OBJ0'))),error='UnsupportedOpcode',public=False)
    add('query_does_not_execute_expression',named('CNT0',integer(0)),ret(op(0x8e,op(0x72,integer(1),integer(1),b'CNT0'))),error='UnsupportedOpcode',
        after='let untouched:bool=qe_integer(program.store.space.objects[0].value,0);',extra='untouched',public=False)
    add('earlier_effect_retained',named('CNT0',integer(0)),op(0x70,integer(42),b'CNT0')+ret(op(0x87,b'CNT0')),error='UnsupportedValue',
        after='let kept:bool=qe_integer(program.store.space.objects[0].value,42);',extra='kept',public_after={'CNT0':'integer:42'})
    # Ordinary public records can be malformed; metadata kind deliberately ignores unrelated storage/body details.
    for opcode,label,expected,error in [(0x8e,'type',3,'Success'),(0x87,'size',None,'InvalidState')]:
        add(label+'_malformed_owned',named('OBJ0',buffer([1])),ret(op(opcode,b'OBJ0')),expected,error,
            patch=f'program.store.space.objects[0].value=Value::Buffer {{buffer_storage:BufferStorage::Owned {{buffer_owner:{MAX}}}}};')
        add(label+'_wrong_source_unit',named('OBJ0',buffer([1])),ret(op(opcode,b'OBJ0')),expected,error,
            patch='program.store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:1,buffer_initializer:Span {unit:8,start:0,end:1}}};')
    add('type_malformed_method',method('SIDE',ret(integer(1))),ret(op(0x8e,b'SIDE')),8,
        patch=f'program.store.space.objects[0].value=Value::Method {{flags:255,body:Span {{unit:8,start:{MAX},end:0}}}};program.definitions.entries[0].present=false;')
    add('type_malformed_field',prefix=region()+field(),body=ret(op(0x8e,b'FLD0')),expected=5,
        patch=f'program.store.space.objects[1].value=Value::FieldUnit {{binding:FieldBinding::Region {{region_object:{MAX}}}}};')
    for opcode,label,expected,error in [(0x8e,'type',4,'Success'),(0x87,'size',None,'InvalidState')]:
        add(label+'_malformed_package',named('OBJ0',package(integer(1),integer(2))),ret(op(opcode,b'OBJ0')),expected,error,
            patch='program.store.space.objects[2].has_next=true;program.store.space.objects[2].next=1;')
    for opcode,label,expected,error in [(0x8e,'type',14,'Success'),(0x87,'size',None,'UnsupportedValue')]:
        add(label+'_buffer_field_metadata',named('OBJ0',integer(0)),ret(op(opcode,b'OBJ0')),expected,error,
            patch=f'program.store.space.objects[0].value=Value::BufferField {{backing_object:{MAX},bit_offset:{MAX},bit_length:{MAX}}};')
    for kind in ['RefOf','Index','Unresolved']:

        for opcode,label,expected in [(0x8e,'type',3),(0x87,'size',3)]:
            add(label+'_arg_reference_'+kind,named('OBJ0',buffer([1,2,3])),ret(op(opcode,b'\x68')),expected,
                flags=1,args=(f'Value::Reference {{kind:ReferenceKind::{kind},object_id:0}}',))
    for target_kind,prefix,target,typ in [('field',region()+field(),1,5),('method',method('SIDE',ret(integer(1))),0,8)]:
        add('type_reference_to_'+target_kind,prefix,ret(op(0x8e,b'\x68')),typ,flags=1,args=(f'Value::Reference {{kind:ReferenceKind::RefOf,object_id:{target}}}',))
    for opcode,label in [(0x8e,'type'),(0x87,'size')]:
        add(label+'_reference_cycle',named('OBJ0',integer(0)),ret(op(opcode,b'OBJ0')),error='ReferenceCycle',patch='program.store.space.objects[0].value=Value::Reference {kind:ReferenceKind::Named,object_id:0};')
        add(label+'_dangling_reference',named('OBJ0',integer(0)),ret(op(opcode,b'OBJ0')),error='InvalidState',patch=f'program.store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::Named,object_id:{MAX}}};')
    for budget,error in [(0,'WorkLimit'),(1,'WorkLimit'),(3,'WorkLimit'),(4,'Success'),(1025,'Capacity')]:
        add('query_budget_'+str(budget),named('OBJ0',integer(0)),ret(op(0x8e,b'OBJ0')),1 if error=='Success' else None,error,budget=budget,
            extra=f'result.steps=={budget if budget<4 else 4 if budget==4 else 0}')
    add('query_integer32',named('OBJ0',string('ABC')),ret(op(0x87,b'OBJ0')),3,bits=32)
    # SizeOf's real decoder commits only after inspection and contribution.
    def direct(label,data,patch='',error='Success',next_pc=2,count=1,value=7):
        body=array(data)+f'let mut store:ObjectStore=ObjectStore {{}};let mut frame:Frame=Frame {{end:{len(data)},source_length:{len(data)},body:Span {{unit:7,end:{len(data)}}},operation_count:1}};'
        body+='store.space.object_count=1;store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:0}};store.bytes.blocks[0]=ByteBlock {initialized:true,length:7};'
        body+='frame.locals[0]=Binding::Shared {shared_id:0};frame.operations[0]=Operation {opcode:0xa4,value_arity:1};frame.operations[0].operands[0]=Operand::Integer {number:99};'+patch
        body+='let result:ExecutionOutcome=decode_term(&input,&store,&mut frame);'
        body+=f'let operand_ok:bool=qe_operand(frame.operations[0].operands[0],{value});'
        check=f'result==ExecutionOutcome::{error} && frame.pc=={next_pc} && frame.operations[0].count=={count} && operand_ok && frame.cache_count==0'
        old=f'frame.pc=={next_pc}'
        rows.append(dict(name=label,body=body,check=check,mutation=[old,f'frame.pc=={next_pc^1}'],aml_hex=data.hex(),expected=None,error=error,public=False,public_after={},revision=2,note='Direct SizeOf decoder cursor/operand preservation.'))
    direct('size_contribution',b'\x87\x60')
    direct('size_contribution_full',b'\x87\x60',patch='frame.operations[0].count=1;',error='InvalidState',next_pc=0,count=1,value=99)
    direct('size_operation_stack_invalid',b'\x87\x60',patch='frame.operation_count=17;',error='InvalidState',next_pc=0,count=0,value=99)
    direct('size_selection_failure_cursor',b'\x87\x71',error='UnsupportedOpcode',next_pc=0,count=0,value=99)
    direct('size_truncated_cursor',b'\x87',error='Truncated',next_pc=0,count=0,value=99)
    deep='frame.scope=Path {absolute:true,count:16};'+''.join(f'frame.scope.segments[{i}]=0x5f5f4241;' for i in range(16))
    direct('size_path_capacity',b'\x87OBJ0',patch=deep,error='Capacity',next_pc=0,count=0,value=99)
    chain='store.space.object_count=64;'+''.join(f'store.space.objects[{i}].value=Value::Reference {{kind:ReferenceKind::RefOf,object_id:{i+1}}};' for i in range(63))+'store.space.objects[63].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:63}};store.bytes.blocks[63]=ByteBlock {initialized:true,length:7};'
    direct('size_exact64_reference_chain',b'\x87\x60',patch=chain)
    direct('size_64_reference_cycle',b'\x87\x60',patch=chain+'store.space.objects[63].value=Value::Reference {kind:ReferenceKind::RefOf,object_id:0};',error='ReferenceCycle',next_pc=0,count=0,value=99)
    assert len({row['name'] for row in rows})==len(rows)
    return rows


def render(row,control=False,entry=None):
    check=row['check'].replace(*row['mutation']) if control else row['check']
    head='machine test_result()' if entry is None else f'machine {entry}(&mut self)'
    return head+'->i32 {\n'+row['body']+'transition '+check+' {true -> (0) _ -> (1)}\n}\n'


def main():
    p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');args=p.parse_args()
    path=HERE/'cases.json';text=json.dumps(cases(),indent=2,sort_keys=True)+'\n'
    if args.check:assert path.read_text()==text
    else:path.write_text(text)
    print('PASS',len(cases()),'authored query opcode scenarios')


if __name__=='__main__':main()
