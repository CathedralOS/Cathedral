#!/usr/bin/env python3
"""Original synthetic ACPI header fixtures; no firmware or upstream AML inputs."""
import argparse
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]


def fix(data, offset, length=None):
    data[offset] = 0
    data[offset] = (-sum(data[:length])) & 255
    return data


def rsdp(revision=2, length=36):
    data = bytearray(max(20, length))
    data[:8] = b'RSD PTR '
    data[9:15] = b'CATOS '
    data[15] = revision
    data[16:20] = (0x81234567).to_bytes(4, 'little')
    if revision:
        data[20:24] = length.to_bytes(4, 'little')
        data[24:32] = (0xfedcba9876543210).to_bytes(8, 'little')
    fix(data, 8, 20)
    if revision:
        fix(data, 32)
    return data


def sdt(signature=b'RSDT', payload=b'', revision=1):
    data = bytearray(36) + payload
    data[:4] = signature
    data[4:8] = len(data).to_bytes(4, 'little')
    data[8] = revision
    data[10:16] = b'CATOS '
    data[16:24] = b'ORIGINAL'
    data[24:28] = (0x12345678).to_bytes(4, 'little')
    data[28:32] = b'CATL'
    data[32:36] = (0x87654321).to_bytes(4, 'little')
    return fix(data, 9)


def cases():
    rows = []
    def add(name, kind, data, error=0, length=None, **extra):
        rows.append(dict(name=name, kind=kind, bytes=data.hex(), length=len(data) if length is None else length,
                         error=error, **extra))
    v1, v2 = rsdp(0,20), rsdp()
    add('rsdp_legacy', 'rsdp', v1, address=0x81234567, revision=0, declared=20)
    add('rsdp_extended', 'rsdp', v2, address=0x81234567, revision=2, declared=36, xsdt=0xfedcba9876543210)
    add('rsdp_trailing_ignored', 'rsdp', v1 + b'not part of legacy', declared=20)
    add('rsdp_empty', 'rsdp', b'', 2)
    add('rsdp_short', 'rsdp', v1, 2, length=19)
    add('rsdp_capacity', 'rsdp', v2, 1, length=4097)
    add('rsdp_short_extended', 'rsdp', v2, 2, length=35)
    data=bytearray(v2); data[0]=0; add('rsdp_signature', 'rsdp', data,3)
    for rev in (1,3,255):
        data=bytearray(v2);data[15]=rev;fix(data,8,20);fix(data,32)
        add('rsdp_revision_'+str(rev),'rsdp',data,4)
    data=bytearray(v2);data[9]=128;fix(data,8,20);fix(data,32);add('rsdp_oem_bytes','rsdp',data,raw_oem=128)
    data=bytearray(v2);data[8]=(data[8]+1)&255;add('rsdp_legacy_checksum','rsdp',data,6)
    fix(data,32);add('rsdp_compensated_legacy_checksum','rsdp',data,6)
    data=bytearray(v2);data[32]=(data[32]+1)&255;add('rsdp_extended_checksum','rsdp',data,6)
    for declared in (0,20,35):
        data=bytearray(v2);data[20:24]=declared.to_bytes(4,'little');fix(data,32)
        add('rsdp_declared_'+str(declared),'rsdp',data,5)
    data=bytearray(v2);data[20:24]=(37).to_bytes(4,'little');fix(data,32);add('rsdp_declared_truncated','rsdp',data,2)
    data=bytearray(v2);data[20:24]=(0xffffffff).to_bytes(4,'little');fix(data,32);add('rsdp_declared_max','rsdp',data,2)
    data=rsdp(2,40);data[36:40]=b'tail';fix(data,32);add('rsdp_extension','rsdp',data,declared=40)
    data[-1]^=1;add('rsdp_extension_checksum','rsdp',data,6)
    for i in (33,34,35):
        data=bytearray(v2);data[i]=1;fix(data,32);add('rsdp_reserved_'+str(i),'rsdp',data,reserved_index=i-33)
    data=rsdp(2,4096);data[-1]=255;fix(data,32);add('rsdp_capacity_exact','rsdp',data,declared=4096)
    data=bytearray(v2);data[24:32]=bytes(8);fix(data,32);add('rsdp_zero_xsdt','rsdp',data,xsdt=0,selected=0x81234567)
    data[16:20]=bytes(4);fix(data,8,20);fix(data,32);add('rsdp_no_root','rsdp',data,xsdt=0,selected=0)
    base=sdt()
    add('sdt_header','sdt',base,declared=36)
    add('sdt_trailing_ignored','sdt',base+b'extra',declared=36)
    add('sdt_empty','sdt',b'',2)
    add('sdt_short','sdt',base,2,length=35)
    add('sdt_capacity','sdt',base,1,length=4097)
    add('sdt_wrong_signature','sdt',sdt(b'XSDT'),3)
    for declared in (0,35):
        data=bytearray(base);data[4:8]=declared.to_bytes(4,'little');fix(data,9)
        add('sdt_declared_'+str(declared),'sdt',data,5)
    for declared in (37,0xffffffff):
        data=bytearray(base);data[4:8]=declared.to_bytes(4,'little');fix(data,9)
        add('sdt_declared_'+str(declared),'sdt',data,2)
    for offset in (10,16):
        data=bytearray(base);data[offset]=128;fix(data,9);add('sdt_oem_bytes_'+str(offset),'sdt',data,raw_oem=128 if offset==10 else None,raw_table=128 if offset==16 else None)
    data=bytearray(base);data[9]=(data[9]+1)&255;add('sdt_checksum','sdt',data,6)
    add('sdt_revision_preserved','sdt',sdt(revision=255),declared=36,revision=255)
    for xsdt,width in ((False,4),(True,8)):
        sig=b'XSDT' if xsdt else b'RSDT'
        addresses=[0x81234567,0xfedcba9876543210 if xsdt else 0xffffffff]
        data=sdt(sig,b''.join(a.to_bytes(width,'little') for a in addresses))
        prefix='xsdt' if xsdt else 'rsdt'
        add(prefix+'_entries','root',data,xsdt=xsdt,count=2,addresses=addresses)
        add(prefix+'_empty','root',sdt(sig),xsdt=xsdt,count=0,addresses=[])
        add(prefix+'_partial','root',sdt(sig,b'\0'),9,xsdt=xsdt)
        add(prefix+'_revision','root',sdt(sig,revision=2),4,xsdt=xsdt)
        data=bytearray(data);data[-1]^=1;add(prefix+'_checksum','root',data,6,xsdt=xsdt)
    for xsdt,width,count in ((False,4,1015),(True,8,507)):
        payload=bytearray(width*count);address=0xfedcba9876543210 if xsdt else 0xffffffff
        payload[-width:]=address.to_bytes(width,'little')
        add('root_last_'+str(width),'entry',sdt(b'XSDT' if xsdt else b'RSDT',payload),xsdt=xsdt,index=count-1,address=address)
    for tail in range(1,8):
        data=bytes(range(1,9+tail));add('checksum_tail_'+str(tail),'checksum',data,sum=sum(data)&255)
    add('checksum_wrap','checksum',bytes([255])*257,sum=255)
    add('checksum_empty','checksum',b'',sum=0)
    add('checksum_capacity','checksum',b'',1,length=4097)
    add('checksum_capacity_exact','checksum',bytes(4095)+b'\xff',sum=255)
    return rows


def render(rows, evaluate=True):
    out=['// SPDX-License-Identifier: MIT OR Apache-2.0',
         '// Generated original fixtures by header_fixtures.py; do not edit.',
         'use acpi::headers::RsdpResult;','use acpi::headers::SdtResult;',
         'use acpi::headers::RootResult;','use acpi::headers::EntryResult;',
         'use acpi::headers::ChecksumResult;','use acpi::headers::RootPointer;',
         'use acpi::headers::SIGNATURE_RSDT;','use acpi::parser::parse_rsdp;',
         'use acpi::parser::parse_sdt;','use acpi::parser::parse_root;',
         'use acpi::parser::root_entry;','use acpi::parser::select_root;',
         'use acpi::bytes::checksum;']
    for row in rows:
        kind=row['kind'];length=row['length'];xsdt=str(row.get('xsdt',False)).lower()
        out += [f"machine test_{row['name']}() -> bool {{",'    let mut input: [u8; 4096];']
        out += [f'    input[{i}] = {b};'for i,b in enumerate(bytes.fromhex(row['bytes'])) if b]
        typ={'rsdp':'RsdpResult','sdt':'SdtResult','root':'RootResult','checksum':'ChecksumResult','entry':'EntryResult'}[kind]
        call={'rsdp':f'parse_rsdp(&input, {length})','sdt':f'parse_sdt(&input, {length}, SIGNATURE_RSDT)',
              'root':f'parse_root(&input, {length}, {xsdt})','checksum':f'checksum(&input, {length})','entry':f"root_entry(&input, {length}, {xsdt}, {row.get('index',0)})"}[kind]
        out += [f'    let result: {typ} = {call};']
        checks=[f"result.error == {row['error']}"]
        if not row['error']:
            for key,field in [('address','rsdt_address'),('declared','length'),('revision','revision')]:
                if key in row and kind in ('rsdp','sdt'):checks.append(f"result.value.{field} == {row[key]}")
            if kind=='rsdp' and isinstance(row.get('xsdt'),int):checks.append(f"result.value.xsdt_address == {row['xsdt']}")
            if kind=='rsdp' and 'reserved_index' in row:checks.append(f"result.value.reserved[{row['reserved_index']}] == 1")
            if row.get('raw_oem') is not None:checks.append(f"result.value.oem_id[0] == {row['raw_oem']}")
            if row.get('raw_table') is not None:checks.append(f"result.value.oem_table_id[0] == {row['raw_table']}")
            if kind=='sdt':checks += ['result.value.oem_revision == 0x12345678','result.value.creator_id == 0x4c544143','result.value.creator_revision == 0x87654321']
            if kind=='root':
                checks += [f"result.entry_count == {row['count']}",f"result.entry_size == {8 if row['xsdt'] else 4}"]
                for i,address in enumerate(row['addresses']):
                    out += [f'    let entry{i}: EntryResult = root_entry(&input, {length}, {xsdt}, {i});']
                    checks += [f'entry{i}.error == 0',f'entry{i}.address == {address}']
                out += [f"    let end: EntryResult = root_entry(&input, {length}, {xsdt}, {row['count']});",
                        f'    let huge: EntryResult = root_entry(&input, {length}, {xsdt}, 18446744073709551615);']
                checks += ['end.error == 10','huge.error == 10']
            if kind=='entry':checks.append(f"result.address == {row['address']}")
            if kind=='rsdp' and 'selected' in row:
                out += ['    let selected: RootPointer = select_root(result, false);']
                checks += [f"selected.address == {row['selected']}",f"selected.available == {str(row['selected'] != 0).lower()}",'selected.entry_size == 4']
            if kind=='checksum': checks.append(f"result.sum == {row['sum']}")
            if row['name']=='rsdp_extended':
                out += ['    let preferred: RootPointer = select_root(result, false);','    let fallback: RootPointer = select_root(result, true);']
                checks += ['preferred.available','preferred.address == 0xfedcba9876543210','preferred.entry_size == 8',
                           'fallback.available','fallback.address == 0x81234567','fallback.entry_size == 4']
        elif kind=='rsdp':
            out += ['    let selected: RootPointer = select_root(result, false);']
            checks += ['!selected.available','selected.address == 0']
        out += ['    '+' &&\n    '.join(checks),' }']
    if evaluate:
        out += ['machine test_result() -> i32 {','    transition '+' &&\n        '.join(f"test_{r['name']}()"for r in rows)+' { true -> (0) _ -> (1) }','}',
                'const TEST_RESULT: i32 = test_result();','machine require_success(value: i32) requires value == 0; {}',
                'data Main {}','machine Main::main(&mut self) { require_success(TEST_RESULT); }','']
    else:
        out += ['// Source-check every authored fixture; check.py evaluates bounded groups separately.',
                'data Main {}','machine Main::main(&mut self) {}','']
    return '\n'.join(out)


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args()
    rows=cases()
    outputs={HERE/'header-cases.json':json.dumps({'format':'cathedral-acpi-header-cases-v1','provenance':'Original initialized synthetic ACPI6.6 test inputs; no captured firmware or upstream AML test input.','cases':rows},indent=2)+'\n', HERE/'main.omg':render(rows, evaluate=False)}
    for path,text in outputs.items():
        if args.check:
            if not path.exists() or path.read_text()!=text:raise SystemExit(f'fixture differs: {path}')
        else:path.write_text(text)
    print(f'{len(rows)} original header cases '+('verified'if args.check else 'generated'))


if __name__=='__main__':main()
