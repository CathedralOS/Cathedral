#!/usr/bin/env python3
"""Regenerate this exact-pin machine protocol slice; not a general Rust parser."""
from pathlib import Path
import json
import re
import sys
import uuid

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'tools/ports'))
import inventory
import rust_layout
HERE = Path(__file__).resolve().parent
RAW = ROOT / 'source/contracts/uefi/raw'
UPSTREAM = ROOT / 'reference_code/rust-osdev/uefi-rs'
PIN = 'c0facddf9ba42b74906a37fca2869e6cdbc8da6a'
FILES = ['pci/mod.rs', 'pci/root_bridge.rs', 'usb/mod.rs', 'usb/io.rs',
         'usb/host_controller.rs', 'iommu.rs', 'rng.rs', 'acpi.rs',
         'memory_protection.rs', 'misc.rs', 'driver.rs', 'string.rs']
ROOTS = ['uefi-raw/src/protocol/' + file for file in FILES]


def endbrace(text, start):
    depth = 1
    for index in range(start + 1, len(text)):
        depth += (text[index] == '{') - (text[index] == '}')
        if depth == 0:
            return index
    raise ValueError('unclosed source brace')


def upper(name):
    return re.sub(r'(?<=[a-z0-9])(?=[A-Z])', '_', name).upper()


def guid_literal(spelling):
    value = uuid.UUID(spelling)
    return 'Guid { data1: 0x%08x, data2: 0x%04x, data3: 0x%04x, data4: [' % value.fields[:3] + ', '.join('0x%02x' % byte for byte in value.bytes[8:]) + '] }'


def main():
    manifest = inventory.snapshot(UPSTREAM, PIN, ROOTS, 'https://github.com/rust-osdev/uefi-rs')
    records = []
    constants = []
    owners = {}
    for path in ROOTS:
        original = (UPSTREAM / path).read_text()
        text = re.sub(r'//[^\n]*', '', original)
        module = path.removeprefix('uefi-raw/src/protocol/').removesuffix('.rs').removesuffix('/mod').replace('/', '::')
        prefix = 'uefi_raw::protocol::' + module + '::'
        impl_ranges = []
        for match in re.finditer(r'impl (\w+)\s*{', text):
            end = endbrace(text, match.end() - 1)
            impl_ranges.append((match.start(), end, match[1]))
        for match in re.finditer(r'pub (struct|enum) (\w+)\s*(?::\s*(\w+)\s*(?:=>)?\s*)?{', text):
            kind, name, base = match.groups()
            end = endbrace(text, match.end() - 1)
            body = text[match.end():end]
            record = {'name': name, 'rust': prefix + name, 'path': path, 'base': base,
                      'flags': kind == 'struct' and base is not None}
            if base:
                record['fields'] = [('raw', base)]
                for const in re.finditer(r'(?:const\s+)?([A-Z][A-Z_0-9]*)\s*=\s*(.*?)(?:;|,(?=\s*(?:[A-Z_]|$)))', body, re.S):
                    constants.append({'name': upper(name) + '_' + const[1], 'owner': name, 'source_name': const[1],
                                      'expr': const[2].strip(), 'rust': prefix + name + '::' + const[1], 'path': path, 'guid': base == 'Guid'})
            else:
                hits = list(re.finditer(r'\bpub\s+((?:r#)?\w+)\s*:', body))
                record['fields'] = [(hit[1], ' '.join(body[hit.end():hits[index + 1].start() if index + 1 < len(hits) else len(body)].strip().rstrip(',').split())) for index, hit in enumerate(hits)]
            records.append(record)
            owners[name] = record
        for match in re.finditer(r'pub type (\w+)\s*=\s*(.*?);', text, re.S):
            name, carrier = match.groups()
            records.append({'name': name, 'rust': prefix + name, 'path': path, 'base': 'addr',
                            'fields': [('raw', 'addr')], 'alias': ' '.join(carrier.split()), 'flags': False})
        for match in re.finditer(r'pub const (\w+)\s*:\s*([^=]+?)=\s*(.*?);', text, re.S):
            name, typ, expression = match.groups()
            owner = next((name for start, end, name in impl_ranges if start <= match.start() <= end), None)
            constants.append({'name': upper(owner) + '_' + name if owner else name,
                              'owner': owner, 'source_name': name, 'expr': expression.strip(),
                              'rust': prefix + (owner + '::' if owner else '') + name,
                              'path': path, 'guid': typ.strip() == 'Guid', 'type': typ.strip()})
    if len({r['name'] for r in records}) != len(records):
        raise ValueError('record name collision')
    measurements = []
    for record in records:
        name, rust = record['name'], record['rust']
        measurements += [(name + '.size', f'size_of::<{rust}>()', 'size'),
                         (name + '.alignment', f'align_of::<{rust}>()', 'alignment')]
        if not record['base']:
            measurements += [(name + '.' + field + '.offset', f'offset_of!({rust}, {field})', 'offset') for field, typ in record['fields']]
    for const in constants:
        if not const['guid']:
            suffix = ('.bits()' if owners[const['owner']]['flags'] else '.0') if const['owner'] else ''
            measurements.append((const['name'], const['rust'] + suffix, 'value'))
    probe = ['// SPDX-License-Identifier: MIT OR Apache-2.0', '#![no_std]', 'use core::mem::{size_of, align_of, offset_of};',
             '#[unsafe(no_mangle)]', f'pub static CATHEDRAL_MACHINE_LAYOUT: [u64; {len(measurements)}] = [']
    probe += [f'    {expression} as u64, // {name}' for name, expression, kind in measurements]
    probe += ['];']
    # Actual upstream GUID values also must compile equal to the independent byte spelling.
    for const in constants:
        if const['guid']:
            spelling = re.search(r'guid!\("([\w-]+)"\)', const['expr'])[1]
            const['spelling'] = spelling
            access = const['rust'] + ('.0' if const['owner'] and owners.get(const['owner'], {}).get('base') == 'Guid' else '')
            probe += [f'const _: () = {{ let actual = {access}.to_bytes(); let expected: [u8; 16] = {list(uuid.UUID(spelling).bytes_le)}; let mut i = 0; while i < 16 {{ assert!(actual[i] == expected[i]); i += 1; }} }};']
    (HERE / 'measure.rs').write_text('\n'.join(probe) + '\n')
    measured, artifact = rust_layout.measure(HERE / 'Cargo.toml', HERE / 'measure.rs', 'CATHEDRAL_MACHINE_LAYOUT', 'cathedral_uefi_machine_probe')
    values = {name: {'kind': kind, 'value': measured[name]} for name, expr, kind in measurements}
    output = ['// SPDX-License-Identifier: MIT OR Apache-2.0', '// Modified translation of rust-osdev/uefi-rs at ' + PIN + '.',
              '// See machine.PORT.md. Inert carriers do not grant firmware, MMIO, DMA or I/O authority.',
              '// PORT-BLOCKED[omega:named-calling-policy]: native slots require admitted calling/ownership contracts.',
              'module machine_protocols;', 'use scalars::Guid;', '']
    def mapped(typ):
        if '*' in typ or 'fn(' in typ or 'fn (' in typ:
            return 'addr'
        return {'Handle': 'addr', 'usize': 'u64', 'isize': 'i64', 'Boolean': 'u8', 'ResetSystemFn': 'addr'}.get(typ, typ)
    for record in records:
        output += ['// Upstream: ' + record['rust']]
        if record.get('alias'):
            output += ['// Foreign carrier: ' + record['alias']]
        output += ['pub data ' + record['name'] + ' [copy] {']
        for field, typ in record['fields']:
            output += ['    ' + field.removeprefix('r#') + ': ' + mapped(typ) + ';']
            if mapped(typ) == 'addr':
                output += ['    // Foreign carrier: ' + typ]
        output += ['}', '']
    for const in constants:
        owner = const['owner']
        if const['guid']:
            literal = guid_literal(const['spelling'])
            typ = owner if owner and owners.get(owner, {}).get('base') == 'Guid' else 'Guid'
            if typ != 'Guid':
                literal = typ + ' { raw: ' + literal + ' }'
            values[const['name']] = {'kind': 'bytes', 'value': uuid.UUID(const['spelling']).bytes_le.hex()}
        else:
            typ = owner if owner else const['type']
            literal = str(measured[const['name']])
            if owner:
                literal = owner + ' { raw: ' + literal + ' }'
        output += ['pub const ' + const['name'] + ': ' + typ + ' = ' + literal + ';']
    (RAW / 'machine_protocols.omg').write_text('\n'.join(output) + '\n')
    plans = ['// SPDX-License-Identifier: MIT OR Apache-2.0', '// Required UEFI x64 geometry, not an observed Omega layout.',
             'module machine_layouts;', 'use omega::language::core::layout;', '']
    for record in records:
        name = record['name']
        plans += [f'pub data {name}X64Layout {{}}', f'pub {name}X64Policy: {name}X64Layout satisfies Layout;',
                  f'pub machine {name}X64Layout::plan(schema: Schema) -> Plan satisfies Layout::plan {{', '    let mut entries: [FieldEntry; 64];']
        for index, (field, typ) in enumerate(record['fields']):
            offset = measured.get(name + '.' + field + '.offset', 0)
            plans += [f'    entries[{index}] = FieldEntry {{ key: schema.fields[{index}].key, placement: FieldPlan::At {{ offset: {offset} }} }};']
        plans += ['    Plan { entries: entries, entry_count: ' + str(len(record['fields'])) + ', size_fixed: ' + str(measured[name + '.size']) + ', size_is_dynamic: false, align: ' + str(measured[name + '.alignment']) + ' }', '}', '']
    (RAW / 'machine_layouts.omg').write_text('\n'.join(plans))
    target = 'source/contracts/uefi/raw/machine_protocols.omg'
    for path, entry in manifest['files'].items():
        entry.update(disposition='translated', targets=[{'path': target, 'anchor': 'module machine_protocols;'}]); entry.pop('reason', None)
        for key, row in entry['symbols'].items():
            name = key.split(':', 1)[1]
            anchor = row['anchor']
            if name == '_' or anchor.startswith('pub mod'):
                row.update(disposition='omitted', reason='Rust compile-time size assertion covered by cross-target measured vectors.' if name == '_' else 'Rust module organization flattened into machine_protocols.'); continue
            choices = [r for r in records if r['path'] == path and r['name'] == name]
            if choices:
                destination = 'pub data ' + name + ' '
            elif re.match(r'pub\s+(?:r#)?\w+\s*:', anchor):
                destination = name.removeprefix('r#') + ':'
            else:
                candidates = [c for c in constants if c['path'] == path and c['source_name'] == name]
                if len(candidates) > 1:
                    prefix = '\n'.join((UPSTREAM / path).read_text().splitlines()[:int(key.split(':')[0])])
                    owner = list(re.finditer(r'(?:impl|pub struct|pub enum) (\w+)', prefix))[-1][1]
                    candidates = [c for c in candidates if c['owner'] == owner]
                if len(candidates) != 1:
                    raise ValueError(('unmapped symbol', path, key, anchor))
                destination = 'pub const ' + candidates[0]['name'] + ':'
            row.update(disposition='translated', targets=[{'path': target, 'anchor': destination}]); row.pop('reason', None)
    (RAW / 'machine-inventory.json').write_text(json.dumps(manifest, indent=2) + '\n')
    vector = {'format': 'cathedral-port-vectors-v1', 'target': {'abi': 'UEFI x86-64', 'pointer_bits': 64, 'endian': 'little'},
              'provenance': {'kind': 'upstream', 'description': 'Pinned actual Rust declarations cross-compiled for x86_64-unknown-uefi; not Omega observations.', 'sources': ROOTS, 'revision': PIN}, 'measurements': values}
    (RAW / 'machine.vectors.json').write_text(json.dumps(vector, indent=2) + '\n')
    (HERE / 'schema.json').write_text(json.dumps({'records': records, 'constants': constants}, indent=2) + '\n')
    print(len(records), 'records;', len(constants), 'constants;', len(values), 'measurements')


if __name__ == '__main__':
    main()
