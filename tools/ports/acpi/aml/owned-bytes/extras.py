#!/usr/bin/env python3
"""Composition and lifetime witnesses supplementary to frozen whole-store corpus."""
import fixtures as f
IMPORTS=f.IMPORTS+'''use pipeline::program::Program;
use pipeline::program::Prepared;
use pipeline::program::prepare_program;
'''
HELPERS='''machine fx_return_index(program:&mut Program)->ByteResult {
 let result:ByteResult=make_byte_index(&program.source,program.length,program.unit,&mut program.store,0,1);result
}
'''
def cases():
 rows=[]
 def add(name,body,check,old,new):
  assert check.count(old)==1;rows.append(dict(name=name,body=body,check=check,mutation=[old,new]))
 for kind,payload,length in [('buffer',f.source(),4),('string',f.source('String',end=2),2)]:
  add('length_'+kind,f.seed(payload,data=[65,66])+'let result:ByteResult=byte_length(&input,2,7,&store,0);',f'result.outcome==ByteOutcome::Success && result.length=={length}',f'result.length=={length}',f'result.length=={length+1}')
 setup=f.seed()+'store.space.objects[1].value=Value::Reference {kind:ReferenceKind::Local,object_id:0};let result:ByteResult=materialize(&input,2,7,&mut store,1);let observed:ByteRead=read_bytes(&input,2,7,&store,1);'
 add('materialize_through_identity',setup,'result.outcome==ByteOutcome::Success && result.object==0 && observed.bytes[0]==42 && store.space.object_count==4','observed.bytes[0]==42','observed.bytes[0]==43')
 setup=f.seed()+'store.space.objects[1].value=Value::Reference {kind:ReferenceKind::Arg,object_id:0};let result:ByteResult=create_buffer_field(&input,2,7,&mut store,1,8,8);let read:ByteResult=read_field_integer(&input,2,7,&store,result.object,IntegerSize::EightBytes);'
 add('create_through_identity',setup,'result.outcome==ByteOutcome::Success && read.outcome==ByteOutcome::Success && read.value==19','read.value==19','read.value==20')
 setup=f.seed(f.source('String',end=2),data=[65,66])+'let index:ByteResult=make_byte_index(&input,2,7,&mut store,0,1);let written:ByteResult=write_field_integer(&input,2,7,&mut store,index.object,IntegerSize::EightBytes,67);let observed:ByteRead=read_bytes(&input,2,7,&store,0);'
 add('string_index_write',setup,'index.outcome==ByteOutcome::Success && written.outcome==ByteOutcome::Success && observed.kind==ByteKind::String && observed.bytes[1]==67 && input[1]==66','observed.bytes[1]==67','observed.bytes[1]==68')
 for source_kind,target_kind in [('String','Buffer'),('Buffer','String')]:
  setup=f.seed(f.source(source_kind,declared=2,end=2),data=[65,66])+f'store.space.objects[1].value={f.source(target_kind,declared=2,end=2)};let copied:ByteResult=clone_bytes_into(&input,2,7,&mut store,1,0);let observed:ByteRead=read_bytes(&input,2,7,&store,1);'
  add('clone_changes_type_'+source_kind.lower(),setup,f'copied.outcome==ByteOutcome::Success && observed.kind==ByteKind::{source_kind} && observed.bytes[1]==66','observed.bytes[1]==66','observed.bytes[1]==67')
 setup=f.seed()+'let index:ByteResult=make_byte_index(&input,2,7,&mut store,0,3);'+f.block('store',0,[1,2])+'let result:ByteResult=write_field_integer(&input,2,7,&mut store,index.object,IntegerSize::EightBytes,99);'
 add('field_revalidates_shrunken_backing',setup,'result.outcome==ByteOutcome::Bounds && store.bytes.blocks[0].bytes[1]==2','store.bytes.blocks[0].bytes[1]==2','store.bytes.blocks[0].bytes[1]==3')
 setup=f.seed()+'store.space.objects[1].value=Value::Reference {kind:ReferenceKind::Named,object_id:0};store.space.objects[2].value=Value::BufferField {backing_object:1,bit_length:8};let result:ByteResult=write_field_integer(&input,2,7,&mut store,2,IntegerSize::EightBytes,99);'
 add('field_resolves_reference_backing',setup,'result.outcome==ByteOutcome::Success && result.object==0 && store.bytes.blocks[0].bytes[0]==99','store.bytes.blocks[0].bytes[0]==99','store.bytes.blocks[0].bytes[0]==100')
 # Actual parser produces buffer source metadata; affine Program owns all bytes.
 # Name(BUF0, Buffer(4){0x41,0x42}) and a scope-local host call returning Index.
 source=[8,66,85,70,48,17,5,10,4,65,66]
 body='let mut input:[u8;1024];'+f.assignments('input',source)+f'let prepared:Prepared=prepare_program(input,{len(source)},7,8,32);let mut program:Program=prepared.program;input[10]=99;let index:ByteResult=fx_return_index(&mut program);let result:ByteResult=write_field_integer(&program.source,program.length,program.unit,&mut program.store,index.object,IntegerSize::EightBytes,67);let observed:ByteRead=read_bytes(&program.source,program.length,program.unit,&program.store,0);'
 add('program_retains_index_after_host_call',body,'prepared.outcome==Outcome::Success && index.outcome==ByteOutcome::Success && result.outcome==ByteOutcome::Success && observed.bytes[0]==65 && observed.bytes[1]==67 && observed.bytes[2]==0 && program.source[10]==66 && program.store.space.object_count==3','observed.bytes[1]==67','observed.bytes[1]==68')
 return rows

def render(row,control=False,machine='test_result'):
 check=row['check'].replace(*row['mutation'])if control else row['check'];return 'machine '+machine+('(&mut self)'if'::'in machine else'()')+'->i32 {\n'+row['body']+'\ntransition '+check+' {true -> (0) _ -> (1)}}\n'
