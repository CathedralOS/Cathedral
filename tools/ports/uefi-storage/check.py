#!/usr/bin/env python3
"""Audit the complete pinned raw storage slice and its cross-target ABI vectors."""
import hashlib
import json
from pathlib import Path
import re
import sys
import uuid

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'tools/ports'))
import inventory
import vectors
import rust_layout

HERE = Path(__file__).resolve().parent
RAW = ROOT / 'source/contracts/uefi/raw'
UPSTREAM = ROOT / 'reference_code/rust-osdev/uefi-rs'


def require(condition, detail):
    if not condition:
        raise ValueError(detail)


def source_fields(source, name):
    body = source.split('pub struct ' + name + ' {', 1)[1].split('\n}', 1)[0]
    body = re.sub(r'//[^\n]*', '', body)
    matches = list(re.finditer(r'pub ((?:r#)?\w+):\s*', body))
    return [(match[1], ' '.join(body[match.end():matches[index + 1].start() if index + 1 < len(matches) else len(body)].strip().rstrip(',').split()))
            for index, match in enumerate(matches)]


def main():
    manifest = inventory.read_json(RAW / 'storage-inventory.json')
    print(json.dumps(inventory.check(manifest, UPSTREAM, require_transcribed=True)))
    expected = vectors.validate(inventory.read_json(RAW / 'storage.vectors.json'))['measurements']
    shapes = inventory.read_json(HERE / 'source-shapes.json')
    require(shapes['pin'] == manifest['upstream']['revision'], 'shape manifest pin differs')
    omega = (RAW / 'storage.omg').read_text()
    layouts = (RAW / 'storage_layouts.omg').read_text()
    field_count = 0
    for name, record in shapes['structs'].items():
        fields = record['fields']
        require(source_fields((UPSTREAM / record['path']).read_text(), name) == [(f[0], f[2]) for f in fields], f'{name}: upstream field/signature metadata differs')
        body = omega.split('pub data ' + name + ' {', 1)[1].split('\n}', 1)[0]
        body_without_comments = re.sub(r'//[^\n]*', '', body)
        actual = re.findall(r'(\w+):\s*([^;]*(?:;\s*\d+\])?)\s*;', body_without_comments)
        require(actual == [(f[1], f[3]) for f in fields], f'{name}: Omega fields/types/order differ')
        for _, _, rust_type, omega_type in fields:
            if omega_type == 'addr':
                require('// Upstream: ' + rust_type in body, f'{name}: original pointer/callback signature missing')
        plan = layouts.split('pub machine ' + name + 'X64Layout::plan', 1)[1].split('\n}', 1)[0]
        offsets = [int(n) for n in re.findall(r'FieldPlan::At \{ offset: (\d+) \}', plan)]
        require(offsets == [expected[name + '.' + field[1] + '.offset']['value'] for field in fields], f'{name}: requested offsets differ')
        for source_key, vector_key in [('size_fixed', 'size'), ('align', 'alignment')]:
            require(int(re.search(source_key + r': (\d+)', plan)[1]) == expected[name + '.' + vector_key]['value'], f'{name}: requested {vector_key} differs')
        field_count += len(fields)
    for name, (ty, _) in shapes['wrappers'].items():
        require(f'pub data {name} [copy] {{ raw: {ty}; }}' in omega, f'{name}: scalar representation differs')
    guid_count = 0
    for constant in shapes['constants']:
        name, kind, value = constant['name'], constant['kind'], constant['value']
        line = re.search(r'pub const ' + name + r':[^\n]+', omega)[0]
        if kind == 'guid':
            guid = uuid.UUID(value)
            require('guid!("' + value + '")' in (UPSTREAM / constant['path']).read_text(), f'{name}: GUID absent in pinned source')
            fields = [int(n, 16) for n in re.findall(r'0x([0-9a-f]+)', line)]
            require(fields == list(guid.fields[:3]) + list(guid.bytes[8:]), f'{name}: Omega GUID fields differ')
            require(expected[name + '.bytes']['value'] == guid.bytes_le.hex(), f'{name}: GUID bytes differ')
            guid_count += 1
        elif kind == 'bytes':
            require('[' + ', '.join(str(n) for n in value) + ']' in line, f'{name}: byte literal differs')
            require(expected[name + '.bytes']['value'] == bytes(value).hex(), f'{name}: byte vector differs')
        else:
            literal = re.search(r'=\s*(?:\w+\s*\{\s*raw:\s*)?(\d+)', line)
            require(literal is not None and int(literal[1]) == value == expected[name]['value'], f'{name}: constant differs')
    measured, artifact = rust_layout.measure(HERE / 'Cargo.toml', HERE / 'src/lib.rs',
                                           'CATHEDRAL_STORAGE_LAYOUT', 'cathedral_uefi_storage_layout_probe')
    integers = {name: row['value'] for name, row in expected.items() if row['kind'] != 'bytes'}
    require(set(measured) == set(integers), 'upstream measurement coverage differs from every integer vector')
    for name, actual in measured.items():
        require(actual == integers[name], f'{name}: measured upstream {actual} != expected {integers[name]}')
    print(f'{len(shapes["structs"])} records / {field_count} fields / {len(shapes["wrappers"])} scalar carriers / {len(shapes["constants"])} constants checked')
    print(f'{len(measured)} UEFI x64 Rust layout/value measurements agree; {guid_count} GUIDs checked')
    print('Rust LLVM artifact SHA-256:', hashlib.sha256(artifact).hexdigest())
    print('Omega ABI comparison, firmware/device execution, variable-tail access NOT RUN')


if __name__ == '__main__':
    try:
        main()
    except (ValueError, KeyError, IndexError, OSError) as error:
        raise SystemExit(f'error: {error}')
