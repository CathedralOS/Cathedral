#!/usr/bin/env python3
"""Generate the reviewed pinned network transcription and independent Rust probe.

Restricted lexical extraction for this exact pin, not a general Rust translator.
"""
from pathlib import Path
import json
import re
import sys
import uuid

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(HERE.parent))
import inventory
PIN = 'c0facddf9ba42b74906a37fca2869e6cdbc8da6a'
UPSTREAM = ROOT / 'reference_code/rust-osdev/uefi-rs'
RAW = ROOT / 'source/contracts/uefi/raw'
SOURCES = ['uefi-raw/src/net.rs', 'uefi-raw/src/protocol/network']
# Validate checkout identity and bytes before reading source.
inventory.sources(UPSTREAM, PIN, SOURCES)


def endbrace(text, opening):
    depth = 1
    for index in range(opening + 1, len(text)):
        depth += (text[index] == '{') - (text[index] == '}')
        if depth == 0:
            return index
    raise ValueError('unclosed source brace')


def upper(name):
    return re.sub(r'(?<=[a-z0-9])(?=[A-Z])', '_', name).upper()


def fields(body):
    matches = list(re.finditer(r'\bpub\s+(\w+)\s*:', body))
    return [(m[1], ' '.join(body[m.end():matches[i+1].start() if i+1 < len(matches) else len(body)].strip().rstrip(',').split()))
            for i, m in enumerate(matches)]


records, constants = [], []
paths = [UPSTREAM / SOURCES[0]] + sorted((UPSTREAM / SOURCES[1]).glob('*.rs'))
for path in paths:
    relative = path.relative_to(UPSTREAM).as_posix()
    prefix = 'uefi_raw::' if path.name == 'net.rs' else 'uefi_raw::protocol::network::' + path.stem + '::'
    source = re.sub(r'//[^\n]*', '', path.read_text())
    # Keep repr metadata until each declaration has captured it.
    for match in re.finditer(r'pub (struct|enum|union) (\w+)\s*(?::\s*(\w+)\s*(?:=>)?\s*)?([({])', source):
        kind, name, base, opening = match.groups()
        if opening == '(':
            end = source.index(')', match.end())
            base = source[match.end():end].replace('pub ', '').strip()
            body = ''
        else:
            end = endbrace(source, match.end() - 1)
            body = source[match.end():end]
        preceding = source[:match.start()].rsplit('}', 1)[-1]
        packed = 'repr(C, packed)' in preceding
        row = {'name': name, 'rust': prefix + name, 'path': relative,
               'kind': kind, 'packed': packed, 'base': base,
               'fields': [('raw', base)] if base else fields(body)}
        records.append(row)
        if base:
            for item in re.finditer(r'(?:const\s+)?([A-Z][A-Z_0-9]*)\s*=\s*([^,;]+)[,;]', body):
                expr = item[2].strip()
                if not re.fullmatch(r'(?:0x[0-9a-fA-F_]+|[0-9_]+)', expr):
                    raise ValueError((name, item[1], expr))
                value = int(expr.replace('_', ''), 16 if expr.startswith('0x') else 10)
                constants.append({'owner': name, 'name': upper(name)+'_'+item[1], 'rust': prefix+name+'::'+item[1],
                                  'value': value, 'kind': 'value', 'accessor': '.bits()' if kind == 'struct' else '.0'})
    for match in re.finditer(r'impl (\w+)\s*{', source):
        owner = match[1]
        body = source[match.end():endbrace(source, match.end() - 1)]
        for item in re.finditer(r'pub const (\w+)\s*:\s*([^=]+)=\s*(.*?);', body, re.S):
            name, typ, expr = item.groups()
            guid = re.search(r'guid!\("([0-9a-fA-F-]+)"\)', expr)
            if guid:
                constants.append({'owner': owner, 'name': upper(owner)+'_'+name, 'rust': prefix+owner+'::'+name,
                                  'kind': 'bytes', 'value': uuid.UUID(guid[1]).bytes_le.hex(), 'guid': guid[1]})
            elif name == 'DHCP_MAGIK':
                constants.append({'owner': owner, 'name': upper(owner)+'_'+name, 'rust': prefix+owner+'::'+name,
                                  'kind': 'value', 'value': int(expr.strip(), 16), 'primitive': 'u32', 'accessor': ''})
            elif name == 'ZERO':
                pass  # An explicit all-byte-zero record constant is emitted below.
            else:
                raise ValueError((owner, name, expr))

# Primitive alias; unlike a firmware pointer, a UDP port is an inert 16-bit word.
records.append({'name':'PxeBaseCodeUdpPort','rust':'uefi_raw::protocol::network::pxe::PxeBaseCodeUdpPort',
                'path':'uefi-raw/src/protocol/network/pxe.rs','kind':'alias','packed':False,'base':'u16','fields':[('raw','u16')]})
# Union storage is explicit; no invented Omega overlay syntax or typed pointer.
union_storage = {'IpAddress':'[u8; 16]', 'PxeBaseCodePacket':'[u8; 1472]',
                 'PxeBaseCodeIcmpErrorUnion':'u32', 'HttpAccessPoint':'addr',
                 'HttpRequestOrResponse':'addr', 'Tcp4Packet':'addr'}

def translated_type(typ):
    if '*' in typ or 'fn(' in typ or 'fn (' in typ:
        return 'addr'
    if typ == 'usize':
        return 'u64'
    if typ.startswith('['):
        inner, count = typ[1:-1].split(';')
        return '['+translated_type(inner.strip())+'; '+count.strip()+']'
    return typ

lines = ['// SPDX-License-Identifier: MIT OR Apache-2.0',
         '// Modified Omega translation of rust-osdev/uefi-rs at '+PIN+'.',
         '// See network.PORT.md; generator tools/ports/uefi-network/generate.py.',
         '// Inert raw x86-64 shapes. Addresses and function slots grant no authority.',
         '// Fixed foreign geometry is supplied separately, never inferred from native data.',
         'module network;']
for name in ['Boolean','Char8','Char16','Event','Guid','Handle']:
    lines.append('use scalars::'+name+';')
lines += ['use status::Status;', '']
probe = ['// SPDX-License-Identifier: MIT OR Apache-2.0',
         '// Independent measurements of pinned Rust; never Cathedral target code.',
         'use core::mem::{size_of,align_of,offset_of};', 'fn main() {']
measurements = {}
for record in records:
    name, rust = record['name'], record['rust']
    lines += ['// Pinned source: '+rust, 'pub data '+name+' [copy] {']
    mapped = []
    for field, typ in record['fields']:
        tail = bool(re.search(r';\s*0\s*\]', typ))
        if record['kind'] == 'union':
            lines.append('    // Union alternative '+field+': '+typ+' (offset 0).')
        elif tail:
            lines.append('    // PORT-BLOCKED[omega:runtime-layout-strides]: '+field+': '+typ+' needs a bounded runtime tail view.')
        else:
            target = translated_type(typ)
            lines.append('    '+field+': '+target+';')
            if target == 'addr':
                lines.append('    // Foreign carrier: '+typ)
            mapped.append((field, target))
        if field == 'raw' and record.get('base'):
            # tuple newtypes vs primitive aliases/bitflags
            original = '0' if record['kind'] == 'struct' and typ.startswith('[') else None
        else:
            original = field
        if original is not None:
            key = name+'.'+field+'.offset'
            probe.append(f'println!("{key}={{}}", offset_of!({rust}, {original}));')
            measurements[key] = {'kind':'offset','value':0}
    if record['kind'] == 'union':
        storage = union_storage[name]
        lines.append('    raw: '+storage+';')
        mapped = [('raw', storage)]
    lines += ['}', '']
    record['omega_fields'] = mapped
    for kind, op in [('size','size_of'),('alignment','align_of')]:
        key=name+'.'+kind
        probe.append(f'println!("{key}={{}}", {op}::<{rust}>());')
        measurements[key]={'kind':kind,'value':0}

lines += ['pub const IP_ADDRESS_ZERO: IpAddress = IpAddress { raw: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] };', '']
for constant in constants:
    name, rust = constant['name'], constant['rust']
    if constant['kind'] == 'bytes':
        guid = uuid.UUID(constant['guid'])
        a,b,c,_,_,_ = guid.fields
        value = f'Guid {{ data1: 0x{a:08x}, data2: 0x{b:04x}, data3: 0x{c:04x}, data4: ['+', '.join(f'0x{n:02x}' for n in guid.bytes[8:])+'] }'
        lines.append('pub const '+name+': Guid = '+value+';')
        probe.append('{ let bytes = '+rust+'.to_bytes(); print!("'+name+'="); for b in bytes { print!("{:02x}", b); } println!(); }')
    else:
        typ=constant.get('primitive',constant['owner'])
        value=str(constant['value']) if 'primitive' in constant else typ+' { raw: '+str(constant['value'])+' }'
        lines.append('pub const '+name+': '+typ+' = '+value+';')
        probe.append(f'println!("{name}={{}}", {rust}'+constant['accessor']+');')
    measurements[name]={'kind':constant['kind'],'value':constant['value']}
lines += ['// RFC2131 section2/Figure2; differs from pinned BROADCAST=1 (see network.PORT.md).', 'pub const DHCP_RFC2131_BROADCAST_BIT: u16 = 0x8000;']
probe.append('}')
(RAW/'network.omg').write_text('\n'.join(lines)+'\n')
(HERE/'src/main.rs').write_text('\n'.join(probe)+'\n')
(HERE/'schema.json').write_text(json.dumps({'records':records,'constants':constants},indent=2)+'\n')
(HERE/'expected-template.json').write_text(json.dumps({'format':'cathedral-port-vectors-v1',
    'target':{'abi':'uefi-x86_64-pinned-uefi-raw','pointer_bits':64,'endian':'little'},
    'provenance':{'kind':'upstream','revision':PIN,'description':'Template; measure pinned Rust before claiming values.', 'sources':SOURCES},
    'measurements':measurements},indent=2)+'\n')
print(f'Generated {len(records)} records and {len(constants)} constants; values still require independent measurement.')
