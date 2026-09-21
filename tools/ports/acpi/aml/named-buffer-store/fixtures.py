"""Equal-extent named Buffer copies and retained scalar/admission regressions."""
import copy
import importlib.util
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
BASE = HERE.parent / 'named-value-store-scalar'
sys.path.insert(0, str(BASE))
spec = importlib.util.spec_from_file_location('scalar_fixtures', BASE / 'fixtures.py')
scalar = importlib.util.module_from_spec(spec)
spec.loader.exec_module(scalar)
old = scalar.old
MAX = old.MAX
IMPORTS = scalar.IMPORTS
HELPERS = scalar.HELPERS
body = scalar.body


def regression_cases():
    rows = scalar.cases()
    changed = []
    for row in rows:
        if row['name'].startswith('buffer_self_'):
            row['expected'] = {'bytes': row['old']}
            changed.append(row['name'])
    assert changed == ['buffer_self_False', 'buffer_self_True']
    return rows


def cases():
    rows = []

    def add(name, extent=3, data=None, error=None, **kw):
        row = copy.deepcopy(old.cases()[0])
        row.update(name=name, target='Buffer', source='Buffer', old=[90]*extent,
                   data=list(data if data is not None else bytes((i*37+128)%256 for i in range(extent))),
                   number=0, bits=64, default=False)
        row.update(kw)
        row['expected'] = {'error': error} if error else {'bytes': row['data']}
        rows.append(row)

    # Every byte matters, including embedded zero and bytes outside String ASCII.
    for bits in [32, 64]:
        for owned in [False, True]:
            for source_owned in [False, True]:
                for extent in [0, 1, 3, 8, 256]:
                    add(f'copy_{bits}_{owned}_{source_owned}_{extent}', extent=extent,
                        bits=bits, owned=owned, source_owned=source_owned)
    for owned in [False, True]:
        for extent in [0, 3, 256]:
            add(f'self_{owned}_{extent}', extent=extent, data=bytes([90])*extent,
                owned=owned, source_id=0)
    add('shared_immutable_span', owned=False, source_owned=False,
        data=b'ZZZ', mutation='s.space.objects[1].value=s.space.objects[0].value;')
    add('declared_source_padding', source_owned=False, data=b'XYZ\0\0', extent=5,
        mutation='s.space.objects[1].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:5,buffer_initializer:Span {unit:7,start:512,end:515}}};')
    add('initializer_sets_extent', source_owned=False,
        mutation='s.space.objects[1].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:1,buffer_initializer:Span {unit:7,start:512,end:515}}};')
    add('destination_materialized_padding', extent=5, owned=False,
        mutation='s.space.objects[0].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:5,buffer_initializer:Span {unit:7,start:0,end:1}}};')
    add('last_slot_full_arena', destination=63, count=64,
        mutation='s.space.objects[63].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:63}};s.bytes.blocks[63].length=3;')
    add('self_only_allocated_slot', count=1, source_id=0, data=b'ZZZ')
    add('aliases_preserved', mutation='s.space.entries[1].object=0;s.space.entries[1].alias=true;')
    add('unrelated_entry_count_ignored', mutation=f's.space.count={MAX};')
    add('owned_input_metadata_ignored', length=MAX, unit=MAX)
    for extent, n in [(0,1),(1,0),(1,3),(3,1),(255,256),(256,255)]:
        add(f'extent_mismatch_{extent}_{n}', extent=extent, data=b'A'*n,
            error='Bounds' if extent==0 else 'UnsupportedValue')
    for source in [2,64,1<<63,MAX]:
        add(f'invalid_source_{source}', source_id=source, error='InvalidState')
    for destination in [2,64,MAX]:
        add(f'invalid_destination_{destination}', destination=destination, error='InvalidState')
    for count in [0,65,MAX]:
        add(f'invalid_count_{count}', count=count, error='InvalidState')
    failures = [
        ('wrong_owner', True, 's.space.objects[1].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:0}};', 'InvalidState'),
        ('uninitialized', True, 's.bytes.blocks[1].initialized=false;', 'InvalidState'),
        ('length_capacity', True, 's.bytes.blocks[1].length=257;', 'Capacity'),
        ('length_max', True, f's.bytes.blocks[1].length={MAX};', 'Capacity'),
        ('unit', False, 's.space.objects[1].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:3,buffer_initializer:Span {unit:8,start:512,end:515}}};', 'Bounds'),
        ('span_reversed', False, 's.space.objects[1].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:3,buffer_initializer:Span {unit:7,start:515,end:512}}};', 'Bounds'),
        ('span_end', False, 's.space.objects[1].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:3,buffer_initializer:Span {unit:7,start:1023,end:1025}}};', 'Bounds'),
        ('declared_capacity', False, 's.space.objects[1].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:257,buffer_initializer:Span {unit:7,start:512,end:515}}};', 'Capacity'),
        ('initializer_capacity', False, 's.space.objects[1].value=Value::Buffer {buffer_storage:BufferStorage::Source {declared_size:3,buffer_initializer:Span {unit:7,start:512,end:769}}};', 'Capacity'),
    ]
    for name, owned, mutation, error in failures:
        add('source_'+name, source_owned=owned, mutation=mutation, error=error)
        # Complete source admission precedes the equal-extent policy check.
        add('empty_destination_source_'+name, extent=0, data=b'XYZ',
            source_owned=owned, mutation=mutation, error=error)
    add('old_backing_before_bad_source', source_id=MAX,
        mutation='s.bytes.blocks[0].length=257;', error='Capacity')
    add('old_source_before_bad_source', owned=False, source_id=MAX, unit=8, error='Bounds')
    for kind in old.KINDS:
        add('wrapper_'+kind, mutation='s.space.objects[1].value=Value::Reference {kind:ReferenceKind::'+kind+',object_id:0};', error='UnsupportedValue')
    assert len({r['name'] for r in rows}) == len(rows)
    return rows
