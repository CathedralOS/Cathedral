#!/usr/bin/env python3
"""Bind fixed parsers to pinned source symbols and compiled Rust wire-layout facts."""
import argparse
import json
import re
import subprocess
from fixed_model import ROOT,HERE,UP,PIN,FILES,records,rust_measure
from header_evidence import api,translated,omitted
DEST=ROOT/'source/libraries/acpi'


def evidence():
    model=records();measures=rust_measure(model)
    inventory=api.snapshot(UP,PIN,FILES,'https://github.com/rust-osdev/acpi')
    inventory['scope']='Bounded initialized-byte fixed-table/GAS parsing and pure helper slice; no mapping, register I/O, borrowed native overlays or firmware activation. See fixed.PORT.md.'
    types=(DEST/'fixed_types.omg').read_text()
    for path,file in inventory['files'].items():
        translated(file,'fixed.PORT.md','## Source map','Each source anchor separately classified; hardware/mapping operations explicitly omitted.')
        for key,row in file['symbols'].items():
            lineno,name=key.split(':',1);line=int(lineno)
            if name=='SIGNATURE':
                translated(row,'headers.omg','SIGNATURE_'+{'src/sdt/fadt.rs':'FADT','src/sdt/hpet.rs':'HPET','src/sdt/madt.rs':'MADT','src/sdt/mcfg.rs':'MCFG'}[path]);continue
            if path=='src/address.rs':
                mapping={'RawGenericAddress':'data RawGenericAddress','GenericAddress':'data GenericAddress','AddressSpace':'ADDRESS_SYSTEM_MEMORY','StandardAccessSize':'ACCESS_UNDEFINED','Error':'ERROR_GENERIC_ADDRESS','try_from':'machine standard_access_size','is_empty':'machine gas_is_empty','from_raw':'machine gas_from_raw','standard_access_size':'machine standard_access_size','gas_decode_access_bit_width':'machine gas_decode_access_bit_width'}
                if name in ['MappedGas','map_gas','read','write']:
                    omitted(row,'Hardware ownership/mapping and volatile MMIO or I/O-port access are outside the pure initialized-byte library. A driver adapter needs explicit authority; no inert address becomes a pointer.')
                else:translated(row,'gas.omg',mapping.get(name,name+':'),'Address/access enum identity uses explicit numeric carriers; primary corrections and access-width heuristic limitations are recorded in fixed.PORT.md.')
            elif path=='src/sdt/fadt.rs':
                if name=='PowerProfile':translated(row,'fadt.omg','data PowerProfile')
                elif name in ['FixedFeatureFlags','IaPcBootArchFlags','ArmBootArchFlags']:translated(row,'fadt.omg','// '+name+' represented','Explicit underlying carrier replaces Rust wrapper identity; every bit getter preserved.')
                elif name=='validate':translated(row,'fadt.omg','machine parse_fadt','Bounded signature/checksum/revision-length validation replaces unsafe native overlay validation.')
                elif line<=123:translated(row,'fixed_types.omg','data Fadt'if name=='Fadt'else name+':','Owned decoded fields; optional extension requires revision and actual byte availability.')
                else:translated(row,'fadt.omg','machine '+name)
            elif path=='src/sdt/hpet.rs':
                if name=='new':omitted(row,'Whole mapped AcpiTables lookup constructor needs separate table-discovery authority. Pure field extraction is implemented by hpet_info; do not equate that with the mapped constructor.')
                elif line<71:translated(row,'hpet.omg','data '+name if name in ['PageProtection','HpetInfo']else name+':','Encoded last-comparator index is corrected to count plus1; raw encoded value also retained.')
                else:translated(row,'fixed_types.omg','data HpetTable'if name=='HpetTable'else name+':')
            elif path=='src/sdt/mcfg.rs':
                if name=='fmt':omitted(row,'Rust Debug formatter omitted; owned records retain all wire data.')
                elif name=='entries':translated(row,'mcfg.omg','machine mcfg_entry','Checked indexed access replaces borrowed trailing slice; partial-tail quirk explicit and reported.')
                else:translated(row,'fixed_types.omg','data '+name if name in ['Mcfg','McfgEntry']else name+':')
            else:
                if name=='MadtError':translated(row,'madt.omg','data MadtError','Original inert error variants retained, including inactive wake timeout; parser APIs return the explicit documented numeric carrier.')
                elif name=='get_mpwk_mailbox_addr':translated(row,'madt_search.omg','machine get_mpwk_mailbox_addr','Bounded pure scan returns numeric address only; no mailbox ownership or access.')
                elif name in ['entries','MadtEntryIter','Item','next']:translated(row,'madt.omg','machine madt_next','Explicit cursor step replaces borrowed iterator. Unknown records observable, length/typed-prefix checked, each successful nonterminal step progresses at least2 bytes.')
                elif name in ['MadtEntry','Polarity','TriggerMode','MpProtectedModeWakeupCommand']:translated(row,'madt.omg','data '+name)
                elif name=='from':translated(row,'madt.omg','machine wakeup_command','Unsupported wake command returns error17 instead of panic.')
                elif name in ['Noop','Wakeup','Sleep','AcceptPages']:translated(row,'madt.omg','case '+name)
                elif name in ['supports_8259','parse_mps_inti_flags']:translated(row,'madt.omg','machine '+name)
                else:translated(row,'fixed_types.omg','data '+name if name in model else name+':','Owned logical record, not a repr(C) overlay. Local SAPIC UID tail stays bounded raw bytes via madt_entry_byte; mailbox declaration grants no volatile access.')
    vectors={'format':'cathedral-port-vectors-v1','target':{'pointer_bits':64,'endian':'little','abi':'ACPI serialized packed records; host Rust declaration observation, not Omega ABI'},
        'provenance':{'kind':'upstream','description':'Exact pinned Rust field/representation declarations compiled as a minimal host-only layout probe, numeric facts cross-checked against ACPI6.6/HPET1.0a. No Omega native layout assertion.',
            'sources':['https://github.com/rust-osdev/acpi/tree/'+PIN,'https://uefi.org/specs/ACPI/6.6/05_ACPI_Software_Programming_Model.html','https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/software-developers-hpet-spec-1-0a.pdf']},
        'measurements':{key:{'kind':'size'if key.endswith('.size')else'alignment'if key.endswith('.alignment')else'offset','value':value}for key,value in measures.items()}}
    return inventory,vectors


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args()
    inventory,vectors=evidence()
    observation={'format':'cathedral-acpi-host-rust-layout-v1','upstream_revision':PIN,'source_hashes':{path:row['sha256']for path,row in inventory['files'].items()},'rustc_verbose_version':subprocess.check_output(['rustc','-vV'],text=True),'scope':'Minimal modified pinned declarations compiled and executed on host. Not full upstream crate or Omega native ABI.','measurements':vectors['measurements']}
    for path,data in [(DEST/'fixed-inventory.json',inventory),(DEST/'fixed.vectors.json',vectors),(DEST/'fixed-rust-observation.json',observation)]:
        text=json.dumps(data,indent=2,sort_keys=True)+'\n'
        if args.check:
            if not path.exists() or path.read_text()!=text:raise SystemExit('Fixed evidence differs: '+str(path))
        else:path.write_text(text)
    # Check each pure flag getter against the actual pinned implementation, not a mirrored test.
    rust=(UP/'src/sdt/fadt.rs').read_text();omega=(DEST/'fadt.omg').read_text()
    expected={name:1<<int(bit)for name,bit in re.findall(r'pub fn (\w+)\(&self\) -> bool \{\s*self\.0\.get_bit\((\d+)\)',rust)}
    actual={name:int(bit)for name,bit in re.findall(r'pub machine (\w+)\(flags: u\d+\) -> bool \{ \(flags & (\d+)\)',omega)}
    if expected!=actual or len(actual)!=30:raise SystemExit('Flag getters differ from pin')
    print(len(vectors['measurements']),'compiled Rust declaration layout facts and30 pinned flag getters verified; Omega native ABI NOT RUN')

if __name__=='__main__':main()
