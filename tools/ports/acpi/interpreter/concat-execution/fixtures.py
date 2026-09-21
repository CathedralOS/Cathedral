"""Concatenate results from independent byte operations and whole-state witnesses."""
from pathlib import Path
import runpy
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[4]
COMPARATOR=HERE.parent/'generic-execution/focused/bridge-atomicity/main.omg'
RETIRE_FIXTURES=HERE.parent/'field-write-execution/fixtures.py'

def result(a,b,size):
    ak,av=a;bk,bv=b;mask=(1<<size)-1
    if ak in ('integer','inline'):
        left=(av&mask).to_bytes(size//8,'little')
        if bk in ('integer','inline'):right=bv&mask
        elif bk=='string':right=int(bv.decode(),16)&mask
        elif bk=='buffer':right=int.from_bytes(bv[:size//8],'little')
        else:raise ValueError('unsupported')
        return 'buffer',left+right.to_bytes(size//8,'little')
    if ak=='buffer':
        if bk in ('integer','inline'):right=(bv&mask).to_bytes(size//8,'little')
        elif bk=='string':right=bv+b'\0' if bv else b''
        elif bk=='buffer':right=bv
        else:right=b'[Package]\0'
        return 'buffer',av+right
    left=av if ak=='string' else b'[Package]'
    if bk in ('integer','inline'):right=f'{bv&mask:0{size//4}X}'.encode()
    elif bk=='buffer':right=b' '.join(f'{x:02X}'.encode() for x in bv)
    elif bk=='string':right=bv
    else:right=b'[Package]'
    return 'string',left+right

def install(store,id,kind,value):
    if kind in ('integer','inline'):return f'{store}.space.objects[{id}]=Object {{value:Value::Integer {{number:{value}}}}};'
    if kind=='package':return f'{store}.space.objects[{id}]=Object {{value:Value::Package {{}}}};'
    cap=kind.title();owner='string_owner' if kind=='string' else 'buffer_owner';storage='string_storage' if kind=='string' else 'buffer_storage'
    s=f'{store}.space.objects[{id}]=Object '+'{value:Value::'+cap+' {'+storage+':'+cap+'Storage::Owned {'+owner+':'+str(id)+'}}};'
    s+=f'{store}.bytes.blocks[{id}]=ByteBlock '+'{initialized:true,length:'+str(len(value))+'};'
    s+=''.join(f'{store}.bytes.blocks[{id}].bytes[{i}]={v};' for i,v in enumerate(value) if v)
    return s

def operand(value,id):return f'Operand::Integer {{number:{value[1]}}}' if value[0]=='inline' else f'Operand::Object {{object_id:{id}}}'

def cases():
    rows=[]
    for size in (32,64):
        def add(name,a=('buffer',b'A'),b=('buffer',b'B'),target='null',count=4,setup='',left=None,right=None,error=None):
            kind,data=result(a,b,size)
            rows.append(dict(name=f'{name}_{size}',size=size,a=a,b=b,target=target,count=count,setup=setup,left=left,right=right,error=error,kind=kind,data=data))
        basic=[('inline',0x123456789abcdef0),('string',b'F0'),('buffer',b'\x12\x34')]
        for a in basic:
            for b in basic:add(a[0]+'_'+b[0],a,b)
        add('stored_integers',('integer',0x123456789abcdef0),('integer',9))
        add('description_left',('package',0),('inline',9))
        add('description_right',('string',b'x'),('package',0))
        add('empty_string',('string',b''),('string',b''))
        add('empty_buffer',('buffer',b''),('inline',0))
        add('full_string',('string',b'a'*128),('string',b'b'*128))
        add('overflow',('string',b'a'*256),('string',b'b'),error='Capacity')
        add('malformed_right_before_fit',('string',b'a'*256),('string',b'b'),setup='store.bytes.blocks[1].bytes[0]=0;',error='Encoding')
        for target in ['local','local_cell','argument','arg_ref','named_integer','named_string','named_buffer','field','arg_field','field_disabled','debug','stale_named','stale_cell','stale_ref']:
            add('target_'+target,target=target,error={'field_disabled':'UnresolvedRegion','debug':'UnresolvedService','stale_named':'MissingObject','stale_cell':'MissingObject','stale_ref':'MissingObject'}.get(target))
        add('self_string',('string',b'A'),('string',b'B'),target='self')
        add('late_target_storage',('string',b'A'),('string',b'B'),target='named_string',setup='store.bytes.blocks[2].initialized=false;',error='InvalidState')
        add('last_slot',('inline',1),('inline',2),count=63)
        add('last_slot_local_rollback',('inline',1),('inline',2),target='local',count=63,error='Capacity')
        add('full_arena',('inline',1),('inline',2),count=64,error='Capacity')
        add('parent_full',setup='frame.operations[0].count=1;',error='InvalidState')
        add('parent_full_field',target='field',setup='frame.operations[0].count=1;',error='InvalidState')
        add('uninitialized',left='Operand::Uninitialized',error='Uninitialized')
        add('missing_source',left='Operand::Object {object_id:64}',error='InvalidState')
        add('transparent',setup='store.space.objects[3].value=Value::Reference {kind:ReferenceKind::Named,object_id:0};',left='Operand::Object {object_id:3}')
        add('explicit_reference',setup='store.space.objects[3].value=Value::Reference {kind:ReferenceKind::RefOf,object_id:0};',left='Operand::Object {object_id:3}',error='UnsupportedValue')
        add('cycle',setup='store.space.objects[3].value=Value::Reference {kind:ReferenceKind::Named,object_id:3};',left='Operand::Object {object_id:3}',error='InvalidState')
    for size in (32,64):
        basic=[('inline',0x123456789abcdef0),('string',b'F0'),('buffer',b'\x12\x34')]
        for a in basic:
            for b in basic:rows.append(dict(name=f'aml_{a[0]}_{b[0]}_{size}',size=size,a=a,b=b,bytecode='null'))
        for target in ['local','self','nested']:
            rows.append(dict(name=f'aml_{target}_{size}',size=size,a=('string',b'A') if target=='self' else ('buffer',b'A'),b=('string',b'B') if target=='self' else ('buffer',b'B'),bytecode=target))
    return rows

def body(row,control):
    if 'bytecode' in row:return bytecode_body(row,control)
    size=row['size'];a,b=row['a'],row['b'];target=row['target'];count=row['count'];kind,data=row['kind'],row['data']
    s=f'let input:[u8;1024];let mut store:ObjectStore=ObjectStore {{}};store.space.object_count={count};'
    s+=install('store',0,*a)+install('store',1,*b)+install('store',2,'integer',77)
    s+='store.bytes.blocks[63].bytes[255]=91;'
    s+=f'let mut frame:Frame=Frame {{field_writes:true,size:IntegerSize::{"FourBytes" if size==32 else "EightBytes"},operation_count:1}};'
    s+='frame.operations[0]=Operation {opcode:0xa4,value_arity:1};frame.lookup_cache[15].object_id=71;'
    dest='Target::Null'
    if target in ('local','local_cell','stale_cell'):
        dest='Target::Local {local_index:0}'
        if target!='local':s+='frame.locals[0]=Binding::Cell {cell_id:'+str(4 if target=='stale_cell' else 2)+'};'
    elif target in ('argument','arg_ref','arg_field','stale_ref'):
        dest='Target::Argument {argument_index:0}'
        if target=='argument':s+='frame.arguments[0]=Binding::Shared {shared_id:2};'
        else:
            s+='store.space.objects[3].value=Value::Reference {kind:ReferenceKind::RefOf,object_id:'+str(4 if target=='stale_ref' else 2)+'};frame.arguments[0]=Binding::Shared {shared_id:3};'
    elif target=='debug':dest='Target::Debug'
    elif target!='null':dest='Target::Named {object_id:'+str(0 if target=='self' else 4 if target=='stale_named' else 2)+'}'
    if target=='named_string':s+=install('store',2,'string',b'z')
    if target=='named_buffer':s+=install('store',2,'buffer',b'xx')
    if target in ('field','arg_field','field_disabled'):s+='store.space.objects[2].value=Value::FieldUnit {};'
    if target=='field_disabled':s+='frame.field_writes=false;'
    s+=row['setup']+'let mut expected_store:ObjectStore=store;let mut expected_frame:Frame=frame;'
    if not row['error']:
        s+=install('expected_store',count,kind,data)+f'expected_store.space.object_count={count+1};'
        if target in ('local','argument'):
            s+=install('expected_store',count+1,kind,data)+f'expected_store.space.object_count={count+2};'
            s+=f'expected_frame.{"locals" if target=="local" else "arguments"}[0]=Binding::Cell '+'{cell_id:'+str(count+1)+'};'
        elif target in ('local_cell','arg_ref','named_buffer','self'):
            s+=install('expected_store',0 if target=='self' else 2,kind,data)
        elif target=='named_integer':s+=install('expected_store',2,'integer',int.from_bytes(data[:size//8],'little'))
        elif target=='named_string':s+=install('expected_store',2,'string',b' '.join(f'{v:02X}'.encode()for v in data) if kind=='buffer' else data)
        if target in ('field','arg_field'):
            value=f'Operand::Object {{object_id:{count}}}'
            s+=f'expected_frame.deferred_write=DeferredWrite::Field {{object:2,source:{value},result:{value}}};'
        else:s+=f'expected_frame.operations[0].count=1;expected_frame.operations[0].operands[0]=Operand::Object {{object_id:{count}}};'
    left=row['left'] or operand(a,0);right=row['right'] or operand(b,1)
    s+=f'let outcome:ExecutionOutcome=retire_concat(&input,&mut store,&mut frame,{left},{right},{dest});'
    s+='let store_before:bool=fx_store(&store,&expected_store);let frame_before:bool=ba_frame(&frame,&expected_frame);'
    if target in ('field','arg_field') and not row['error']:
        s+=f'let completed:ExecutionOutcome=complete_field_write(&input,&mut store,&mut frame,Operand::Integer {{number:999}});expected_frame.deferred_write=DeferredWrite::None;expected_frame.operations[0].count=1;expected_frame.operations[0].operands[0]=Operand::Object {{object_id:{count}}};'
    else:s+='let completed:ExecutionOutcome=ExecutionOutcome::Success;'
    if control:s+='expected_frame.lookup_cache[15].object_id=72;'
    s+='let store_after:bool=fx_store(&store,&expected_store);let frame_after:bool=ba_frame(&frame,&expected_frame);'
    return s+f'transition outcome==ExecutionOutcome::{row["error"] or "Success"} && completed==ExecutionOutcome::Success && store_before && frame_before && store_after && frame_after {{true -> (0) _ -> (1)}}\n'

def render(match=''):
    rows=[r for r in cases() if not match or r['name'] in match.split(',')];assert rows
    retired=runpy.run_path(str(RETIRE_FIXTURES))
    source=COMPARATOR.read_text().split('machine ba_check(')[0]
    source=source.replace(' let arrays:bool=ba_frame_arrays(a,b,0,16,true);',' let arrays:bool=ba_frame_arrays(a,b,0,16,true);let pending:bool=fw_deferred(a.deferred_write,b.deferred_write);')
    source=source.replace(' scope && body && result && arrays &&',' pending && a.field_writes==b.field_writes && scope && body && result && arrays &&')
    source+='use execution::execution_model::DeferredWrite;\nuse execution::concat_execution::retire_concat;\nuse execution::write_retirement::complete_field_write;\n'+retired['DEFERRED']
    source+='\nuse pipeline::program::Program;\nuse pipeline::program::Prepared;\nuse pipeline::program::prepare_program;\nuse pipeline::program::run_program;\nuse execution::engine::ExecutionResult;\nuse aml::byte_storage::ByteRead;\nuse aml::byte_storage::ByteOutcome;\nuse aml::byte_storage::ByteKind;\nuse aml::byte_storage::read_bytes;\nmachine cx_returned(result:ExecutionResult,id:u64)->bool {\n transition result.operand {Operand::Object {object_id} -> (object_id==id && result.has_value && result.outcome==ExecutionOutcome::Success) _ -> (false)}\n}\n'
    names=[]
    for row in rows:
        for control in (False,True):
            label='Suite::'+row['name']+('_control' if control else '_positive');names.append(label+'='+str(int(control)))
            source+='machine '+label+'(&mut self)->i32 {\n'+body(row,control)+'}\n'
    return rows,source,names


def aml_integer(n):
    if n in (0,1):return bytes([n])
    for width,op in [(1,10),(2,11),(4,12),(8,14)]:
        if n<1<<(width*8):return bytes([op])+n.to_bytes(width,'little')
    raise ValueError(n)
def aml_value(value):
    kind,data=value
    if kind in ('inline','integer'):return aml_integer(data)
    if kind=='string':return b'\x0d'+data+b'\0'
    body=aml_integer(len(data))+data;assert len(body)+1<64
    return b'\x11'+bytes([len(body)+1])+body

def bytecode_body(row,control):
    size=row['size'];a,b=row['a'],row['b'];mode=row['bytecode'];kind,data=result(a,b,size)
    target=b'\x60' if mode=='local' else b'LEFT' if mode=='self' else b'\0'
    expression=b'\x73LEFTRGHT'+target
    if mode=='nested':expression=b'\x73'+expression+aml_integer(3)+b'\0'
    body=b'MAIN\0\xa4'+expression;assert len(body)+1<64
    aml=b'\x08LEFT'+aml_value(a)+b'\x08RGHT'+aml_value(b)+b'\x14'+bytes([len(body)+1])+body
    s='let mut input:[u8;1024];'+''.join(f'input[{i}]={v};'for i,v in enumerate(aml) if v)
    s+=f'let prepared:Prepared=prepare_program(input,{len(aml)},7,128,128);let mut program:Program=prepared.program;let mut expected_store:ObjectStore=program.store;'
    s+=install('expected_store',3,kind,data);result_id=3;count=4
    if mode=='local':s+=install('expected_store',4,kind,data);count=5
    if mode=='self':s+=install('expected_store',0,kind,data)
    if mode=='nested':
        kind,data=result((kind,data),('inline',3),size);s+=install('expected_store',4,kind,data);result_id=4;count=5
    s+=f'expected_store.space.object_count={count};'
    if control:s+='expected_store.bytes.blocks[63].bytes[255]=1;'
    s+='let mut path:Path=Path {absolute:true,count:1};path.segments[0]=1313423693;let arguments:[Value;7];'
    s+=f'let returned:ExecutionResult=run_program(&mut program,path,&arguments,0,IntegerSize::{"FourBytes" if size==32 else "EightBytes"},1024);'
    s+=f'let identity:bool=cx_returned(returned,{result_id});let same:bool=fx_store(&program.store,&expected_store);'
    return s+'transition prepared.outcome==aml::model::Outcome::Success && identity && same {true -> (0) _ -> (1)}\n'
