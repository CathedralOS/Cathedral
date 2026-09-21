"""Original public Program boundary witnesses for shared returned-data quotas."""
import importlib.util,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
LEGACY=ROOT/'tools/ports/acpi/interpreter/execution/fixtures.py'
spec=importlib.util.spec_from_file_location('integer_fixture',LEGACY);old=importlib.util.module_from_spec(spec);spec.loader.exec_module(old)
integer,pkg,ret,op,name=old.integer,old.pkg,old.ret,old.op,old.name
MAX=(1<<64)-1

def cases():
    rows=[];buffer=pkg(0x11,integer(3)+b'ABC');package=pkg(0x12,b'\x02'+buffer+integer(5))
    prefix=b'\x08'+name('X')+package+b'\x08'+name('TICK')+integer(0)
    written=op(0x70,integer(1),name('TICK'))
    def add(label,setup='',error='Success',objects=3,bytes=3,default=False,body=None,tick=1,steps=True,root=0,scalar=None,present=True,count=5):
        code=written+(ret(name('X')) if body is None else body)
        table=prefix+pkg(0x14,name('MAIN')+b'\0'+code)
        rows.append(dict(name=label,setup=setup,error=error,objects=objects,bytes=bytes,default=default,table=table.hex(),tick=tick,steps=steps,root=root,scalar=scalar,present=present if error=='Success' else False,count=count))
    add('default_valid',default=True)
    add('exact_quotas')
    bad_owner='program.store.space.objects[1].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:2}};'
    add('default_nested_bad_owner',bad_owner,error='InvalidState',default=True)
    add('nested_invalid_string','program.store.space.objects[1].value=Value::String {string_storage:StringStorage::Owned {string_owner:1}};program.store.bytes.blocks[1]=ByteBlock {initialized:true,length:1};program.store.bytes.blocks[1].bytes[0]=255;',error='InvalidState')
    add('nested_bad_package','program.store.space.objects[1].value=Value::Package {first:2,count:2};',error='InvalidState')
    add('empty_nested_package',f'program.store.space.objects[1].value=Value::Package {{first:{MAX},count:0}};',bytes=0)
    add('shared_reference','program.store.space.objects[2].value=Value::Reference {kind:ReferenceKind::RefOf,object_id:1};')
    add('opaque_reference_cycle','program.store.space.objects[2].value=Value::Reference {kind:ReferenceKind::RefOf,object_id:2};')
    add('dangling_reference',f'program.store.space.objects[2].value=Value::Reference {{kind:ReferenceKind::RefOf,object_id:{MAX}}};',error='InvalidState')
    add('lexical_forward','let mut forward:Path=Path {count:1};forward.segments[0]=1598314310;program.store.space.objects[2].value=Value::NameReference {name:forward,scope:Path {absolute:true}};')
    add('lexical_invalid','program.store.space.objects[2].value=Value::NameReference {name:Path {},scope:Path {}};',error='InvalidState')
    add('padding_uninitialized','program.store.space.objects[2].value=Value::Uninitialized;')
    add('unsupported_nested_service','program.store.space.objects[2].value=Value::OperationRegion {};',error='UnsupportedValue')
    add('object_quota_short',objects=2,error='WorkLimit')
    add('byte_quota_short',bytes=2,error='Capacity')
    add('zero_object_quota',objects=0,error='WorkLimit')
    add('zero_byte_quota',bytes=0,error='Capacity')
    add('oversized_object_request',objects=65,error='Capacity',tick=0,steps=False)
    add('oversized_byte_request',bytes=16385,error='Capacity',tick=0,steps=False)
    add('maximum_object_request',objects=MAX,error='Capacity',tick=0,steps=False)
    add('inline_integer_zero_quotas',objects=0,bytes=0,body=ret(integer(17)),scalar=17)
    add('void_zero_quotas',objects=0,bytes=0,body=b'\xa3',present=False)
    add('existing_execution_error',body=ret(name('MISS')),error='MissingObject')
    add('empty_buffer_zero_bytes','program.store.space.objects[1].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:1}};program.store.bytes.blocks[1]=ByteBlock {initialized:true};',bytes=0)
    add('private_result_graph',body=op(0x9d,name('X'),b'\x60')+ret(b'\x60'),root=5,count=6)
    add('failed_result_keeps_allocation',bad_owner,body=op(0x9d,name('X'),b'\x60')+ret(b'\x60'),error='InvalidState',count=6)
    add('opaque_root_identity','program.store.space.objects[0].value=Value::Reference {kind:ReferenceKind::RefOf,object_id:1};',objects=2)
    return rows

def render(rows):
    s='''module main;
use aml::model::Value;
use aml::model::Path;
use aml::model::Outcome;
use aml::model::BufferStorage;
use aml::model::StringStorage;
use aml::model::ByteBlock;
use aml::model::ReferenceKind;
use pipeline::program::Program;
use pipeline::program::Prepared;
use pipeline::program::prepare_program;
use pipeline::program::run_program;
use pipeline::program::run_program_limited;
use execution::engine::ExecutionResult;
use execution::execution_model::ExecutionOutcome;
use execution::execution_model::Operand;
use integer_helpers::integers::IntegerSize;
data Suite {}
machine result_value(result:ExecutionResult,present:bool,scalar:bool,expected:u64,object:u64)->bool {
 transition result.operand {
  Operand::Integer {number} -> (present && scalar && number==expected && result.has_value && result.value.initialized && result.value.number==expected)
  Operand::Object {object_id} -> (present && !scalar && object_id==object && result.has_value)
  Operand::Uninitialized -> (!present && !result.has_value && !result.value.initialized)
 }
}
machine tick(value:Value,wanted:u64)->bool {transition value {Value::Integer {number} -> (number==wanted) _ -> (false)}}
'''
    selections=[]
    for row in rows:
        data=bytes.fromhex(row['table'])
        for control in (False,True):
            machine='Suite::'+row['name']+('_control' if control else '_positive');selections.append(machine+'='+str(int(control)))
            s+=f'machine {machine}(&mut self)->i32 {{let mut input:[u8;1024];'+''.join(f'input[{i}]={v};'for i,v in enumerate(data))
            s+=f'let prepared:Prepared=prepare_program(input,{len(data)},7,128,128);let loaded:Outcome=prepared.outcome;let mut program:Program=prepared.program;'+row['setup']
            s+='let mut path:Path=Path {absolute:true,count:1};path.segments[0]=1313423693;let arguments:[Value;7];'
            call='run_program(&mut program,path,&arguments,0,IntegerSize::EightBytes,256)' if row['default'] else f'run_program_limited(&mut program,path,&arguments,0,IntegerSize::EightBytes,256,{row["objects"]},{row["bytes"]})'
            s+='let result:ExecutionResult='+call+';'
            s+=f'let value:bool=result_value(result,{str(row["present"]).lower()},{str(row["scalar"]is not None).lower()},{row["scalar"] or 0},{row["root"]});'
            wanted=row['tick']+1 if control else row['tick']
            s+=f'let effect:bool=tick(program.store.space.objects[3].value,{wanted});'
            steps='result.steps>0' if row['steps'] else 'result.steps==0'
            s+=f'transition loaded==Outcome::Success && result.outcome==ExecutionOutcome::{row["error"]} && value && effect && {steps} && program.store.space.object_count=={row["count"]} {{true -> (0) _ -> (1)}}}}\n'
    return s,selections
if __name__=='__main__':
    (HERE/'cases.json').write_text(json.dumps(cases(),indent=2)+'\n');print(len(cases()),'Program boundary cases')
