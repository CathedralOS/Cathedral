"""Original fail-closed boundary pairs for the three canonical Field bindings."""
from pathlib import Path
import importlib.util
import re
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[4]
spec=importlib.util.spec_from_file_location('loader_fixtures',HERE.parent/'field-namespace/fixtures.py')
loader=importlib.util.module_from_spec(spec);spec.loader.exec_module(loader)
EXTRA='''use aml::model::ObjectStore;
use aml::model::ByteArena;
use aml::model::ByteBlock;
use aml::object_queries::object_type;
use aml::object_queries::QueryResult;
use execution::execution_model::ExecutionOutcome;
use execution::execution_model::Operand;
use execution::execution_model::BindingResult;
use execution::execution_model::SlotResult;
use execution::frames::scalar_slot;
use execution::generic_values::validate_operand;
use execution::generic_values::destination_allowed;
use execution::generic_target_model::TargetSelection;
use execution::generic_target_values::apply_selected;
use integers::integers::IntegerSize;
// Build one sentinel byte pattern and publish complete blocks under local bounds.
machine seed_bytes(arena:&mut ByteArena,index:u64,count:u64) {
 let mut bytes:[u8;256];seed_octets(&mut bytes,0,256);seed_blocks(arena,bytes,index,count);
}
machine seed_octets(bytes:&mut [u8;256],index:u64,count:u64)
terminates by(index,count)->Nat::BoundedDistance;
{seed_octet(bytes,index);transition index<count {true -> seed_octets(bytes,index+1,count) _ -> {}}}
machine seed_octet(bytes:&mut [u8;256],index:u64) {transition index<256 {true -> put(bytes,index) _ -> {}}
 state put(bytes:&mut [u8;256],index:u64) {bytes[index]=(index^165)as u8;}}
machine seed_blocks(arena:&mut ByteArena,bytes:[u8;256],index:u64,count:u64)
terminates by(index,count)->Nat::BoundedDistance;
{seed_block(arena,bytes,index);transition index<count {true -> seed_blocks(arena,bytes,index+1,count) _ -> {}}}
machine seed_block(arena:&mut ByteArena,bytes:[u8;256],index:u64) {transition index<64 {true -> put(arena,bytes,index) _ -> {}}
 state put(arena:&mut ByteArena,bytes:[u8;256],index:u64) {arena.blocks[index]=ByteBlock {initialized:true,length:index+1,bytes:bytes};}}
machine equal_bytes(a:&ByteArena,b:&ByteArena,index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let same:bool=equal_block(a,b,index);transition index<count {true -> equal_bytes(a,b,index+1,count,prior && same) _ -> (prior)}}
machine equal_block(a:&ByteArena,b:&ByteArena,index:u64)->bool {transition index<64 {true -> same(a.blocks[index],b.blocks[index]) _ -> (true)}
 state same(a:ByteBlock,b:ByteBlock)->bool {let bytes:bool=equal_octets(&a.bytes,&b.bytes,0,256,true);bytes && a.initialized==b.initialized && a.length==b.length}}
machine equal_octets(a:&[u8;256],b:&[u8;256],index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let same:bool=equal_octet(a,b,index);transition index<count {true -> equal_octets(a,b,index+1,count,prior && same) _ -> (prior)}}
machine equal_octet(a:&[u8;256],b:&[u8;256],index:u64)->bool {transition index<256 {true -> (a[index]==b[index]) _ -> (true)}}
machine type_five(result:QueryResult)->bool {transition result {QueryResult::Value {object,number} -> (object==0 && number==5) _ -> (false)}}
machine binding_kind(binding:FieldBinding)->DeclarationKind {transition binding {FieldBinding::Region {region_object} -> (DeclarationKind::Field) FieldBinding::Bank {region_object,selector_object} -> (DeclarationKind::Bank) FieldBinding::Index {index_object,data_object} -> (DeclarationKind::Index)}}
machine boundary(binding:FieldBinding,size:IntegerSize,control:bool)->i32 {
 let mut input:[u8;1024];let mut store:ObjectStore;store.space=initial_space();store.space.object_count=3;
 let kind:DeclarationKind=binding_kind(binding);
 let value:Value=Value::FieldUnit {binding:binding,declaration:Declaration {kind:kind,bank_value:18446744073709551615},field:Field {bit_offset:5,bit_length:8}};
 store.space.objects[0].value=value;store.space.objects[1].value=Value::Integer {number:9};store.space.objects[2].value=Value::Reference {kind:ReferenceKind::Local,object_id:0};
 seed_bytes(&mut store.bytes,0,64);let mut expected:ObjectStore=store;
 let query:QueryResult=object_type(&store.space,0,1);let direct:bool=type_five(query);
 let transparent:QueryResult=object_type(&store.space,2,2);let indirect:bool=type_five(transparent);
 let scalar:SlotResult=scalar_slot(size,value);
 let admitted:ExecutionOutcome=validate_operand(&input,0,0,&store,Operand::Object {object_id:0});
 let allowed:ExecutionOutcome=destination_allowed(&store,0);
 let stored:BindingResult=apply_selected(&input,0,0,size,&mut store,TargetSelection::Named {object_id:0},Operand::Integer {number:9},false);
 let copied:BindingResult=apply_selected(&input,0,0,size,&mut store,TargetSelection::Named {object_id:0},Operand::Object {object_id:1},true);
 let source:BindingResult=apply_selected(&input,0,0,size,&mut store,TargetSelection::Named {object_id:1},Operand::Object {object_id:0},true);
 let precedence:BindingResult=apply_selected(&input,0,0,size,&mut store,TargetSelection::Named {object_id:0},Operand::Object {object_id:64},false);
 corrupt(&mut expected,control);
 let space_entries:bool=entries(&store.space,&expected.space,0,32,true);let space_objects:bool=objects(&store.space,&expected.space,0,64,true);let arena:bool=equal_bytes(&store.bytes,&expected.bytes,0,64,true);
 transition direct && indirect && scalar.outcome==ExecutionOutcome::UnresolvedRegion && admitted==ExecutionOutcome::UnresolvedRegion && allowed==ExecutionOutcome::UnresolvedRegion && stored.outcome==ExecutionOutcome::UnresolvedRegion && copied.outcome==ExecutionOutcome::UnresolvedRegion && source.outcome==ExecutionOutcome::UnresolvedRegion && precedence.outcome==ExecutionOutcome::UnresolvedRegion && space_entries && space_objects && arena && store.space.count==expected.space.count && store.space.object_count==expected.space.object_count {true -> (0) _ -> (1)}
}
machine corrupt(store:&mut ObjectStore,control:bool) {transition control {true -> change(store) _ -> {}}state change(store:&mut ObjectStore) {store.bytes.blocks[63].bytes[255]=store.bytes.blocks[63].bytes[255]^1;}}
'''
def fixture():
 source=loader.IMPORTS.replace('use aml::loader::load_with_definitions;\n','')+loader.HELPERS+EXTRA+'data Suite {}\n';names=[]
 for kind,binding in [('region','FieldBinding::Region {region_object:17}'),('bank','FieldBinding::Bank {region_object:17,selector_object:31}'),('index','FieldBinding::Index {index_object:17,data_object:31}')]:
  for width in ['FourBytes','EightBytes']:
   for control in [False,True]:
    name=f'Suite::{kind}_{width}_'+('control'if control else'positive');names.append(name+'='+str(int(control)))
    source+=f'machine {name}(&mut self)->i32 {{let result:i32=boundary({binding},IntegerSize::{width},{str(control).lower()});result}}\n'
 # Unique free-helper names avoid pinned interpreter leaf-name collisions with
 # states in the imported generic execution packages.
 for helper in re.findall(r'^machine (\w+)\(',source,re.M):
  source=re.sub(r'\b'+helper+r'\(', 'fb_'+helper+'(',source)
 return source,names
if __name__=='__main__':
 source,names=fixture();(HERE/'main.omg').write_text(source);(HERE/'selections.txt').write_text('\n'.join(names)+'\n');print('Generated six Field binding behavior/control pairs')
