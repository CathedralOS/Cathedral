"""Focused graph-accounting and diagnostic-preservation boundary witnesses."""
import importlib.util
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
PARENT = HERE.parent
spec = importlib.util.spec_from_file_location('boundary_fixtures', PARENT/'fixtures.py')
base = importlib.util.module_from_spec(spec)
spec.loader.exec_module(base)
ROOT, LEGACY = base.ROOT, base.LEGACY


def cases():
    original = next(row for row in base.cases() if row['name'] == 'exact_quotas')
    setup = '''program.store.space.objects[2].value=Value::Reference {kind:ReferenceKind::RefOf,object_id:5};
program.store.space.objects[5].value=Value::Reference {kind:ReferenceKind::Index,object_id:6};
program.store.space.objects[6].value=Value::Buffer {buffer_storage:BufferStorage::Owned {buffer_owner:6}};
program.store.bytes.blocks[6]=ByteBlock {initialized:true,length:4};
program.store.bytes.blocks[6].bytes[0]=68;program.store.bytes.blocks[6].bytes[1]=69;
program.store.bytes.blocks[6].bytes[2]=70;program.store.bytes.blocks[6].bytes[3]=71;
program.store.space.object_count=7;'''
    rows = []
    for name, objects, byte_count, outcome in [
        ('nested_reference_exact', 5, 7, 'Success'),
        ('nested_reference_object_short', 4, 7, 'WorkLimit'),
        ('nested_reference_byte_short', 5, 6, 'Capacity'),
    ]:
        row = dict(original, name=name, setup=setup, objects=objects, bytes=byte_count,
                   error=outcome, present=outcome == 'Success', count=7)
        rows.append(row)
    return rows


def render(rows):
    source, names = base.render(rows)
    source = source.replace('use execution::engine::ExecutionResult;',
                            'use execution::engine::ExecutionResult;\nuse execution::engine::run_method;')
    # Observe the same loaded state through the engine before applying the public
    # result boundary. This independently establishes exact diagnostic retention.
    baseline = '''let mut baseline:Program=program;
let before:ExecutionResult=run_method(&baseline.source,baseline.length,baseline.unit,&mut baseline.store,&baseline.definitions.entries,path,&arguments,0,IntegerSize::EightBytes,256);
let result:ExecutionResult='''
    source = source.replace('let result:ExecutionResult=', baseline)
    source = source.replace(' && value && effect && ',
        ' && value && effect && before.outcome==ExecutionOutcome::Success && before.has_value && result.steps==before.steps && result.fault_offset==before.fault_offset && ')
    return source, names


if __name__ == '__main__':
    (HERE/'cases.json').write_text(json.dumps(cases(), indent=2)+'\n')
    print(len(cases()), 'supplemental Program boundary cases')
