#!/usr/bin/env python3
"""Original initialized TermLists for declaration-time capture and execution."""
import argparse
import json
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[3]

def name(value): return value.encode('ascii').ljust(4,b'_')
def absolute(*segments):
    return b'\\'+(b'' if len(segments)==1 else b'\x2e' if len(segments)==2 else b'\x2f'+bytes([len(segments)]))+b''.join(name(v)for v in segments)
def integer(value):
    if value<2:return bytes([value])
    return b'\x0a'+bytes([value])
def package(op,payload):
    for width in range(1,5):
        length=len(payload)+width
        if length<(64 if width==1 else 1<<(4+8*(width-1))):
            encoded=bytes([length])if width==1 else bytes([((width-1)<<6)|(length&15)])+(length>>4).to_bytes(width-1,'little')
            return bytes([op])+encoded+payload
    raise ValueError('oversized fixture')
def method(label,body,flags=0):return package(0x14,name(label)+bytes([flags])+body)
def named(label,value):return b'\x08'+name(label)+integer(value)
def scope(label,body):return package(0x10,name(label)+body)
def returned(value):return b'\xa4'+value
def array(data,var='input'):
    return f'let mut {var}: [u8;1024];\n'+''.join(f'{var}[{i}] = {v};\n'for i,v in enumerate(data)if v)
def path(var,*segments):
    return f'let mut {var}: Path = Path {{ absolute:true,count:{len(segments)} }};\n'+''.join(f'{var}.segments[{i}] = {int.from_bytes(name(v),"little")};\n'for i,v in enumerate(segments))
IMPORTS='''// SPDX-License-Identifier: MIT OR Apache-2.0
// Original Cathedral synthetic AML; no firmware or external sample input.
use aml::model::Outcome;
use aml::model::Path;
use aml::model::Span;
use aml::model::Value;
use aml::model::Namespace;
use aml::model::ObjectStore;
use aml::model::NamespaceResult;
use aml::model::Lookup;
use aml::model::MethodDefinition;
use aml::model::MethodDefinitions;
use aml::model::ObservedLoad;
use aml::namespace::empty;
use aml::namespace::get;
use aml::namespace::bind;
use aml::loader::load_with_definitions;
use pipeline::program::Program;
use pipeline::program::Prepared;
use pipeline::program::prepare_program;
use pipeline::program::run_program;
use execution::engine::ExecutionResult;
use execution::engine::run_method;
use execution::execution_model::ExecutionOutcome;
use integer_helpers::integers::IntegerSize;
'''

def cases():
    rows=[]
    def add(label,body,check,old,new,description):
        assert check.count(old)==1,(label,old)
        if ',empty(),'in body:body='let initial_space:Namespace=empty();\n'+body.replace(',empty(),',',initial_space,')
        rows.append(dict(name=label,body=body,check=check,mutation=[old,new],description=description))
    def program_case(label,data,expected=42,error='Success',after='',before='',description='',unit=7,term_budget=64):
        body=array(data)+f'let prepared: Prepared = prepare_program(input,{len(data)},{unit},{term_budget},128);\nlet mut program: Program = prepared.program;\n'+before+path('target_path','MAIN')+'let arguments: [Value;7];\nlet result: ExecutionResult = run_program(&mut program,target_path,&arguments,0,IntegerSize::FourBytes,256);\n'+after
        check=f'prepared.outcome == Outcome::Success && result.outcome == ExecutionOutcome::{error}'
        if error=='Success':check+=f' && result.value.initialized && result.value.number == {expected}';old=f'result.value.number == {expected}';new=f'result.value.number == {expected+1}'
        else:old=f'result.outcome == ExecutionOutcome::{error}';new='result.outcome == ExecutionOutcome::Success'
        add(label,body,check,old,new,description)
    minimal=method('MAIN',returned(b'\x72'+integer(40)+integer(2)+b'\0'))
    program_case('load_run_add',minimal,description='Real Method declaration captures scope/body and executes Add/Return')
    program_case('owned_source_copy',minimal,before='input[10] = 99;\n',description='Mutating caller input after prepare does not replace owned method bytes')
    program_case('nested_method',method('FOO',returned(b'\x72\x68\x01\x00'),1)+method('MAIN',returned(name('FOO')+integer(41))),description='Method arity and both captured source spans feed nested call')
    common=scope('D0',named('X',11)+method('FOO',returned(name('X'))))+scope('D1',named('X',22)+b'\x06'+absolute('D0','FOO')+name('ALIS'))
    tail=method('MAIN',returned(absolute('D1','ALIS')))
    program_case('cross_scope_alias',common+tail,11,description='Alias invocation resolves X from declaration D0, not invocation alias D1')
    program_case('alias_original_rebound',common+b'\x08'+absolute('D0','FOO')+integer(99)+tail,11,description='Original declaration scope survives source Name rebinding after Alias')
    twice=method('FOO',returned(integer(11)))+b'\x06'+name('FOO')+name('OLDN')+method('FOO',returned(integer(22)))+method('MAIN',returned(b'\x72'+name('OLDN')+name('FOO')+b'\0'))
    program_case('method_redeclaration',twice,33,description='Fresh method ID gets new observation; old alias retains old body and observation')
    program_case('serialized_boundary',method('MAIN',returned(integer(1)),8),error='UnresolvedSynchronization',description='Loader captures flags; executor explicitly rejects serialized method')
    program_case('source_unit_mismatch',minimal,error='SourceUnit',before='program.unit = 8;\n',description='Mismatched retained source unit is rejected before byte execution')
    program_case('method_payload_mismatch',minimal,error='MethodDefinition',before='program.store.space.objects[0].value = Value::Method { body: Span { unit:7,start:0,end:0 } };\n',description='Changed live Method payload does not silently rewrite captured observation')
    program_case('maximum_source_unit',minimal,unit=(1<<64)-1,description='Opaque u64 maximum source identity remains exact data')
    for label,data,budget,error in [('rollback_unsupported',minimal+b'\x72',64,'UnsupportedSyntax'),('rollback_after_capture',minimal,1,'WorkLimit'),('zero_load_budget',minimal,0,'WorkLimit')]:
        body=array(data)+f'let prepared: Prepared = prepare_program(input,{len(data)},7,{budget},128);\nlet mut program: Program = prepared.program;\n'+path('target_path','MAIN')+'let arguments: [Value;7];\nlet result: ExecutionResult = run_program(&mut program,target_path,&arguments,0,IntegerSize::FourBytes,64);\n'
        check=f'prepared.outcome == Outcome::{error} && !program.loaded && program.store.space.object_count == 0 && !program.definitions.entries[0].present && result.outcome == ExecutionOutcome::InvalidState'
        add(label,body,check,'program.store.space.object_count == 0','program.store.space.object_count == 1','Failed static load rolls namespace and captured definitions back; failed program cannot execute')
    for label,length,budget in [('input_capacity',1025,64),('input_maximum',(1<<64)-1,64),('budget_maximum',len(minimal),(1<<64)-1)]:
        body=array(minimal)+f'let prepared: Prepared = prepare_program(input,{length},7,{budget},128);\n'
        add(label,body,'prepared.outcome == Outcome::Capacity && !prepared.program.loaded && !prepared.program.definitions.entries[0].present','prepared.outcome == Outcome::Capacity','prepared.outcome == Outcome::Success','Capacity rejection does not install namespace or method observations')
    # Populate the object arena through an actual parsed Package, while leaving
    # room in the separate namespace entry array for a Method declaration.
    arena=b'\x08'+name('PKG0')+package(0x12,bytes([62])+b'\x00'*62)
    one=method('MAIN',returned(integer(42)))
    for label,following,error in [('last_object_slot',one,'Success'),('object_capacity_rollback',one+method('NEXT',returned(integer(1))),'Capacity')]:
        body=array(arena,'initial_bytes')+array(following)+f'let initial: ObservedLoad = load_with_definitions(&initial_bytes,{len(arena)},6,empty(),MethodDefinitions {{}},4,128);\nlet loaded: ObservedLoad = load_with_definitions(&input,{len(following)},7,initial.load.space,initial.definitions,8,128);\n'
        if error=='Success':
            check='initial.load.outcome == Outcome::Success && loaded.load.outcome == Outcome::Success && loaded.load.space.object_count == 64 && loaded.definitions.entries[63].present && loaded.definitions.entries[63].object_id == 63'
            old='loaded.load.space.object_count == 64';new='loaded.load.space.object_count == 63'
        else:
            check='initial.load.outcome == Outcome::Success && loaded.load.outcome == Outcome::Capacity && loaded.load.space.object_count == 63 && !loaded.definitions.entries[63].present'
            old='loaded.load.space.object_count == 63';new='loaded.load.space.object_count == 64'
        add(label,body,check,old,new,'Fresh Method slot63 is usable; later allocation failure rolls it and its observation back')
    body=array(one)+path('old_path','OLDN')+'let mut occupied: MethodDefinitions;\noccupied.entries[0] = MethodDefinition { present:true,object_id:0,scope:old_path,body:Span {unit:99,start:3,end:4} };\n'+f'let loaded: ObservedLoad = load_with_definitions(&input,{len(one)},7,empty(),occupied,4,128);\n'
    check='loaded.load.outcome == Outcome::InvalidState && loaded.load.space.object_count == 0 && loaded.definitions.entries[0].present && loaded.definitions.entries[0].body.unit == 99'
    add('occupied_observation_slot',body,check,'loaded.definitions.entries[0].body.unit == 99','loaded.definitions.entries[0].body.unit == 7','An occupied observation slot is rejected before committing the tentative namespace insertion; original sidecar is preserved')
    first=method('OLDN',returned(integer(11)))
    for label,second,expected_load,entry,expected_run in [
        ('incremental_rollback',method('NEWN',returned(integer(22)))+b'\x72','UnsupportedSyntax','OLDN','Success'),
        ('incremental_source_unit',method('MAIN',returned(name('OLDN'))),'Success','MAIN','SourceUnit')]:
        body=array(first,'first_bytes')+array(second,'second_bytes')+f'let first: ObservedLoad = load_with_definitions(&first_bytes,{len(first)},6,empty(),MethodDefinitions {{}},4,128);\nlet second: ObservedLoad = load_with_definitions(&second_bytes,{len(second)},7,first.load.space,first.definitions,8,128);\n'
        source='first_bytes'if expected_load!='Success'else'second_bytes';length=len(first)if source=='first_bytes'else len(second);unit=6 if source=='first_bytes'else 7
        body+=f'let mut program: Program = Program {{ loaded:true,source:{source},length:{length},unit:{unit},store:ObjectStore {{space:second.load.space}},definitions:second.definitions }};\n'+path('target_path',entry)+'let arguments: [Value;7];\nlet result: ExecutionResult = run_program(&mut program,target_path,&arguments,0,IntegerSize::FourBytes,64);\n'
        check=f'first.load.outcome == Outcome::Success && second.load.outcome == Outcome::{expected_load} && second.definitions.entries[0].present && second.definitions.entries[0].body.unit == 6 && result.outcome == ExecutionOutcome::{expected_run}'
        if expected_load!='Success':check+=' && !second.definitions.entries[1].present && result.value.number == 11';old='result.value.number == 11';new='result.value.number == 12'
        else:check+=' && second.definitions.entries[1].present && second.definitions.entries[1].body.unit == 7';old='result.outcome == ExecutionOutcome::SourceUnit';new='result.outcome == ExecutionOutcome::Success'
        add(label,body,check,old,new,'Lowlevel incremental loader preserves declaration observations; rollback retains callable prior source, while absent multiunit dispatch rejects calls into another source unit')
    repeated=named('X',0)+method('MAIN',b'\x75'+name('X')+returned(name('X')))
    body=array(repeated)+f'let prepared: Prepared = prepare_program(input,{len(repeated)},7,8,128);\nlet mut program: Program = prepared.program;\n'+path('target_path','MAIN')+'let arguments:[Value;7];\nlet first:ExecutionResult=run_program(&mut program,target_path,&arguments,0,IntegerSize::FourBytes,64);\nlet second:ExecutionResult=run_program(&mut program,target_path,&arguments,0,IntegerSize::FourBytes,64);\n'
    check='prepared.outcome == Outcome::Success && first.outcome == ExecutionOutcome::Success && first.value.number == 1 && second.outcome == ExecutionOutcome::Success && second.value.number == 2 && program.definitions.entries[1].present'
    add('persistent_namespace',body,check,'second.value.number == 2','second.value.number == 3','Repeated calls mutate the retained namespace without reloading source or replacing observations')
    return rows

def render(row,control=False,entry=None):
    check=row['check']
    if control:check=check.replace(*row['mutation'])
    head='machine test_result()'if entry is None else f'machine {entry}(&mut self)'
    body=head+' -> i32 {\n'+row['body']+'transition '+check+' { true -> (0) _ -> (1) }\n}\n'
    return body

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args();rows=cases()
    main_source=IMPORTS+render(rows[0])+'const TEST_RESULT:i32=test_result();\nmachine require_ok(value:i32) requires value==0;{}\ndata Main{}\nmachine Main::main(&mut self){require_ok(TEST_RESULT);}\n'
    outputs={HERE/'cases.json':json.dumps(rows,indent=2)+'\n',HERE/'main.omg':main_source}
    for target,text in outputs.items():
        if args.check:
            if not target.exists()or target.read_text()!=text:raise SystemExit('Fixture differs: '+str(target))
        else:target.write_text(text)
    print(f'{len(rows)} pipeline scenarios '+('verified'if args.check else'generated'))
if __name__=='__main__':main()
