#!/usr/bin/env python3
"""Independent interval vectors for one selected native write recipe."""
import argparse
import importlib.util
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
BASE = HERE.parent/'field-writes'
sys.path.insert(0, str(BASE))
import vectors
import geometry_vectors as geometry
spec = importlib.util.spec_from_file_location('bulk_fixtures', BASE/'fixtures.py')
bulk = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bulk)
MAX = (1 << 64)-1


def selected(row, index, name=None):
    row = dict(row, name=name or row['name']+'_chunk_'+str(index), index=index)
    plan = row['geometry']
    if 'error' in plan:
        expected = dict(error='Geometry', cause=plan['error'])
    elif row['payload_length'] != (row['length']+7)//8 or row['payload_length'] > 256:
        expected = dict(error='PayloadLength')
    elif index >= plan['count']:
        expected = dict(error='Chunk', index=index, cause='InvalidChunk')
    else:
        chunk = plan['chunks'][index]
        old = row['previous'][index]
        update = (row['flags'] >> 5) & 3
        if chunk['partial'] and update == 0 and old is None:
            expected = dict(error='Chunk', index=index, cause='NeedsPrevious')
        else:
            base = (old or 0) if update == 0 and chunk['partial'] else MAX if update == 1 and chunk['partial'] else 0
            payload = int.from_bytes(bytes(row['payload']), 'little')
            value = (payload >> chunk['field_bit']) & ((1 << chunk['bit_count'])-1)
            native = ((base & ~chunk['mask']) | (value << chunk['native_bit'])) & ((1 << (8*chunk['width']))-1)
            expected = dict(offset=chunk['offset'], width=chunk['width'], value=native, locked=bool(row['flags'] & 16))
    row['expected'] = expected
    row['previous_word'] = row['previous'][index] if index < len(row['previous']) else None
    return row


def cases():
    rows = []
    for code in range(5):
        for update in range(3):
            for size in [4, 8]:
                flags = code | (update << 5) | (16 if size == 4 else 0)
                plan = geometry.plan(3, 131, flags, 300, size)
                row = vectors.make(dict(name=f'width_{code}_update_{update}_size_{size}', offset=3, length=131,
                                        flags=flags, region=300, size=size, patch='', expected=plan),
                                   missing=range(257) if update else (), poison=True)
                for index in sorted({0, plan['count']//2, plan['count']-1}):
                    rows.append(selected(row, index))
    old = vectors.cases()
    for name in ['capacity_1', 'full_absent', 'interior_absent', 'late_missing', 'first_missing',
                 'flat_1_4', 'flat_2_4']:
        row = next(row for row in old if row['name'] == name)
        count = row['geometry']['count']
        for index in sorted({0, count//2, count-1}):
            rows.append(selected(row, index))
    for row in old:
        if row['expected'].get('error') == 'Geometry':
            rows.append(selected(dict(row, payload_length=MAX), MAX, 'geometry_first_'+row['name']))
        elif row['name'].startswith('payload_'):
            rows.append(selected(row, MAX, row['name']+'_before_index'))
    for name in ['geometry_0_0_1', 'geometry_2_1_17', 'capacity_1']:
        row = next(row for row in old if row['name'] == name)
        for index in sorted({row['geometry']['count'], 257, 258, 1 << 63, MAX}):
            rows.append(selected(row, index, 'invalid_'+name+'_'+str(index)))
    rows.append(dict(name='default_failure', kind='default'))
    assert len({row['name'] for row in rows}) == len(rows)
    return rows


HEAD = bulk.HEAD + '''use writes::model::ChunkWriteResult;
data DefaultChunk [copy] {result:ChunkWriteResult;}
machine chunk_write(result:ChunkWriteResult,expected:NativeWrite,expected_lock:LockRequirement)->bool {
 transition result {
  ChunkWriteResult::Write {record,lock} -> (record.offset==expected.offset && record.width==expected.width && record.value==expected.value && lock==expected_lock)
  _ -> (false)
 }
}
machine chunk_error(result:ChunkWriteResult,expected:WriteError)->bool {
 transition result {ChunkWriteResult::Failure {error} -> compare(error,expected) _ -> (false)}
 state compare(actual:WriteError,expected:WriteError)->bool {let same:bool=same_error(actual,expected);same}
}
machine same_error(actual:WriteError,expected:WriteError)->bool {
 transition actual {
  WriteError::Geometry {cause} -> geometry(expected,cause)
  WriteError::PayloadLength -> payload(expected)
  WriteError::CountMismatch -> count(expected)
  WriteError::Chunk {index,cause} -> chunk(expected,index,cause)
 }
 state geometry(expected:WriteError,actual:Error)->bool {transition expected {WriteError::Geometry {cause} -> (cause==actual) _ -> (false)}}
 state payload(expected:WriteError)->bool {transition expected {WriteError::PayloadLength -> (true) _ -> (false)}}
 state count(expected:WriteError)->bool {transition expected {WriteError::CountMismatch -> (true) _ -> (false)}}
 state chunk(expected:WriteError,index:u64,cause:Error)->bool {let wanted_index:u64=index;let wanted_cause:Error=cause;transition expected {WriteError::Chunk {index,cause} -> (index==wanted_index && cause==wanted_cause) _ -> (false)}}
}
'''
def body(row, control=False):
    if row.get('kind') == 'default':
        setup = 'let initial:DefaultChunk=DefaultChunk {};let result:ChunkWriteResult=initial.result;'
        expected = dict(error='Geometry', cause='InvalidFlags')
    else:
        setup = f'let access:AccessResult=flags::decode_flags({row["flags"]});let mut field:Field=Field {{bit_offset:{row["offset"]},bit_length:{row["length"]},access:access.access}};field.access.flags={row["flags"]};let mut kind:DeclarationKind=DeclarationKind::Field;'+row['patch']
        setup += 'let mut input:Bytes=Bytes {};let poisoned:bool=poison(&mut input.bytes,0,256);'
        for index, value in enumerate(row['payload']):
            setup += f'input.bytes[{index}]={value};'
        previous = 'Previous::Absent' if row['previous_word'] is None else f'Previous::Value {{word:{row["previous_word"]}}}'
        size = 'FourBytes' if row['size'] == 4 else 'EightBytes'
        setup += f'let result:ChunkWriteResult=write::assemble_chunk(kind,&field,{row["region"]},IntegerSize::{size},&input.bytes,{row["payload_length"]},{row["index"]},{previous});'
        expected = row['expected']
    if 'error' in expected:
        if control:
            error = 'WriteError::CountMismatch'
        elif expected['error'] == 'Geometry':
            error = 'WriteError::Geometry {cause:Error::'+expected['cause']+'}'
        elif expected['error'] == 'Chunk':
            error = f'WriteError::Chunk {{index:{expected["index"]},cause:Error::{expected["cause"]}}}'
        else:
            error = 'WriteError::'+expected['error']
        check = f'chunk_error(result,{error})'
    else:
        lock = 'UnmetGlobalLock' if expected['locked'] else 'NotRequested'
        record = f'NativeWrite {{offset:{expected["offset"]},width:{expected["width"]},value:{expected["value"] ^ int(control)}}}'
        check = f'chunk_write(result,{record},LockRequirement::{lock})'
    return setup+'let good:bool='+check+';transition good {true -> (0) _ -> (1)}\n'


def render(rows):
    source = HEAD+'data Suite {}\n'
    entries = []
    for row in rows:
        for control in [False, True]:
            name = 'Suite::'+row['name']+('_control' if control else '_positive')
            source += 'machine '+name+'(&mut self)->i32 {\n'+body(row, control)+'}\n'
            entries.append(name+'='+str(int(control)))
    return source, entries


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write', action='store_true')
    args = parser.parse_args()
    rows = cases()
    if args.write:
        (HERE/'cases.json').write_text(json.dumps(rows, indent=2, sort_keys=True)+'\n')
    else:
        assert json.loads((HERE/'cases.json').read_text()) == rows
    print('PASS', len(rows), 'single-chunk fixture pairs')
