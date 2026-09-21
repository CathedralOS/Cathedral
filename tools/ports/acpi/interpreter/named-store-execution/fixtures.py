"""Actual AML execution and complete-store/frame named Store bridge witnesses."""
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
GENERIC = HERE.parent / 'generic-execution'
sys.path.insert(0, str(GENERIC))
spec = importlib.util.spec_from_file_location('generic_fixtures', GENERIC / 'fixtures.py')
base = importlib.util.module_from_spec(spec)
spec.loader.exec_module(base)
integer, pkg, ret, op, name = base.integer, base.pkg, base.ret, base.op, base.name
MAX = (1 << 64) - 1


def execution_cases():
    rows = []
    buffer = lambda data: pkg(0x11, integer(len(data)) + data)
    string = lambda data: b'\x0d' + data + b'\0'

    def add(label, source, target, kind, number=0, data=b'', bits=64, body=None, setup=None, error='Success', count=3, declarations=b''):
        table = b'\x08' + name('SRC') + source + b'\x08' + name('DST') + target
        table += declarations
        table += pkg(0x14, name('MAIN') + b'\0' + (body if body is not None else op(0x70, name('SRC'), name('DST')) + ret(name('DST'))))
        rows.append(dict(name=label, table=table.hex(), kind=kind, number=number, bytes=data.hex(), bits=bits,
                         after={}, object_count=count, error=error, note='', setup=setup or [], mutate_source=False))

    for bits in (32, 64):
        width = bits // 8
        number = 0x123456789ABCDEF0 & ((1 << bits) - 1)
        for source_kind, source, scalar, raw in (
            ('integer', integer(0x123456789ABCDEF0), number, number.to_bytes(width, 'little')),
            ('string', string(b'2A'), 42, b'2A\0'),
            ('buffer', buffer(b'\x2a\x01\x03'), 0x03012A, b'\x2a\x01\x03'),
        ):
            for target_kind, target, kind in (('integer', integer(7), 1), ('string', string(b'old'), 3), ('buffer', buffer(b'zzzzz'), 2)):
                if target_kind == 'integer':
                    add(f'{source_kind}_to_{target_kind}_{bits}', source, target, kind, number=scalar, bits=bits)
                else:
                    text = (f'{number:0{width * 2}X}'.encode() if source_kind == 'integer' else b'2A' if source_kind == 'string' else b'2A 01 03')
                    data = text if target_kind == 'string' else raw[:5].ljust(5, b'\0')
                    add(f'{source_kind}_to_{target_kind}_{bits}', source, target, kind, data=data, bits=bits, error='UnsupportedValue' if source_kind == target_kind == 'buffer' else 'Success')
        for target, kind, expected in ((string(b'old'), 3, f'{42:0{width*2}X}'.encode()), (buffer(b'zzzzz'), 2, b'\x2a\0\0\0\0')):
            add(f'add_result_to_{kind}_{bits}', integer(40), target, kind, data=expected, bits=bits,
                body=op(0x72, name('SRC'), integer(2), name('DST')) + ret(name('DST')))
        add(f'divide_named_results_{bits}', integer(42), string(b'old'), 3, data=f'{2:0{width*2}X}'.encode(), bits=bits,
            body=op(0x78, name('SRC'), integer(5), name('DST'), name('SRC')) + ret(name('DST')))
        rows[-1]['after'] = {'SRC': 8}
        add(f'store_expression_converted_integer_{bits}', string(b'2A'), integer(7), 1, number=42, bits=bits,
            body=ret(op(0x70, name('SRC'), name('DST'))))
        rows[-1]['after'] = {'DST': 42}
        add(f'store_expression_converted_string_{bits}', integer(42), string(b'old'), 3, data=f'{42:0{width*2}X}'.encode(), bits=bits,
            body=ret(op(0x70, name('SRC'), name('DST'))))
        add(f'store_expression_converted_buffer_{bits}', integer(42), buffer(b'zzzzz'), 2, data=b'\x2a\0\0\0\0', bits=bits,
            body=ret(op(0x70, name('SRC'), name('DST'))))
    add('named_alias_preserves_type', string(b'2A'), integer(7), 1, number=42,
        body=op(0x70, name('SRC'), name('ALS')) + ret(name('DST')),
        declarations=op(0x06, name('DST'), name('ALS')))
    for kind in ('Named', 'Local', 'Arg'):
        add('transparent_source_' + kind.lower(), integer(0), integer(7), 1, number=7,
            setup=[f'program.store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:1}};'])
    for kind in ('RefOf', 'Index'):
        add('explicit_source_' + kind.lower(), integer(0), integer(7), 1, error='UnsupportedValue',
            setup=[f'program.store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:1}};'])
    add('source_cycle', integer(0), integer(7), 1, error='InvalidState',
        setup=['program.store.space.objects[0].value=Value::Reference {kind:ReferenceKind::Named,object_id:0};'])
    add('empty_buffer_destination', integer(9), buffer(b''), 2, error='Bounds')
    add('empty_buffer_source', buffer(b''), integer(9), 1, error='Empty')
    add('empty_string_source', string(b''), integer(9), 1, error='Empty')
    add('buffer_string_capacity', buffer(bytes([42]) * 86), string(b'old'), 3, error='Capacity')
    add('full_arena_scalar_no_allocation', integer(42), string(b'old'), 3, data=b'000000000000002A', count=64,
        setup=['program.store.space.object_count=64;'])
    add('copyobject_still_replaces_type', string(b'2A'), integer(7), 3, data=b'2A',
        body=op(0x9d, name('SRC'), name('DST')) + ret(name('DST')))
    add('store_local_still_replaces_type', string(b'2A'), integer(7), 3, data=b'2A', count=4,
        body=op(0x70, integer(7), b'\x60') + op(0x70, name('SRC'), b'\x60') + ret(b'\x60'))
    return rows


def bridge_cases():
    rows = []
    def add(label, setup='', expected='', operand='Operand::Integer {number:42}', error='Success', bits=64, scalar=False, target='Target::Named {object_id:1}', copy=False):
        rows.append(dict(name=label, setup=setup, expected=expected, operand=operand, error=error, bits=bits, scalar=scalar, target=target, copy=copy))
    for bits in (32, 64):
        for scalar in (False, True):
            prefix = f'width{bits}_{"integer" if scalar else "generic"}'
            text = f'{42:0{bits//4}X}'.encode()
            setup = 'store.space.objects[1].value=Value::String {string_storage:StringStorage::Owned {string_owner:1}};store.bytes.blocks[1]=ByteBlock {initialized:true,length:3};store.bytes.blocks[1].bytes[0]=111;store.bytes.blocks[1].bytes[1]=108;store.bytes.blocks[1].bytes[2]=100;'
            expected = f'expected_store.bytes.blocks[1]=ByteBlock {{initialized:true,length:{len(text)}}};' + ''.join(f'expected_store.bytes.blocks[1].bytes[{i}]={v};' for i,v in enumerate(text))
            add(prefix+'_string', setup, expected, bits=bits, scalar=scalar)
            setup = 'store.space.objects[1].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:1}};store.bytes.blocks[1]=ByteBlock {initialized:true,length:5};store.bytes.blocks[1].bytes[4]=99;'
            add(prefix+'_buffer', setup, 'expected_store.bytes.blocks[1]=ByteBlock {initialized:true,length:5};expected_store.bytes.blocks[1].bytes[0]=42;', bits=bits, scalar=scalar)
            add(prefix+'_integer', '', 'expected_store.space.objects[1].value=Value::Integer {number:42};expected_store.bytes.blocks[1]=ByteBlock {};', bits=bits, scalar=scalar)
    add('transparent_string_source', 'store.space.objects[0].value=Value::Reference {kind:ReferenceKind::Named,object_id:2};store.space.objects[2].value=Value::String {string_storage:StringStorage::Source {string_source:Span {unit:7,start:0,end:2}}};', 'expected_store.space.objects[1].value=Value::Integer {number:42};expected_store.bytes.blocks[1]=ByteBlock {};', 'Operand::Object {object_id:0}')
    add('self_string_source', 'store.space.objects[1].value=Value::String {string_storage:StringStorage::Source {string_source:Span {unit:7,start:0,end:2}}};', 'expected_store.space.objects[1].value=Value::String {string_storage:StringStorage::Owned {string_owner:1}};expected_store.bytes.blocks[1]=ByteBlock {initialized:true,length:2};expected_store.bytes.blocks[1].bytes[0]=50;expected_store.bytes.blocks[1].bytes[1]=65;', 'Operand::Object {object_id:1}')
    for kind in ('RefOf', 'Index'):
        add('explicit_'+kind, f'store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:2}};', operand='Operand::Object {object_id:0}', error='UnsupportedValue')
    add('cycle', 'store.space.objects[0].value=Value::Reference {kind:ReferenceKind::Named,object_id:0};', operand='Operand::Object {object_id:0}', error='ReferenceCycle')
    add('source_max', operand=f'Operand::Object {{object_id:{MAX}}}', error='InvalidState')
    add('uninitialized', operand='Operand::Uninitialized', error='Uninitialized')
    add('empty_source', 'store.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:0}};store.bytes.blocks[0]=ByteBlock {initialized:true};', operand='Operand::Object {object_id:0}', error='Empty')
    add('invalid_ascii', 'store.space.objects[0].value=Value::String {string_storage:StringStorage::Owned {string_owner:0}};store.bytes.blocks[0]=ByteBlock {initialized:true,length:1};store.bytes.blocks[0].bytes[0]=255;', operand='Operand::Object {object_id:0}', error='Encoding')
    add('zero_extent', 'store.space.objects[1].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:1}};store.bytes.blocks[1]=ByteBlock {initialized:true};', error='Bounds')
    add('destination_first', 'store.space.objects[1].value=Value::String {string_storage:StringStorage::Owned {string_owner:1}};store.bytes.blocks[1]=ByteBlock {initialized:true,length:257};', operand=f'Operand::Object {{object_id:{MAX}}}', error='Capacity')
    add('bad_owner', 'store.space.objects[1].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:0}};', error='InvalidState')
    add('target_max', target=f'Target::Named {{object_id:{MAX}}}', error='MissingObject')
    add('count_max', f'store.space.object_count={MAX};', error='MissingObject')
    add('uninitialized_destination_copy', 'store.space.objects[1].value=Value::Uninitialized;', 'expected_store.space.objects[1].value=Value::Integer {number:42};expected_store.bytes.blocks[1]=ByteBlock {};')
    add('full_arena', 'store.space.object_count=64;', 'expected_store.space.objects[1].value=Value::Integer {number:42};expected_store.bytes.blocks[1]=ByteBlock {};')
    add('null_target', target='Target::Null')
    add('package_target', 'store.space.objects[1].value=Value::Package {first:0,count:1};', error='UnsupportedValue')
    return rows


def render_bridge(rows):
    original = (GENERIC / 'focused/bridge-atomicity/main.omg').read_text()
    prefix = original[:original.index('machine Suite::atomic_uninitialized_0')]
    prefix += 'use execution::integer_target_bridge::write_integer_target;\n'
    bodies, selections = [], []
    for row in rows:
        for control in (False, True):
            machine = 'Suite::' + row['name'] + ('_control' if control else '_positive')
            selections.append(machine + '=' + str(int(control)))
            source = f'machine {machine}(&mut self)->i32 {{\n'
            source += '''let mut input:[u8;1024];input[0]=50;input[1]=65;
let mut store:ObjectStore=ObjectStore {};store.space.object_count=3;store.space.count=1;
store.space.entries[0]=Entry {alias:true,object:1};store.space.entries[0].path.segments[15]=71;
store.space.objects[0]=Object {value:Value::Integer {number:42},has_next:true,next:2};
store.space.objects[1]=Object {value:Value::Integer {number:7},has_next:true,next:0};
store.space.objects[2].value=Value::Integer {number:99};store.bytes.blocks[63].bytes[255]=77;
store.bytes.blocks[1].bytes[255]=91;
'''
            source += row['setup']
            source += f'let mut frame:Frame=Frame {{source_length:2,body:Span {{unit:7,end:2}},end:2,size:IntegerSize::{"FourBytes" if row["bits"]==32 else "EightBytes"}}};'
            source += 'frame.arguments[6]=Binding::Shared {shared_id:2};frame.locals[7]=Binding::Integer {number:81};frame.lookup_cache[15].object_id=71;frame.operations[15].operands[6]=Operand::Integer {number:59};'
            source += 'let mut expected_store:ObjectStore=ObjectStore {space:store.space,bytes:store.bytes};let mut expected_frame:Frame=frame;'
            source += row['expected']
            if control:
                source += 'expected_frame.lookup_cache[15].object_id=72;'
            if row['scalar']:
                call = f'write_integer_target(&input,&mut store,&mut frame,{row["target"]},42,{str(row["copy"]).lower()})'
            else:
                call = f'write_generic_target(&input,&mut store,&mut frame,{row["target"]},{row["operand"]},{str(row["copy"]).lower()})'
            source += f'let outcome:ExecutionOutcome={call};let objects:bool=fx_store(&store,&expected_store);let callframe:bool=ba_frame(&frame,&expected_frame);transition outcome==ExecutionOutcome::{row["error"]} && objects && callframe {{true -> (0) _ -> (1)}}\n}}'
            bodies.append(source)
    return prefix + '\n'.join(bodies) + '\n', selections


def render(group, match=''):
    rows = execution_cases() if group == 'execution' else bridge_cases()
    rows = [r for r in rows if any(part in r['name'] for part in match.split(','))]
    assert rows
    source, names = base.render(rows) if group == 'execution' else render_bridge(rows)
    return rows, source, names


if __name__ == '__main__':
    for group in ('execution', 'bridge'):
        rows, source, names = render(group)
        (HERE / f'{group}-cases.json').write_text(json.dumps(rows, indent=2) + '\n')
        print(group, len(rows), 'behavior/control pairs')
