#!/usr/bin/env python3
"""Bind the bounded header slice to exact source anchors and expected wire facts."""
import argparse
import importlib.util
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
spec = importlib.util.spec_from_file_location('port_inventory', HERE.parent/'inventory.py')
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
PIN = '257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
UP = ROOT/'reference_code/rust-osdev/acpi'
DEST = ROOT/'source/libraries/acpi'
SOURCE = 'https://uefi.org/specs/ACPI/6.6/05_ACPI_Software_Programming_Model.html'


def translated(row, file, anchor, note=None):
    row.update(disposition='translated', targets=[{'path':'source/libraries/acpi/'+file,'anchor':anchor}])
    row.pop('reason',None)
    if note:row['note']=note


def omitted(row, reason):
    row.update(disposition='omitted',reason=reason)


def evidence():
    inventory = api.snapshot(UP,PIN,['src/rsdp.rs','src/sdt/mod.rs','src/lib.rs'],'https://github.com/rust-osdev/acpi')
    inventory['scope']='First bounded initialized-byte header/root slice only. Omissions apply to this slice, not the complete ACPI queue. See headers.PORT.md.'
    for path,file in inventory['files'].items():
        translated(file,'headers.PORT.md','## Source map','Partial slice; every source anchor below independently classified.')
        for key,row in file['symbols'].items():
            line,name=key.split(':',1);line=int(line)
            if path=='src/rsdp.rs':
                if name in ['Rsdp','RSDP_V1_LENGTH','RSDP_EXT_LENGTH']:
                    translated(row,'headers.omg',name)
                elif name=='RSDP_SIGNATURE':translated(row,'parser.omg','input[0] == 82')
                elif name=='validate':translated(row,'parser.omg','pub machine parse_rsdp','Strict bounded revision0/2 validation; explicit deviations in headers.PORT.md.')
                elif name in ['signature','checksum','revision','rsdt_address','length','xsdt_address','ext_checksum','oem_id']:
                    translated(row,'headers.omg',name+':','Decoded owned field; OEM bytes retained without a text gate; no borrowed UTF-8 accessor. Legacy unavailable extension fields are zero except synthesized length20.')
                else:omitted(row,'BIOS physical mapping/search constants and operations are outside initialized-byte parsing; requires separately authorized firmware mapping adapter.')
            elif path=='src/sdt/mod.rs':
                if 198<=line<=262:translated(row,'headers.omg','SIGNATURE_'+name)
                elif name in ['SdtHeader','Signature']:translated(row,'headers.omg','data '+name)
                elif name=='validate':translated(row,'parser.omg','pub machine parse_sdt','Bounded exact expected-signature, length and checksum; raw OEM fields. No unsafe overlay or unchecked UTF-8 conversion.')
                elif name in ['signature','length','revision','checksum','oem_id','oem_table_id','oem_revision','creator_id','creator_revision']:
                    translated(row,'headers.omg',name+':','Decoded owned data; creator ID is canonical little-endian u32, preserving four bytes without string conversion.')
                elif name in ['ExtendedField','MIN_REVISION','access']:
                    omitted(row,'Generic revision-gated MaybeUninit field mechanism is not used by the header slice. Fixed-table parsers must decode bounded initialized bytes in ACPI-002.')
                elif name in ['as_str','fmt']:
                    omitted(row,'Rust formatting and borrowed string convenience are absent; signatures and OEM/creator data remain exact decoded carriers.')
                else:omitted(row,'Fixed-table module declaration outside first header slice; full ACPI source inventory retains later work.')
            else:
                if name=='table_entries':translated(row,'parser.omg','pub machine root_entry','Explicit checked indexed byte decoding replaces unsafe iterator; rejects partial entry payloads.')
                elif name=='ignore_xsdt':translated(row,'parser.omg','pub machine select_root','Explicit inert selection argument; no authority or physical mapping.')
                elif name in ['from_rsdp','from_rsdp_with_quirks','from_rsdt','from_rsdt_with_quirks','AcpiTables','AcpiQuirks']:
                    omitted(row,'Whole mapped-table API is outside this slice. Its pure root preference/header/entry suboperations are represented by parse_rsdp/parse_root/select_root; physical mapping and lenient firmware acceptance are intentionally absent.')
                else:omitted(row,'Outside first ACPI-001 header slice: firmware handlers/mapping, fixed tables, AML, policy, formatting and Rust adapter tests are separately inventoried by ACPI-000.')
    measurements={}
    source=(UP/'src/sdt/mod.rs').read_text()
    for name,value in re.findall(r'pub const (\w+): Signature = Signature\(\*b"(.{4})"\);',source):
        measurements['Signature.'+name]={'kind':'value','value':int.from_bytes(value.encode(),'little')}
    for name,value in [('RSDP.legacy_length',20),('RSDP.extended_minimum_length',36),('SDT.header_length',36),('RSDT.entry_width',4),('XSDT.entry_width',8)]:
        measurements[name]={'kind':'size','value':value}
    for record,fields in {'RSDP':{'signature':0,'checksum':8,'oem_id':9,'revision':15,'rsdt_address':16,'length':20,'xsdt_address':24,'ext_checksum':32,'reserved':33},
                          'SDT':{'signature':0,'length':4,'revision':8,'checksum':9,'oem_id':10,'oem_table_id':16,'oem_revision':24,'creator_id':28,'creator_revision':32}}.items():
        for field,value in fields.items():measurements[record+'.'+field]={'kind':'offset','value':value}
    vectors={'format':'cathedral-port-vectors-v1','target':{'pointer_bits':64,'endian':'little','abi':'ACPI6.6 serialized bytes; decoded Omega records are not ABI overlays'},
             'provenance':{'kind':'specification','description':'ACPI6.6 sections5.2.5.3 and5.2.6-5.2.8 wire offsets/lengths plus exact pinned Signature byte literals. Host-checked expectations, not measured Omega ABI.',
                           'sources':[SOURCE,'https://github.com/rust-osdev/acpi/blob/'+PIN+'/src/sdt/mod.rs','https://github.com/rust-osdev/acpi/blob/'+PIN+'/src/rsdp.rs']},'measurements':measurements}
    return inventory,vectors


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args()
    inventory,vectors=evidence()
    for path,data in [(DEST/'headers-inventory.json',inventory),(DEST/'headers.vectors.json',vectors)]:
        text=json.dumps(data,indent=2,sort_keys=True)+'\n'
        if args.check:
            if not path.exists() or path.read_text()!=text:raise SystemExit(f'evidence differs: {path}')
        else:path.write_text(text)
    # Compare every pinned signature literal to actual Omega source, not just regenerated metadata.
    omega=(DEST/'headers.omg').read_text()
    actual={name:int(value,16) for name,value in re.findall(r'pub const SIGNATURE_(\w+): Signature = Signature \{ raw: (0x[0-9a-f]+) \};',omega)}
    expected={name.removeprefix('Signature.'):row['value'] for name,row in vectors['measurements'].items() if name.startswith('Signature.')}
    if actual!=expected:raise SystemExit('Omega signature source differs from pin')
    print(f"{len(actual)} pinned signature literals and {len(vectors['measurements'])} expected wire vectors verified; no native ABI claim")


if __name__=='__main__':main()
