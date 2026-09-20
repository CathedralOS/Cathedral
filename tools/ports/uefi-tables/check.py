#!/usr/bin/env python3
"""Check pinned UEFI table source transcription and cross-target Rust geometry."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import uuid

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'tools/ports'))
import inventory
import vectors

UPSTREAM = ROOT / 'reference_code/rust-osdev/uefi-rs'
RAW = ROOT / 'source/contracts/uefi/raw'
HERE = Path(__file__).resolve().parent
PIN = 'c0facddf9ba42b74906a37fca2869e6cdbc8da6a'


def require(condition, message):
    if not condition:
        raise ValueError(message)


def fields(source, name, rust):
    start = ('pub struct ' if rust else 'pub data ') + name
    body = source.split(start, 1)[1].split('{', 1)[1].split('\n}', 1)[0]
    body = re.sub(r'//[^\n]*', '', body)
    pattern = r'\bpub (\w+):\s*' if rust else r'\b(\w+):\s*'
    return re.findall(pattern, body)


def main():
    manifest = inventory.read_json(RAW / 'tables-inventory.json')
    print(json.dumps(inventory.check(manifest, UPSTREAM, require_transcribed=True)))
    expected = vectors.validate(inventory.read_json(RAW / 'tables.vectors.json'))
    rows = expected['measurements']
    omega = (RAW / 'tables.omg').read_text()
    layouts = (RAW / 'tables_layouts.omg').read_text()
    count = 0
    paths = {'Header':'header', 'SystemTable':'system', 'ConfigurationTable':'configuration',
             'BootServices':'boot', 'RuntimeServices':'runtime',
             'OpenProtocolInformationEntry':'boot', 'TimeCapabilities':'runtime'}
    for name, module in paths.items():
        rust = (UPSTREAM / f'uefi-raw/src/table/{module}.rs').read_text()
        names = fields(rust, name, True)
        require(fields(omega, name, False) == names, f'{name}: field coverage/order differs')
        body = layouts.split(f'pub machine {name}X64Layout::plan', 1)[1].split('\n}', 1)[0]
        offsets = [int(x) for x in re.findall(r'FieldPlan::At \{ offset: (\d+) \}', body)]
        require(offsets == [rows[f'{name}.{field}.offset']['value'] for field in names], f'{name}: authored plan differs from vectors')
        require(int(re.search(r'size_fixed: (\d+)', body)[1]) == rows[f'{name}.size']['value'], f'{name}: authored size differs')
        require(int(re.search(r'align: (\d+)', body)[1]) == rows[f'{name}.alignment']['value'], f'{name}: authored alignment differs')
        if name in ('BootServices', 'RuntimeServices'):
            for index, field in enumerate(names[1:]):
                key = f'{name.upper()}_{field.upper()}_SLOT'
                require(rows[key]['value'] == index, f'{key}: slot ordinal differs')
                require(f'pub const {key}: u64 = {index};' in omega, f'{key}: constant differs')
                require(rows[f'{name}.{field}.offset']['value'] == 24 + 8 * index, f'{key}: offset differs')
        count += len(names)

    # Check all copied GUIDs against pinned source, their numeric Omega fields,
    # and UEFI mixed-endian byte vectors; UUID byte encoding is independent.
    cfg = (UPSTREAM / 'uefi/src/table/cfg.rs').read_text()
    guids = re.findall(r'pub const (\w+): Guid = guid!\("([\w-]+)"\)', cfg)
    runtime = (UPSTREAM / 'uefi-raw/src/table/runtime.rs').read_text()
    guids += [('VARIABLE_VENDOR_' + n, g) for n, g in re.findall(r'(\w+) = guid!\("([\w-]+)"\)', runtime)]
    for name, spelling in guids:
        guid = uuid.UUID(spelling)
        require(rows[name + '.bytes']['value'] == guid.bytes_le.hex(), f'{name}: GUID bytes differ')
        line = re.search(r'pub const ' + name + r':[^\n]+', omega)[0]
        numeric = [int(n, 16) for n in re.findall(r'0x([0-9a-f]+)', line)]
        require(numeric == list(guid.fields[:3]) + list(guid.bytes[8:]), f'{name}: GUID fields differ')

    with tempfile.TemporaryDirectory(prefix='cathedral-uefi-tables-') as temporary:
        subprocess.run(['cargo', 'rustc', '--locked', '--manifest-path', str(HERE / 'Cargo.toml'),
                        '--target', 'x86_64-unknown-uefi', '--target-dir', temporary,
                        '--lib', '--', '--emit=llvm-ir'], check=True)
        artifacts = list(Path(temporary).glob('x86_64-unknown-uefi/debug/deps/cathedral_uefi_table_layout_probe-*.ll'))
        require(len(artifacts) == 1, 'expected one cross-target LLVM artifact')
        artifact = artifacts[0].read_text()
        require('target triple = "x86_64-unknown-windows-msvc"' in artifact, 'unexpected UEFI LLVM target triple')
        encoded = re.search(r'@CATHEDRAL_TABLE_LAYOUT = .*?c"([^"\n]*)"', artifact)[1]
        data = bytearray()
        while encoded:
            if encoded.startswith('\\'):
                data.append(int(encoded[1:3], 16)); encoded = encoded[3:]
            else:
                data.append(ord(encoded[0])); encoded = encoded[1:]
        keys = re.findall(r' as u64, // (\S+)', (HERE / 'src/lib.rs').read_text())
        require(len(data) == 8 * len(keys), 'Rust probe value count mismatch')
        for index, key in enumerate(keys):
            actual = int.from_bytes(data[8 * index:8 * index + 8], 'little')
            require(rows[key]['value'] == actual, f'{key}: Rust {actual} != expected {rows[key]["value"]}')
        encoded_scalars = re.search(r'@CATHEDRAL_SCALAR_LAYOUT = .*?c"([^"\n]*)"', artifact)[1]
        scalar_bytes = bytearray()
        while encoded_scalars:
            if encoded_scalars.startswith('\\'):
                scalar_bytes.append(int(encoded_scalars[1:3], 16)); encoded_scalars = encoded_scalars[3:]
            else:
                scalar_bytes.append(ord(encoded_scalars[0])); encoded_scalars = encoded_scalars[1:]
        scalar_values = [int.from_bytes(scalar_bytes[n:n + 8], 'little') for n in range(0, len(scalar_bytes), 8)]
        print('Companion upstream UEFI x64 size/alignment:', dict(zip(['Guid', 'CapsuleHeader', 'Time', 'MemoryDescriptor'], zip(scalar_values[::2], scalar_values[1::2]))))
        print(f'{count} raw fields checked; {len(guids)} GUIDs checked; {len(keys)} UEFI x64 Rust layout measurements agree')
        print('Rust LLVM artifact SHA-256:', hashlib.sha256(artifacts[0].read_bytes()).hexdigest())
    print('Omega ABI comparison NOT RUN; Rust agreement is not an Omega layout observation')


if __name__ == '__main__':
    try:
        main()
    except (ValueError, KeyError, IndexError, OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(f'error: {error}')
