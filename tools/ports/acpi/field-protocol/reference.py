#!/usr/bin/env python3
"""Observe actual public pinned Bank/Index AML operations using inert Vec callbacks.

The existing public Field harness is compiled unchanged. Independent intended
recipe expansion is compared with actual observations; documented differences
are retained as differences, never a production compatibility mode.
"""
import argparse
import hashlib
import importlib.util
import json
import os
import subprocess
import tempfile
from collections import Counter
from pathlib import Path

import vectors

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
ACCESS = HERE.parent / 'field-access'
PIN = '257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
INITIAL = bytes((11 + 37 * i) & 255 for i in range(512))
SOURCE = 0x123456789abcdef0
spec = importlib.util.spec_from_file_location('protocol_aml_encoding', ACCESS / 'aml_encoding.py')
aml = importlib.util.module_from_spec(spec)
spec.loader.exec_module(aml)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def bit_length(value):
    for width in range(1, 5):
        if value < 1 << (6 if width == 1 else 4 + 8 * (width - 1)):
            return bytes([value]) if width == 1 else bytes([((width - 1) << 6) | (value & 15)]) + (value >> 4).to_bytes(width - 1, 'little')
    raise ValueError('field bit length encoding capacity')


def field_list(name, item):
    return (b'\x00' + bit_length(item['offset']) if item['offset'] else b'') + name + bit_length(item['length'])


def table(row):
    body = b'\x5b\x80REG0\x00' + aml.integer(0x1000) + aml.integer(512)
    for name, item in [(b'SEL0', row['selector']), (b'DAT0', row['data'])]:
        body += aml.pkg(0x5b81, b'REG0' + bytes([item['flags']]) + field_list(name, item))
    item = row['field']
    if row['kind'] == 'Bank':
        body += aml.pkg(0x5b87, b'REG0SEL0' + aml.integer(row['bank_value']) + bytes([item['flags']]) + field_list(b'FLD0', item))
    else:
        body += aml.pkg(0x5b86, b'SEL0DAT0' + bytes([item['flags']]) + field_list(b'FLD0', item))
    method = b'\x70' + aml.integer(SOURCE) + b'FLD0\xa4\x00' if row['direction'] == 'Write' else b'\xa4FLD0'
    return body + aml.method('MAIN', method)


def cases():
    rows = []
    def add(name, category, **kwargs):
        row = vectors.case(name, **kwargs); row['category'] = category; rows.append(row)
    for kind in ['Bank', 'Index']:
        for direction in ['Read', 'Write']:
            for code, width in enumerate([1, 1, 2, 4, 8]):
                add(f'agree_{kind.lower()}_{direction.lower()}_{code}', 'agreement', kind=kind, direction=direction,
                    field=vectors.field(width*8, width*8, code),
                    selector=vectors.field(256, 16, 2), data=vectors.field(512, width*8, code))
            add(f'frequency_{kind.lower()}_{direction.lower()}', 'selection_frequency', kind=kind, direction=direction,
                field=vectors.field(0, 32, 1), data=vectors.field(512, 8, 1))
            for update in [0, 1, 2]:
                add(f'partial_{kind.lower()}_{direction.lower()}_{update}',
                    'selection_frequency' if kind == 'Bank' or (direction == 'Write' and update == 0) else 'agreement',
                    kind=kind, direction=direction, field=vectors.field(3, 19, 1 | (update << 5)),
                    data=vectors.field(512, 8, 1))
    # Index selection is already once per chunk for ordinary reads/full writes.
    for row in rows:
        if row['name'].startswith('frequency_index_'):
            row['category'] = 'agreement'
    for code, width in [(2, 2), (3, 4), (4, 8)]:
        for direction in ['Read', 'Write']:
            add(f'index_alignment_{code}_{direction.lower()}', 'index_alignment', direction=direction,
                field=vectors.field(8, width*8, code), data=vectors.field(512, width*8, code))
    for direction in ['Read', 'Write']:
        for length, flags, offset in [(8, 1, 512), (32, 3, 512), (16, 2, 515), (65, 1, 515)]:
            add(f'data_geometry_{direction.lower()}_{length}_{offset}', 'data_geometry', direction=direction,
                field=vectors.field(0, 16, 2), data=vectors.field(offset, length, flags))
        for kind in ['Bank', 'Index']:
            add(f'selector_preserve_{kind.lower()}_{direction.lower()}',
                'agreement' if kind == 'Index' and direction == 'Read' else 'selection_frequency', kind=kind, direction=direction,
                field=vectors.field(1, 17, 1), selector=vectors.field(259, 9, 2), data=vectors.field(512, 8, 1))
            add(f'selector_overflow_{kind.lower()}_{direction.lower()}', 'selector_overflow', kind=kind, direction=direction,
                field=vectors.field(64, 8, 1), selector=vectors.field(256, 3, 1), data=vectors.field(512, 8, 1), bank_value=8)
    # Single datum transfer isolates the pin's oversized Buffer allocation.
    add('buffer_shape_32bit', 'buffer_size', size=4, field=vectors.field(0, 64, 4), data=vectors.field(512, 64, 4))
    add('bank_value_integer_normalization', 'integer_normalization', kind='Bank', size=4,
        bank_value=(1 << 32) + 5, field=vectors.field(0, 8, 1), selector=vectors.field(256, 64, 4))
    return rows


def expected(row):
    """Expand the mathematical recipe into numeric callbacks on one inert Vec."""
    recipe = row['expected']
    if 'protocol' in recipe or 'error' in recipe:
        return None
    memory = bytearray(INITIAL); log = []
    def read(byte, width):
        value = int.from_bytes(memory[byte:byte+width], 'little')
        log.append(f'read,{byte},{width},{value}'); return value
    def write(byte, width, value):
        value &= (1 << (width*8)) - 1
        log.append(f'write,{byte},{width},{value}')
        memory[byte:byte+width] = value.to_bytes(width, 'little')
    def read_field(plan):
        value = 0
        for chunk in plan['chunks']:
            value |= ((read(chunk['offset'], chunk['width']) >> chunk['native_bit']) & ((1 << chunk['bit_count']) - 1)) << chunk['field_bit']
        return value
    def merge(chunk, rule, value, previous=0):
        bits = chunk['width'] * 8
        base = previous if rule == 0 else (1 << bits) - 1 if rule == 1 else 0
        return ((base & ~chunk['mask']) | (((value >> chunk['field_bit']) << chunk['native_bit']) & chunk['mask'])) & ((1 << bits) - 1)
    def write_field(plan, item, value):
        rule = (item['flags'] >> 5) & 3
        for chunk in plan['chunks']:
            previous = read(chunk['offset'], chunk['width']) if chunk['partial'] and rule == 0 else 0
            write(chunk['offset'], chunk['width'], merge(chunk, rule, value, previous))
    datum = {}; result = 0
    for action in recipe['actions']:
        if action['kind'] == 'Select':
            write_field(recipe['selector'], row['selector'], action['value']); continue
        index = action['index']; chunk = recipe['outer']['chunks'][index]
        mask = (1 << (chunk['width'] * 8)) - 1
        if action['kind'] == 'Read':
            raw = read_field(recipe['data']) if row['kind'] == 'Index' else read(chunk['offset'], chunk['width'])
            datum[index] = raw & mask
            if row['direction'] == 'Read':
                result |= ((datum[index] >> chunk['native_bit']) & ((1 << chunk['bit_count']) - 1)) << chunk['field_bit']
        else:
            word = merge(chunk, (row['field']['flags'] >> 5) & 3, SOURCE, datum.get(index, 0))
            if row['kind'] == 'Index':
                write_field(recipe['data'], row['data'], word)
            else:
                write(chunk['offset'], chunk['width'], word)
    result_text = 'integer:0' if row['direction'] == 'Write' else 'integer:' + str(result)
    if row['direction'] == 'Read' and recipe['outer']['shape'] == 'Buffer':
        count = recipe['outer']['bytes']; result_text = f'buffer:{count}:' + result.to_bytes(count, 'little').hex()
    return dict(load='ok', result=result_text, accesses=';'.join(log), memory=memory.hex(),
                forbidden_calls='0', created_mutexes='1')


def compare(row, observed):
    strict = expected(row); category = row['category']
    assert observed['load'] == 'ok', (row['name'], observed)
    assert observed['forbidden_calls'] == '0' and observed['created_mutexes'] == '1'
    assert 'panic' not in observed and observed.get('result', '').startswith(('integer:', 'buffer:'))
    differing = [] if strict is None else [key for key in strict if strict[key] != observed[key]]
    if category == 'agreement':
        assert strict == observed, (row['name'], strict, observed)
    elif category == 'selector_overflow':
        assert strict is None and row['expected'] == dict(protocol='SelectorOverflow')
        assert observed['accesses'], 'pin silently truncates selector instead of rejecting'
    elif category == 'integer_normalization':
        # Actual public evaluation may normalize the literal before the private write.
        assert not differing or set(differing) <= {'accesses', 'memory'}
    else:
        assert differing, (row['name'], 'expected documented difference')
        if category == 'selection_frequency':
            assert differing == ['accesses'], (row['name'], differing)
        if category == 'buffer_size':
            assert differing == ['result']
    return dict(name=row['name'], category=category, differing=differing,
                expected=strict, observed=observed)


def source_inputs():
    return [HERE/'reference.py', HERE/'vectors.py', ACCESS/'reference.rs', ACCESS/'reference.Cargo.lock', ACCESS/'aml_encoding.py']


def main():
    parser = argparse.ArgumentParser(); parser.add_argument('--repository', type=Path, default=ROOT)
    parser.add_argument('--write', action='store_true')
    parser.add_argument('--fetch', action='store_true', help='Fetch the exact retained Cargo.lock dependencies before the offline build')
    parser.add_argument('--cargo', type=Path, default=Path('cargo'), help='Cargo executable; caller supplies its toolchain on PATH')
    parser.add_argument('--target-dir', type=Path, default=Path(tempfile.gettempdir())/'cathedral-field-protocol-public')
    args = parser.parse_args()
    upstream = args.repository / 'reference_code/rust-osdev/acpi'
    assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=upstream, text=True).strip() == PIN
    paths = sorted((upstream/'src').rglob('*.rs')) + [upstream/p for p in ['Cargo.toml', 'LICENCE-MIT', 'LICENCE-APACHE']]
    provenance = {str(path.relative_to(upstream)): sha(path) for path in paths}
    for path, digest in provenance.items():
        assert hashlib.sha256(subprocess.check_output(['git', 'show', PIN+':'+path], cwd=upstream)).hexdigest() == digest
    before = {str(p.relative_to(ROOT)): sha(p) for p in source_inputs()}
    target = args.target_dir.resolve()
    with tempfile.TemporaryDirectory(prefix='field-protocol-reference-') as directory:
        work = Path(directory)
        (work/'Cargo.toml').write_text(f'[package]\nname="cathedral-field-access-reference"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nacpi={{path="{upstream}"}}\n[[bin]]\nname="reference"\npath="{ACCESS}/reference.rs"\n')
        (work/'Cargo.lock').write_bytes((ACCESS/'reference.Cargo.lock').read_bytes())
        if args.fetch:
            subprocess.run([str(args.cargo), 'fetch', '--locked', '--manifest-path', str(work/'Cargo.toml')], check=True)
        subprocess.run([str(args.cargo), 'build', '--release', '--offline', '--locked', '--manifest-path', str(work/'Cargo.toml'), '--target-dir', str(target)], check=True)
        binary = target/'release'/('reference.exe' if os.name == 'nt' else 'reference'); observations = []; comparisons = []
        for row in cases():
            data = table(row); path = work/'table.aml'; path.write_bytes(data)
            run = subprocess.run([str(binary), str(path), '1' if row['size'] == 4 else '2'], capture_output=True, text=True, timeout=5, check=True)
            observed = dict(line.split('\t', 1) for line in run.stdout.splitlines())
            comparisons.append(compare(row, observed))
            observations.append(dict(**row, aml_hex=data.hex(), observed=observed))
        record = dict(stage='actual public Interpreter load_table/evaluate; unchanged shared Rust harness and inert Vec native callbacks',
                      pin=PIN, upstream_sha256=provenance, source_sha256=before,
                      binary_sha256=sha(binary), rows=observations)
    assert before == {str(p.relative_to(ROOT)): sha(p) for p in source_inputs()}, 'sources changed during observation'
    comparison = dict(stage='independent recipe arithmetic expanded through normal register geometry; differences retained without compatibility mode',
                      public_count=len(comparisons), counts=dict(sorted(Counter(r['category'] for r in comparisons).items())), rows=comparisons)
    for filename, value in [('reference-verification.json', record), ('comparison.json', comparison)]:
        path = HERE/filename
        if args.write:
            path.write_text(json.dumps(value, indent=2, sort_keys=True)+'\n')
        else:
            assert json.loads(path.read_text()) == value, filename
    print('PASS', len(observations), 'public Bank/Index observations', comparison['counts'])


if __name__ == '__main__':
    main()
