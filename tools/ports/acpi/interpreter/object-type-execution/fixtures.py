"""ObjectType through actual loader/engine, plus complete decoder state witnesses."""
import importlib.util
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
GENERIC = HERE.parent / 'generic-execution'
sys.path.insert(0, str(GENERIC))
spec = importlib.util.spec_from_file_location('object_type_generic_fixtures', GENERIC / 'fixtures.py')
base = importlib.util.module_from_spec(spec)
spec.loader.exec_module(base)
integer, pkg, ret, op, name = base.integer, base.pkg, base.ret, base.op, base.name
MAX = (1 << 64) - 1


def execution_cases():
    rows = []
    string = lambda data: b'\x0d' + data + b'\0'
    buffer = lambda data: pkg(0x11, integer(len(data)) + data)
    named = lambda label, value: b'\x08' + name(label) + value
    method = lambda label, count, body: pkg(0x14, name(label) + bytes([count]) + body)
    query = lambda operand: op(0x8e, operand)
    absolute = lambda *labels: b'\x5c' + (b'\x2e' if len(labels) == 2 else b'') + b''.join(map(name, labels))

    def add(label, prefix=b'', body=None, number=0, bits=64, setup=None, after=None, count=None, error='Success'):
        body = ret(query(name('OBJ'))) if body is None else body
        table = prefix + method('MAIN', 0, body)
        rows.append(dict(name=label, table=table.hex(), kind=1, number=number, bytes='', bits=bits,
                         after=after or {}, object_count=count, error=error, note='',
                         setup=setup or [], mutate_source=False))

    for bits in (32, 64):
        for label, payload, result in [('integer', integer(MAX), 1), ('string', string(b'AB'), 2),
                                       ('buffer', buffer(b'AB'), 3), ('package', pkg(0x12, b'\x01' + integer(9)), 4)]:
            add(f'named_{label}_{bits}', named('OBJ', payload), number=result, bits=bits)
        add(f'local_uninitialized_{bits}', body=ret(query(b'\x60')), bits=bits, count=1)
        add(f'argument_uninitialized_{bits}', body=ret(query(b'\x68')), bits=bits, count=1)
        add(f'local_private_integer_{bits}', body=op(0x70, integer(MAX), b'\x60') + ret(query(b'\x60')), number=1, bits=bits, count=1)
        add(f'local_private_string_{bits}', named('OBJ', string(b'AB')),
            op(0x9d, name('OBJ'), b'\x60') + ret(query(b'\x60')), 2, bits, count=3)
        add(f'argument_inline_{bits}', method('CAL', 1, ret(query(b'\x68'))),
            ret(name('CAL') + integer(MAX)), 1, bits, count=2)
        add(f'argument_shared_buffer_{bits}', named('OBJ', buffer(b'AB')) + method('CAL', 1, ret(query(b'\x68'))),
            ret(name('CAL') + name('OBJ')), 3, bits, count=3)
        add(f'debug_metadata_{bits}', body=ret(query(b'\x5b\x31')), number=16, bits=bits, count=1)
        prefix = named('FLAG', integer(7)) + method('CAL', 3, op(0x70, integer(99), name('FLAG')) + ret(integer(1)))
        add(f'parameterized_method_not_invoked_{bits}', prefix, ret(query(name('CAL'))), 8, bits, after={'FLAG': 7}, count=3)

    add('alias_preserves_identity', named('OBJ', string(b'AB')) + op(0x06, name('OBJ'), name('ALS')), ret(query(name('ALS'))), 2, count=2)
    add('typeless_scope', pkg(0x10, name('SCP')), ret(query(name('SCP'))), count=1)
    add('root_scope', body=ret(query(b'\x5c\0')), count=1)
    prefix = named('OBJ', integer(42)) + pkg(0x10, name('SCP') + pkg(0x10, name('OBJ')) + method('CAL', 0, ret(query(name('OBJ')))))
    add('near_scope_shadows_outer_object', prefix, ret(absolute('SCP', 'CAL')), count=3)
    prefix = named('OBJ', integer(0)) + named('DST', string(b'AB'))
    for kind in ('Named', 'Local', 'Arg', 'RefOf', 'Index', 'Unresolved'):
        add('reference_' + kind.lower(), prefix, number=2, count=3,
            setup=[f'program.store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:1}};'])
    setup = ['program.store.space.objects[0].value=Value::Reference {kind:ReferenceKind::RefOf,object_id:1};']
    add('local_reference_descriptor', prefix, op(0x9d, name('OBJ'), b'\x60') + ret(query(b'\x60')), 2, setup=setup, count=4)
    add('argument_reference_descriptor', prefix + method('CAL', 1, ret(query(b'\x68'))),
        ret(name('CAL') + name('OBJ')), 2, setup=setup, count=4)
    add('lexical_reference_descriptor', prefix, number=2, count=3,
        setup=['let mut lexical:Path=Path {count:1};lexical.segments[0]=1599361860;program.store.space.objects[0].value=Value::NameReference {name:lexical,scope:Path {absolute:true}};'])
    add('named_uninitialized', named('OBJ', integer(0)), count=2,
        setup=['program.store.space.objects[0].value=Value::Uninitialized;'])
    add('reference_cycle', named('OBJ', integer(0)), error='ReferenceCycle', count=2,
        setup=['program.store.space.objects[0].value=Value::Reference {kind:ReferenceKind::RefOf,object_id:0};'])
    add('reference_dangling', named('OBJ', integer(0)), error='InvalidState', count=2,
        setup=[f'program.store.space.objects[0].value=Value::Reference {{kind:ReferenceKind::Index,object_id:{MAX}}};'])
    add('unknown_name', error='MissingObject', count=1)
    add('truncated_operand', body=ret(b'\x8e'), error='Truncated', count=1)
    for label, operand in [('refof', op(0x71, name('OBJ'))), ('derefof', op(0x83, name('OBJ'))),
                           ('index', op(0x88, name('OBJ'), integer(0), b'\0')), ('literal', integer(1))]:
        add('excluded_' + label, named('OBJ', buffer(b'AB')), ret(query(operand)), error='UnsupportedOpcode', count=2)
    add('null_name', body=ret(query(b'\0')), error='InvalidTarget', count=1)
    add('malformed_owned_buffer_metadata', named('OBJ', buffer(b'AB')), number=3, count=2,
        setup=[f'program.store.space.objects[0].value=Value::Buffer {{buffer_storage:BufferStorage::Owned {{buffer_owner:{MAX}}}}};'])
    add('malformed_package_metadata', named('OBJ', integer(0)), number=4, count=2,
        setup=[f'program.store.space.objects[0].value=Value::Package {{first:{MAX},count:{MAX}}};'])
    add('malformed_method_body_metadata', method('OBJ', 7, b'\xde'), number=8, count=2,
        setup=[f'program.definitions.entries[0].present=false;program.store.space.objects[0].value=Value::Method {{flags:7,body:Span {{unit:{MAX},start:{MAX},end:{MAX}}}}};'])
    add('region_metadata_no_access', op(0x5b80, name('OBJ'), b'\0', integer(0), integer(16)), number=10, count=2)
    add('legacy_processor_code12', b'\x5b' + pkg(0x83, name('OBJ') + b'\0' * 6), number=12, count=2)
    add('malformed_buffer_field_metadata', named('OBJ', integer(0)), number=14, count=2,
        setup=[f'program.store.space.objects[0].value=Value::BufferField {{backing_object:{MAX},bit_offset:{MAX},bit_length:{MAX}}};'])
    return rows


def decoder_cases():
    rows = []
    def add(label, data, outcome='Success', result=0, setup='', frame_setup='', bits=64):
        rows.append(dict(name=label, input=data.hex(), outcome=outcome, result=result, setup=setup, frame_setup=frame_setup, bits=bits))
    for bits in (32, 64):
        add(f'decoder_local_zero_{bits}', b'\x8e\x60', bits=bits)
        add(f'decoder_inline_integer_{bits}', b'\x8e\x68', result=1, bits=bits,
            frame_setup='frame.arguments[0]=Binding::Integer {number:99};')
    add('decoder_named_no_store_write', b'\x8eOBJ_', result=1)
    add('decoder_debug_no_service', b'\x8e\x5b\x31', result=16)
    add('decoder_missing_no_publication', b'\x8eBAD_', 'MissingObject')
    add('decoder_truncated_no_publication', b'\x8eOBJ', 'Truncated')
    add('decoder_excluded_no_publication', b'\x8e\x71OBJ_', 'UnsupportedOpcode')
    add('decoder_cycle_no_publication', b'\x8eOBJ_', 'ReferenceCycle',
        setup='store.space.objects[0].value=Value::Reference {kind:ReferenceKind::RefOf,object_id:0};')
    add('decoder_bad_id_no_publication', b'\x8eOBJ_', 'InvalidState',
        setup=f'store.space.entries[0].object={MAX};')
    add('decoder_full_operand_no_publication', b'\x8eOBJ_', 'InvalidState',
        frame_setup='frame.operations[0].count=1;frame.operations[0].operands[0]=Operand::Integer {number:71};')
    return rows


def render_decoder(rows):
    original = (GENERIC / 'focused/bridge-atomicity/main.omg').read_text()
    source = original[:original.index('machine Suite::atomic_uninitialized_0')]
    source += '\nuse execution::decode_execution::decode_term;\n'
    selections = []
    for row in rows:
        data = bytes.fromhex(row['input'])
        for control in (False, True):
            machine = 'Suite::' + row['name'] + ('_control' if control else '_positive')
            selections.append(machine + '=' + str(int(control)))
            source += f'machine {machine}(&mut self)->i32 {{let mut input:[u8;1024];'
            source += ''.join(f'input[{i}]={byte};' for i, byte in enumerate(data))
            source += '''let mut store:ObjectStore=ObjectStore {};store.space.count=1;store.space.object_count=1;
store.space.entries[0]=Entry {path:Path {absolute:true,count:1},has_object:true};store.space.entries[0].path.segments[0]=1598702159;
store.space.objects[0]=Object {value:Value::Integer {number:42},has_next:true,next:63};store.bytes.blocks[63].bytes[255]=77;
'''
            source += row['setup']
            source += f'let mut frame:Frame=Frame {{scope:Path {{absolute:true}},source_length:{len(data)},body:Span {{unit:7,end:{len(data)}}},end:{len(data)},size:IntegerSize::{"FourBytes" if row["bits"]==32 else "EightBytes"},operation_count:1}};'
            source += 'frame.operations[0]=Operation {opcode:0xa4,value_arity:1};frame.locals[7]=Binding::Integer {number:81};frame.lookup_cache[15].object_id=71;frame.operations[15].operands[6]=Operand::Integer {number:59};'
            source += row['frame_setup']
            source += 'let expected_store:ObjectStore=ObjectStore {space:store.space,bytes:store.bytes};let mut expected_frame:Frame=frame;'
            if row['outcome'] == 'Success':
                source += f'expected_frame.pc={len(data)};expected_frame.operations[0].count=1;expected_frame.operations[0].operands[0]=Operand::Integer {{number:{row["result"]}}};'
            if control:
                source += 'expected_frame.lookup_cache[15].object_id=72;'
            source += f'let outcome:ExecutionOutcome=decode_term(&input,&store,&mut frame);let objects:bool=fx_store(&store,&expected_store);let callframe:bool=ba_frame(&frame,&expected_frame);transition outcome==ExecutionOutcome::{row["outcome"]} && objects && callframe {{true -> (0) _ -> (1)}}}}\n'
    return source, selections


def render(group, match=''):
    rows = execution_cases() if group == 'execution' else decoder_cases()
    rows = [row for row in rows if any(part in row['name'] for part in match.split(','))]
    assert rows
    source, names = base.render(rows) if group == 'execution' else render_decoder(rows)
    return rows, source, names


if __name__ == '__main__':
    for group in ('execution', 'decoder'):
        rows, _, _ = render(group)
        (HERE / f'{group}-cases.json').write_text(json.dumps(rows, indent=2) + '\n')
        print(group, len(rows), 'behavior/control pairs')
