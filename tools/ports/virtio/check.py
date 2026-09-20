#!/usr/bin/env python3
"""Audit the pin, raw schema coverage and actual cross-target Rust facts."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
import inventory
import rust_layout
import vectors

def clean(source):
    source = re.sub(r'//[^\n]*', '', source)
    source = re.sub(r'/\*.*?\*/', '', source, flags=re.S)
    while '#[' in source:
        start = source.index('#[')
        depth = 0
        for end in range(start+1, len(source)):
            if source[end] == '[': depth += 1
            elif source[end] == ']':
                depth -= 1
                if depth == 0:
                    source = source[:start] + source[end+1:]
                    break
    return source

def main():
    package = ROOT/'source/libraries/virtio'
    checkout = ROOT/'reference_code/rust-osdev/virtio-spec-rs'
    print(json.dumps(inventory.check(inventory.read_json(package/'inventory.json'), checkout, ROOT), indent=2), flush=True)
    shapes = inventory.read_json(HERE/'raw-shapes.json')
    count = 0
    for module, records in shapes.items():
        upstream = clean((checkout/'src'/f'{module}.rs').read_text())
        omega = (package/f'raw_{module}.omg').read_text()
        actual = {}
        for match in re.finditer(r'pub struct (\w+)\s*\{([^}]+)\}', upstream):
            name, body = match.groups()
            if name == 'DiscardWriteZeroesFlags':
                if 'pub data DiscardWriteZeroesFlags [copy] { raw: u32; }' not in omega:
                    raise ValueError('discard/write-zeroes bitfield carrier missing')
                continue
            actual[name] = [list(pair) for pair in re.findall(r'(?:pub\s+)?(\w+)\s*:\s*([^,]+),', body)]
        if actual != records:
            raise ValueError(f'{module}: source raw schema inventory mismatch')
        for name, fields in records.items():
            match = re.search(r'pub data '+name+r' \[copy\] \{([^}]+)\}', omega)
            if not match: raise ValueError(f'{module}.{name}: missing Omega schema')
            translated = [list(pair) for pair in re.findall(r'^\s*(\w+)\s*:\s*(.+);$', match[1], re.M)]
            wanted = [[n, re.sub(r'\ble(16|32|64)\b', r'u\1', t.strip())] for n,t in fields]
            if translated != wanted: raise ValueError(f'{module}.{name}: fields/types/order changed')
            count += 1
        # Enum discriminants and their integer carrier are supplementary to the lexical inventory.
        for match in re.finditer(r'#\[repr\((u\d+)\)\](?:(?!pub enum).)*pub enum (\w+)\s*\{([^}]+)\}', (checkout/'src'/f'{module}.rs').read_text(), re.S):
            carrier, name, body = match.groups()
            if module == 'net' and name in ['Rx','Mac','Vlan','Announce','Mq','GuestOffloads']: name = 'Ctrl'+name
            if f'pub data {name} [copy] {{ raw: {carrier}; }}' not in omega:
                raise ValueError(f'{module}.{name}: enum carrier differs from {carrier}')
            for variant, value in re.findall(r'^\s*(\w+)\s*=\s*(0x[0-9a-f]+|[0-9]+)',body,re.M):
                expected = f'pub const {name}_{variant}: {name} = {name} {{ raw: {int(value,0)} }};'
                if expected not in omega: raise ValueError(f'{module}.{name}.{variant}: discriminant missing')
    print(f'{count} device records: every field, original width/order, and enum carrier/discriminant checked.', flush=True)
    config = clean((checkout/'src/pci.rs').read_text())
    config_body = re.search(r'pub struct CommonCfg\s*\{([^}]+)\}', config)[1]
    expected_fields = [(n, re.sub(r'\ble(16|32|64)\b', r'u\1', t.strip())) for n,t in re.findall(r'(\w+)\s*:\s*([^,]+),', config_body)]
    translated_fields = re.findall(r'^\s*(\w+)\s*:\s*(.+);$', (package/'configuration.omg').read_text(), re.M)
    if translated_fields != expected_fields: raise ValueError('CommonCfg private field order/width mismatch')
    geometry = inventory.read_json(HERE/'geometry.json')['records']
    policies = (package/'layouts.omg').read_text()
    for key, row in geometry.items():
        name = ''.join(word[0].upper()+word[1:] for word in key.split('.'))+'Layout'
        body = policies.split('pub machine '+name+'::plan',1)[1].split('\npub data ',1)[0]
        offsets = [int(n) for n in re.findall(r'FieldPlan::At \{ offset: (\d+) \}',body)]
        if offsets != [field['offset'] for field in row['fields']]: raise ValueError(f'{key}: layout policy offsets mismatch')
        if f"size_fixed: {row['size']}" not in body or f"align: {row['alignment']}" not in body: raise ValueError(f'{key}: layout policy geometry mismatch')
    print('CommonCfg private fields and all 24 requested policy plans audited.', flush=True)
    expected = vectors.validate(inventory.read_json(package/'core.vectors.json'))
    measured, artifact = rust_layout.measure(HERE/'Cargo.toml', HERE/'src/lib.rs', 'CATHEDRAL_VIRTIO_LAYOUT', 'cathedral_virtio_layout_probe')
    metadata = inventory.read_json(HERE/'geometry.json')
    if set(measured) != set(metadata['rust_measured_keys']): raise ValueError('measured key coverage changed')
    for key, value in measured.items():
        if expected['measurements'][key]['value'] != value: raise ValueError(f'{key}: actual Rust target value differs')
    print(f'{len(expected["measurements"])} expected vectors valid; {len(measured)} actual x86_64-unknown-uefi Rust measurements agree.', flush=True)
    print('LLVM artifact SHA-256:', hashlib.sha256(artifact).hexdigest(), flush=True)
    subprocess.run(['cargo','run','--locked','--manifest-path',str(HERE/'Cargo.toml')],cwd=ROOT,check=True)
    print('Private-field offsets remain source-derived; Omega ABI comparison and device/DMA execution NOT RUN.')
if __name__ == '__main__': main()
