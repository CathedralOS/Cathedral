#!/usr/bin/env python3
"""Reviewed pinned record model for fixed ACPI parsers; no firmware execution."""
import hashlib
from pathlib import Path
import re
import subprocess
import tempfile

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
UP=ROOT/'reference_code/rust-osdev/acpi'
PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
FILES=['src/address.rs','src/sdt/fadt.rs','src/sdt/madt.rs','src/sdt/mcfg.rs','src/sdt/hpet.rs']
EXCLUDE={'GenericAddress','HpetInfo'}
MADT_KINDS={'LocalApicEntry':0,'IoApicEntry':1,'InterruptSourceOverrideEntry':2,'NmiSourceEntry':3,
 'LocalApicNmiEntry':4,'LocalApicAddressOverrideEntry':5,'IoSapicEntry':6,'LocalSapicEntry':7,
 'PlatformInterruptSourceEntry':8,'LocalX2ApicEntry':9,'X2ApicNmiEntry':10,'GiccEntry':11,
 'GicdEntry':12,'GicMsiFrameEntry':13,'GicRedistributorEntry':14,'GicInterruptTranslationServiceEntry':15,
 'MultiprocessorWakeupEntry':16}


def records():
    if subprocess.check_output(['git','-C',str(UP),'rev-parse','HEAD'],text=True).strip()!=PIN:raise ValueError('ACPI pin differs')
    result={}
    for file in FILES:
        source=(UP/file).read_text()
        pinned=subprocess.check_output(['git','-C',str(UP),'show',PIN+':'+file]).decode()
        if source!=pinned:raise ValueError('ACPI source differs: '+file)
        for match in re.finditer(r'pub struct (\w+)\s*\{(.*?)\n\}',source,re.S):
            name,body=match.groups()
            if name in EXCLUDE:continue
            fields=[]
            for line in body.splitlines():
                line=line.split('//',1)[0]
                field=re.match(r'\s*(?:pub )?(\w+): (.+),\s*$',line)
                if field:fields.append({'name':field[1],'type':field[2]})
            result[name]={'source':file,'source_sha256':hashlib.sha256(source.encode()).hexdigest(),'fields':fields,
                          'representation':'C' if name=='MultiprocessorWakeupMailbox' else 'C, packed'}
    return result


def rust_measure(model):
    text='''// SPDX-License-Identifier: MIT OR Apache-2.0
// Modified pinned acpi type declarations for host layout observation only.
#![allow(dead_code)]
use std::marker::PhantomPinned;
use std::mem::{MaybeUninit, size_of, align_of, offset_of};
#[derive(Clone, Copy)] struct Signature([u8;4]);
#[derive(Clone, Copy)] struct FixedFeatureFlags(u32);
#[derive(Clone, Copy)] struct IaPcBootArchFlags(u16);
#[derive(Clone, Copy)] struct ArmBootArchFlags(u16);
#[derive(Clone, Copy)] #[repr(transparent)] struct ExtendedField<T: Copy, const MIN: u8>(MaybeUninit<T>);
#[derive(Clone, Copy)] #[repr(C, packed)] struct SdtHeader {
 signature: Signature, length:u32, revision:u8, checksum:u8, oem_id:[u8;6], oem_table_id:[u8;8],
 oem_revision:u32, creator_id:[u8;4], creator_revision:u32,
}
'''
    for name,row in model.items():
        text+=f"#[derive(Clone, Copy)] #[repr({row['representation']})] struct {name} {{\n"
        text+=''.join(f" {f['name']}: {f['type']},\n" for f in row['fields'])+'}\n'
    text+='fn main() {\n'
    for name,row in model.items():
        text+=f'println!("{name}.size={{}}",size_of::<{name}>());\nprintln!("{name}.alignment={{}}",align_of::<{name}>());\n'
        for field in row['fields']:text+=f'println!("{name}.{field["name"]}={{}}",offset_of!({name},{field["name"]}));\n'
    text+='}\n'
    with tempfile.TemporaryDirectory(prefix='cathedral-acpi-rust-layout-') as directory:
        path=Path(directory);(path/'main.rs').write_text(text)
        subprocess.run(['rustc','--edition=2024',str(path/'main.rs'),'-o',str(path/'probe')],check=True)
        output=subprocess.check_output([str(path/'probe')],text=True)
    return {key:int(value)for key,value in (line.split('=')for line in output.splitlines())}


if __name__=='__main__':
    model=records();values=rust_measure(model)
    for name in model:print(name,values[name+'.size'],values[name+'.alignment'])
    print(len(values),'compiled Rust layout observations; not Omega ABI')
