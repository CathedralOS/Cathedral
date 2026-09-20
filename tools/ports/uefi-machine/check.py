#!/usr/bin/env python3
"""Audit machine protocol transcription against pinned UEFI-target Rust evidence."""
import hashlib
import json
from pathlib import Path
import re
import sys
import uuid

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'tools/ports'))
import inventory
import rust_layout
import vectors
HERE = Path(__file__).resolve().parent
RAW = ROOT / 'source/contracts/uefi/raw'
UPSTREAM = ROOT / 'reference_code/rust-osdev/uefi-rs'
SLICE = 'machine'


def require(condition, message):
    if not condition:
        raise ValueError(message)


def main():
    print(inventory.check(inventory.read_json(RAW / (SLICE + '-inventory.json')), UPSTREAM, require_transcribed=True))
    values = vectors.validate(inventory.read_json(RAW / (SLICE + '.vectors.json')))['measurements']
    measured, artifact = rust_layout.measure(HERE / 'Cargo.toml', HERE / 'measure.rs', 'CATHEDRAL_' + SLICE.upper() + '_LAYOUT', 'cathedral_uefi_' + SLICE + '_probe')
    require(set(measured) == {key for key, row in values.items() if row['kind'] != 'bytes'}, 'numeric measurement coverage differs')
    for key, value in measured.items():
        require(values[key]['value'] == value, key + ': Rust measurement differs')
    schema = inventory.read_json(HERE / 'schema.json')
    source = (RAW / (SLICE + '_protocols.omg')).read_text()
    plans = (RAW / (SLICE + '_layouts.omg')).read_text()
    for record in schema['records']:
        name = record['name']
        upstream = re.sub(r'//[^\n]*', '', (UPSTREAM / record['path']).read_text())
        if record.get('alias'):
            alias = re.search(r'pub type ' + name + r'\s*=\s*(.*?);', upstream, re.S)
            require(alias is not None and ' '.join(alias[1].split()) == record['alias'], name + ': upstream alias differs')
        elif record['base']:
            require(re.search(r'pub (?:struct|enum) ' + name + r':\s*' + record['base'] + r'\b', upstream) is not None,
                    name + ': upstream scalar carrier differs')
        else:
            upstream_body = re.search(r'pub struct ' + name + r'\s*{(.*?)\n}', upstream, re.S)[1]
            hits = list(re.finditer(r'\bpub\s+((?:r#)?\w+)\s*:', upstream_body))
            upstream_fields = [[hit[1], ' '.join(upstream_body[hit.end():hits[index + 1].start() if index + 1 < len(hits) else len(upstream_body)].strip().rstrip(',').split())]
                               for index, hit in enumerate(hits)]
            require(upstream_fields == record['fields'], name + ': upstream field metadata differs')
        body = re.search(r'pub data ' + name + r' \[copy\] {(.*?)\n}', source, re.S)[1]
        body = re.sub(r'//[^\n]*', '', body)
        actual = re.findall(r'^\s*(\w+): ([^;]+);', body, re.M)
        expected = []
        for field, typ in record['fields']:
            mapped = 'addr' if '*' in typ or 'fn(' in typ or 'fn (' in typ else {'Handle': 'addr', 'usize': 'u64', 'isize': 'i64', 'Boolean': 'u8', 'ResetSystemFn': 'addr', 'Status': 'u64', 'Event': 'addr', 'ShellFileHandle': 'addr'}.get(typ, typ)
            expected.append((field.removeprefix('r#'), mapped))
        require(actual == expected, name + ': field/type/order differs')
        plan = re.search(r'pub machine ' + name + r'X64Layout::plan.*?{(.*?)\n}', plans, re.S)[1]
        offsets = [int(number) for number in re.findall(r'FieldPlan::At { offset: (\d+) }', plan)]
        require(offsets == [measured.get(name + '.' + field + '.offset', 0) for field, typ in record['fields']], name + ': plan offsets differ')
        require(int(re.search(r'entry_count: (\d+)', plan)[1]) == len(expected), name + ': plan coverage differs')
        require(int(re.search(r'size_fixed: (\d+)', plan)[1]) == measured[name + '.size'], name + ': plan size differs')
        require(int(re.search(r'align: (\d+)', plan)[1]) == measured[name + '.alignment'], name + ': plan alignment differs')
    for const in schema['constants']:
        name = const['name']
        declaration = re.search(r'pub const ' + name + r': (.*?);', source)[1]
        if const['guid']:
            guid = uuid.UUID(const['spelling'])
            numbers = [int(number, 16) for number in re.findall(r'0x([0-9a-f]+)', declaration)]
            require(numbers == list(guid.fields[:3]) + list(guid.bytes[8:]), name + ': GUID fields differ')
            require(values[name]['value'] == guid.bytes_le.hex(), name + ': GUID bytes differ')
        else:
            literal = re.search(r'raw: (\d+)', declaration) if const['owner'] else re.search(r'= (\d+)', declaration)
            require(int(literal[1]) == measured[name], name + ': source constant differs')
    print(f"{len(schema['records'])} records, {len(schema['constants'])} constants, {len(values)} vectors agree")
    print('Rust LLVM artifact SHA-256:', hashlib.sha256(artifact).hexdigest())
    print('Omega emitted ABI comparison and firmware execution NOT RUN')


if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--slice', choices=['machine', 'shell'], default='machine')
    args = parser.parse_args()
    if args.slice == 'shell':
        SLICE = 'shell'
        HERE = HERE.parent / 'uefi-shell'
    main()
