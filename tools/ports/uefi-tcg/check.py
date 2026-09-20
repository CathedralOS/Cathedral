#!/usr/bin/env python3
"""Verify TCG source coverage and actual pinned Rust UEFI-target layout/value data."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
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


def require(condition, message):
    if not condition:
        raise ValueError(message)


def main():
    print(json.dumps(inventory.check(inventory.read_json(RAW / 'tcg-inventory.json'), UPSTREAM, require_transcribed=True)))
    expected = vectors.validate(inventory.read_json(RAW / 'tcg.vectors.json'))['measurements']
    measured, artifact = rust_layout.measure(HERE / 'Cargo.toml', HERE / 'measure.rs', 'CATHEDRAL_TCG_LAYOUT', 'cathedral_uefi_tcg_layout_probe')
    numeric_keys = {name for name, row in expected.items() if row['kind'] != 'bytes'}
    require(set(measured) == numeric_keys, 'upstream measurement coverage differs')
    for name, value in measured.items():
        require(value == expected[name]['value'], f'{name}: measured {value}, expected {expected[name]["value"]}')
    omega = (RAW / 'tcg.omg').read_text()
    constants = {name: int(value.replace('_', ''), 16) for name, value in
                 re.findall(r'pub const (\w+): u\d+ = (0x[0-9a-fA-F_]+);', omega)}
    value_keys = {name for name, row in expected.items() if row['kind'] == 'value'}
    require(set(constants) == value_keys, 'Omega numeric constant coverage differs')
    for name, value in constants.items():
        require(value == measured[name], f'{name}: Omega source literal differs')
    plans = (RAW / 'tcg_layouts.omg').read_text()
    for name, body in re.findall(r'pub data (\w+) \[copy\] \{(.*?)\}', omega, re.S):
        fields = re.findall(r'(\w+):\s*\w+', re.sub(r'//[^\n]*', '', body))
        plan = re.search(r'pub machine '+name+r'X64Layout::plan.*?\n\{(.*?)\n\}', plans, re.S)
        require(plan is not None, f'{name}: missing named layout plan')
        offsets = [int(value) for value in re.findall(r'FieldPlan::At \{ offset: (\d+) \}', plan[1])]
        require(offsets == [expected.get(f'{name}.{field}.offset', {'value': 0})['value'] for field in fields], f'{name}: layout field offsets differ')
        for field, key in [('size_fixed', 'size'), ('align', 'alignment')]:
            require(int(re.search(field+r': (\d+)', plan[1])[1]) == measured[name+'.'+key], f'{name}: plan {field} differs')
        require(int(re.search(r'entry_count: (\d+)', plan[1])[1]) == len(fields), f'{name}: plan coverage differs')
    total_fields = 0
    for version in (1, 2):
        rust = (UPSTREAM / f'uefi-raw/src/protocol/tcg/v{version}.rs').read_text()
        for name, body in re.findall(r'pub struct (\w+) \{(.*?)\n\}', rust, re.S):
            field_names = re.findall(r'^\s*pub (\w+):', body, re.M)
            counterpart = re.search(r'pub data '+name+r' \[copy\] \{(.*?)\n\}', omega, re.S)
            require(counterpart is not None, f'{name}: missing Omega data')
            omega_fields = re.findall(r'^\s*(\w+):', counterpart[1], re.M)
            require(field_names == omega_fields, f'{name}: field order or coverage differs')
            total_fields += len(field_names)
        guid = uuid.UUID(re.search(r'pub const GUID: Guid = guid!\("([\w-]+)"\)', rust)[1])
        key = 'TCG_PROTOCOL_GUID' if version == 1 else 'TCG2_PROTOCOL_GUID'
        require(expected[key]['value'] == guid.bytes_le.hex(), f'{key}: GUID byte order differs')
        body = re.search('pub const '+key+r': Guid = Guid \{(.*?)\};', omega, re.S)[1]
        numbers = [int(value, 16) for value in re.findall(r'0x([0-9a-f]+)', body)]
        require(numbers == list(guid.fields[:3])+list(guid.bytes[8:]), f'{key}: Omega GUID fields differ')
    print(f'{len(measured)} upstream UEFI x64 numeric measurements, 2 GUIDs, {total_fields} source fields agree')
    print('Rust LLVM artifact SHA-256:', hashlib.sha256(artifact).hexdigest())
    print('Omega ABI comparison NOT RUN; source spelling checks are not compiled layout checks')


if __name__ == '__main__':
    try:
        main()
    except (ValueError, KeyError, TypeError, OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(f'error: {error}')
