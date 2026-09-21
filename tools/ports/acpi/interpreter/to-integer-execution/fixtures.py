"""Actual ToInteger bytecode and complete-state retirement witnesses."""
import importlib.util
import json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
NAMED=HERE.parent/'named-store-execution'
spec=importlib.util.spec_from_file_location('named_fixture',NAMED/'fixtures.py')
ns=importlib.util.module_from_spec(spec);spec.loader.exec_module(ns)
GENERIC=ns.GENERIC
integer,pkg,ret,op,name=ns.integer,ns.pkg,ns.ret,ns.op,ns.name
MAX=(1<<64)-1

def execution_cases():
    rows=[]
    string=lambda data:b'\x0d'+data+b'\0'
    buffer=lambda data:pkg(0x11,integer(len(data))+data)
    def add(label,source,target=None,number=42,bits=64,error='Success',body=None,setup=None,after=None,declarations=b''):
        target=integer(7) if target is None else target
        body=ret(op(0x99,name('SRC'),name('DST'))) if body is None else body
        table=b'\x08'+name('SRC')+source+b'\x08'+name('DST')+target+declarations+pkg(0x14,name('MAIN')+b'\0'+body)
        rows.append(dict(name=label,table=table.hex(),kind=1,number=number,bytes='',bits=bits,
                         after=({'DST':number} if error=='Success' else {'DST':7}) if after is None else after,
                         object_count=3,error=error,note='',setup=setup or [],mutate_source=False))
    for bits in (32,64):
        for source_name,source,number in [('integer',integer(MAX),MAX&((1<<bits)-1)),('decimal',string(b'42'),42),('buffer',buffer(b'\x2a\x01\x02\x03\x04\x05\x06\x07\x08'),int.from_bytes(b'\x2a\x01\x02\x03\x04\x05\x06\x07'[:bits//8],'little'))]:
            for target_name,target in [('integer',integer(7)),('string',string(b'old')),('buffer',buffer(b''))]:
                add(f'{source_name}_to_{target_name}_{bits}',source,target,number,bits)
        add(f'hex_null_{bits}',string(b'0x2A'),bits=bits,body=ret(op(0x99,name('SRC'),b'\0')),after={'DST':7})
        add(f'local_target_{bits}',string(b'42'),bits=bits,body=op(0x99,name('SRC'),b'\x60')+ret(b'\x60'),after={'DST':7})
        add(f'nested_add_{bits}',string(b'42'),number=43,bits=bits,body=ret(op(0x72,op(0x99,name('SRC'),b'\0'),integer(1),b'\0')),after={'DST':7})
        add(f'overflow_{bits}',string(str(1<<bits).encode()),bits=bits,error='Overflow')
    for label,source,error in [('empty_string',string(b''),'Empty'),('empty_buffer',buffer(b''),'Empty'),('space',string(b' 42'),'Encoding'),('suffix',string(b'42x'),'Encoding'),('sign',string(b'+42'),'Encoding'),('package',pkg(0x12,b'\0'),'UnsupportedValue')]:
        add(label,source,error=error)
    for kind in ['Named','Local','Arg']:
        add('transparent_'+kind.lower(),integer(0),string(b'42'),setup=[f'program.store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:1}};'])
    for kind in ['RefOf','Index']:
        add('explicit_'+kind.lower(),integer(0),error='UnsupportedValue',setup=[f'program.store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:1}};'])
    add('cycle',integer(0),error='InvalidState',setup=['program.store.space.objects[0].value=Value::Reference {kind:ReferenceKind::Named,object_id:0};'])
    add('malformed_owned',integer(0),error='InvalidState',setup=['program.store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:1}};'])
    add('uninitialized',integer(0),error='Uninitialized',setup=['program.store.space.objects[0].value=Value::Uninitialized;'])
    add('alias_replaces_string',string(b'42'),string(b'old'),body=ret(op(0x99,name('SRC'),name('ALS'))),declarations=op(0x06,name('DST'),name('ALS')))
    add('debug_target',string(b'42'),error='UnresolvedService',body=ret(op(0x99,name('SRC'),b'\x5b\x31')))
    add('full_arena_inline',integer(42),string(b'old'),setup=['program.store.space.object_count=64;'])
    rows[-1]['object_count']=64
    return rows

def bridge_cases():
    rows=[]
    def add(label,setup='',expected='',operand='Operand::Integer {number:42}',target='Target::Named {object_id:1}',bits=64,error='Success',number=42):
        if error=='Success':expected+=f'expected_frame.operations[0].count=1;expected_frame.operations[0].operands[0]=Operand::Integer {{number:{number}}};'
        rows.append(dict(name=label,setup=setup,expected=expected,operand=operand,target=target,bits=bits,error=error,scalar=False,copy=False))
    scalar='expected_store.space.objects[1].value=Value::Integer {number:42};expected_store.bytes.blocks[1]=ByteBlock {};'
    string='store.space.objects[0].value=Value::String {string_storage:StringStorage::Source {string_source:Span {unit:7,end:2}}};'
    oldstring='store.space.objects[1].value=Value::String {string_storage:StringStorage::Owned {string_owner:1}};store.bytes.blocks[1]=ByteBlock {initialized:true,length:3};store.bytes.blocks[1].bytes[0]=111;store.bytes.blocks[1].bytes[1]=108;store.bytes.blocks[1].bytes[2]=100;store.bytes.blocks[1].bytes[255]=91;'
    for bits in (32,64):
        add(f'decimal_replaces_string_{bits}',string+oldstring,scalar,'Operand::Object {object_id:0}',bits=bits)
        add(f'self_replacement_{bits}','store.space.objects[1].value=Value::String {string_storage:StringStorage::Source {string_source:Span {unit:7,end:2}}};',scalar,'Operand::Object {object_id:1}',bits=bits)
        add(f'local_{bits}',string,'expected_frame.locals[0]=Binding::Integer {number:42};','Operand::Object {object_id:0}',target='Target::Local {local_index:0}',bits=bits)
        add(f'argument_{bits}',string,'expected_frame.arguments[0]=Binding::Integer {number:42};','Operand::Object {object_id:0}',target='Target::Argument {argument_index:0}',bits=bits)
        add(f'null_{bits}',string,operand='Operand::Object {object_id:0}',target='Target::Null',bits=bits)
    for kind in ['Named','Local','Arg']:
        add('transparent_'+kind.lower(),f'store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:2}};store.space.objects[2].value=Value::String {{string_storage:StringStorage::Source {{string_source:Span {{unit:7,end:2}}}}}};',scalar,'Operand::Object {object_id:0}')
    for kind in ['RefOf','Index']:
        add('explicit_'+kind.lower(),f'store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:2}};',operand='Operand::Object {object_id:0}',error='UnsupportedValue')
    add('cycle','store.space.objects[0].value=Value::Reference {kind:ReferenceKind::Named,object_id:0};',operand='Operand::Object {object_id:0}',error='ReferenceCycle')
    add('source_max',operand=f'Operand::Object {{object_id:{MAX}}}',error='InvalidState')
    add('target_max',target=f'Target::Named {{object_id:{MAX}}}',error='MissingObject')
    add('uninitialized',operand='Operand::Uninitialized',error='Uninitialized')
    add('empty_source','store.space.objects[0].value=Value::String {string_storage:StringStorage::Owned {string_owner:0}};store.bytes.blocks[0]=ByteBlock {initialized:true};',operand='Operand::Object {object_id:0}',error='Empty')
    add('encoding_before_target','store.space.objects[0].value=Value::String {string_storage:StringStorage::Owned {string_owner:0}};store.bytes.blocks[0]=ByteBlock {initialized:true,length:1};store.bytes.blocks[0].bytes[0]=255;',operand='Operand::Object {object_id:0}',target=f'Target::Named {{object_id:{MAX}}}',error='Encoding')
    add('malformed_target_rejected',oldstring+'store.bytes.blocks[1].length=257;',error='Capacity')
    add('full_arena','store.space.object_count=64;',scalar)
    add('debug_target',target='Target::Debug',error='UnresolvedService')
    add('unresolved_target','store.space.objects[1].value=Value::OperationRegion {};',error='UnresolvedRegion')
    return rows

def render(group,match=''):
    rows=execution_cases() if group=='execution' else bridge_cases()
    rows=[r for r in rows if any(part in r['name'] for part in match.split(','))];assert rows
    if group=='execution':source,names=ns.base.render(rows)
    else:
        source,names=ns.render_bridge(rows)
        source+='\nuse execution::to_integer_execution::retire_to_integer;\n'
        source=source.replace('input[0]=50;input[1]=65;','input[0]=52;input[1]=50;')
        source=source.replace('let mut expected_store:ObjectStore=', 'frame.operation_count=1;frame.operations[0]=Operation {opcode:0xa4,value_arity:1};frame.arguments[0]=Binding::Shared {shared_id:0};let mut expected_store:ObjectStore=')
        for row in rows:
            old=f'write_generic_target(&input,&mut store,&mut frame,{row["target"]},{row["operand"]},false)'
            new=f'retire_to_integer(&input,&mut store,&mut frame,{row["operand"]},{row["target"]})'
            source=source.replace(old,new)
    return rows,source,names

if __name__=='__main__':
    for group in ['execution','bridge']:
        rows,source,names=render(group);(HERE/f'{group}-cases.json').write_text(json.dumps(rows,indent=2)+'\n');print(group,len(rows),'pairs')
