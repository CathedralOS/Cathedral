"""Zero-destination boundaries with existing complete-store and byte comparators."""
import copy
import importlib.util
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
BASE = HERE.parent / 'named-value-store-scalar'
EXTENT = HERE.parent / 'buffer-target-values'
sys.path.insert(0, str(BASE))


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


scalar = load('scalar_fixtures', BASE / 'fixtures.py')
extent = load('extent_fixtures', EXTENT / 'fixtures.py')
old = scalar.old
MAX = old.MAX
IMPORTS = ''.join(line+';\n' for line in dict.fromkeys(
    token.strip() for token in (scalar.IMPORTS+extent.IMPORTS).split(';') if token.strip()))
HELPERS = scalar.HELPERS+extent.HELPERS


def cases():
    rows = []

    def add(name, source='Integer', data=b'', error=None, **kw):
        row = copy.deepcopy(old.cases()[0])
        row.update(name=name, target='Buffer', source=source, data=list(data),
                   old=[], number=MAX, default=False)
        row.update(kw)
        row['expected'] = {'error': error} if error else {'bytes': []}
        rows.append(row)

    for api in [None, 'scalar']:
        for bits in [32, 64]:
            for owned in [False, True]:
                for number in [0, 1 << 32, MAX]:
                    add(f'integer_{api}_{bits}_{owned}_{number}', api=api, bits=bits,
                        owned=owned, number=number)
    for owned in [False, True]:
        for source_owned in [False, True]:
            for n in [0, 1, 256]:
                add(f'string_{owned}_{source_owned}_{n}', source='String', data=b'A'*n,
                    owned=owned, source_owned=source_owned, bits=32 if n==0 else 64)
    # Full source admission applies even when no source bytes will be published.
    failures = [
        ('bad_owner', True, b'', 's.space.objects[1].value=Value::String {string_storage:StringStorage::Owned {string_owner:0}};', 'InvalidState'),
        ('missing_owned', True, b'', 's.bytes.blocks[1].initialized=false;', 'InvalidState'),
        ('owned_capacity', True, b'', 's.bytes.blocks[1].length=257;', 'Capacity'),
        ('owned_max', True, b'', f's.bytes.blocks[1].length={MAX};', 'Capacity'),
        ('source_unit', False, b'', 's.space.objects[1].value=Value::String {string_storage:StringStorage::Source {string_source:Span {unit:8,start:512,end:512}}};', 'Bounds'),
        ('source_reversed', False, b'', 's.space.objects[1].value=Value::String {string_storage:StringStorage::Source {string_source:Span {unit:7,start:513,end:512}}};', 'Bounds'),
        ('source_capacity', False, b'', 's.space.objects[1].value=Value::String {string_storage:StringStorage::Source {string_source:Span {unit:7,start:512,end:769}}};', 'Capacity'),
    ]
    for name, source_owned, data, mutation, error in failures:
        add(name, source='String', data=data, source_owned=source_owned, mutation=mutation, error=error)
    for owned in [False, True]:
        for byte in [0, 128]:
            add(f'full_encoding_{owned}_{byte}', source='String', data=b'A'*255+bytes([byte]),
                source_owned=owned, error='Encoding')
        add(f'empty_self_{owned}', source='Buffer', owned=owned, source_id=0)
        add(f'empty_string_positive_{owned}', source='String', data=b'', old=[90],
            source_owned=owned, error='Empty')
    add('source_id_max', source_id=MAX, error='InvalidState')
    add('source_unallocated', source_id=2, error='InvalidState')
    add('count_max', count=MAX, error='InvalidState')
    add('destination_before_source', source_id=MAX, mutation='s.bytes.blocks[0].length=257;', error='Capacity')
    add('empty_destination_owner', mutation='s.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:1}};', error='InvalidState')
    add('empty_destination_missing', api='scalar', mutation='s.bytes.blocks[0].initialized=false;', error='InvalidState')
    add('empty_destination_span', owned=False, unit=8, error='Bounds')
    for kind in old.KINDS:
        add('unsupported_'+kind, mutation='s.space.objects[1].value=Value::Reference {kind:ReferenceKind::'+kind+',object_id:0};', error='UnsupportedValue')
    add('unsupported_name', mutation='s.space.objects[1].value=Value::NameReference {name:Path {},scope:Path {}};', error='UnsupportedValue')
    add('unequal_buffer_zero', source='Buffer', data=b'A', error='Bounds')
    add('unequal_buffer_positive', source='Buffer', data=b'A', old=[90,90], error='UnsupportedValue')
    add('scalar_last_full', api='scalar', count=64, destination=63,
        mutation='s.space.objects[63].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:63}};s.bytes.blocks[63].length=0;')
    add('scalar_one_object', api='scalar', count=1, mutation='s.space.objects[1].value=Value::Device;')
    add('source_identity_alias', mutation='s.space.entries[1].object=0;s.space.entries[1].alias=true;')
    add('owned_irrelevant_input', source='String', data=b'', length=MAX, unit=MAX)
    # Reuse representative positive-extent cases without changing their oracle.
    names = ['buf_int_32_True_9_4294967296', 'buf_int_64_False_1_18446744073709551615',
             'buf_str_True_False_256_1', 'buf_str_False_True_1_256']
    selected = [copy.deepcopy(r) for r in old.cases() if r['name'] in names]
    assert len(selected) == len(names)
    rows += selected
    # Exercise the preparation API directly, including upper geometry precedence.
    def prep(name, kind='Integer', data=b'AB', error=None, **kw):
        row = copy.deepcopy(extent.cases()[0])
        row.update(name='prepare_'+name, api='extent', kind=kind, data=list(data),
                   extent=0, error=error, expected=[])
        row.update(kw)
        rows.append(row)
    prep('integer32', number=MAX, bits=32)
    prep('integer64', number=MAX, bits=64)
    prep('empty_source', kind='String', data=b'')
    prep('nonempty_owned', kind='String', owned=True)
    prep('full_encoding', kind='String', data=b'A'*255+b'\x80', error='Encoding')
    prep('empty_owner', kind='String', data=b'', owned=True,
         extra='store.bytes.blocks[0].initialized=false;', error='InvalidState')
    prep('invalid_id', source=MAX, error='InvalidState')
    prep('capacity_before_encoding', kind='String', data=b'\x80', extent=257, error='Capacity')
    prep('invalid_id_before_capacity', source=MAX, extent=257, error='InvalidState')
    prep('unsupported_buffer', kind='Buffer', error='UnsupportedValue')
    prep('positive_empty_excluded', kind='String', data=b'', extent=1, error='Empty')
    assert len({r['name'] for r in rows}) == len(rows)
    return rows


def body(row, control=False):
    return extent.body(row, control) if row.get('api') == 'extent' else scalar.body(row, control)
