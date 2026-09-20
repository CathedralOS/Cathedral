#!/usr/bin/env python3
"""Measure pinned host Rust, cross-check every numeric fact on UEFI x64."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
sys.path.insert(0,str(HERE.parent))
import inventory
import vectors
parser=argparse.ArgumentParser()
parser.add_argument('--write',action='store_true',help='replace checked-in vectors after successful cross-check')
args=parser.parse_args()
path=ROOT/'source/contracts/uefi/raw/hii.vectors.json'
doc=json.loads((HERE/'expected-template.json').read_text())
inventory.sources(ROOT/'reference_code/rust-osdev/uefi-rs',doc['provenance']['revision'],doc['provenance']['sources'])
output=subprocess.check_output(['cargo','run','--locked','--quiet','--manifest-path',str(HERE/'Cargo.toml')],text=True)
rows={}
for line in output.splitlines():
    key,value=line.split('=',1)
    if key in rows:raise ValueError('duplicate probe row '+key)
    rows[key]=value
if set(rows)!=set(doc['measurements']):raise ValueError('probe coverage mismatch')
for key,row in doc['measurements'].items():row['value']=rows[key] if row['kind']=='bytes' else int(rows[key])
source=(HERE/'src/main.rs').read_text()
checks=['#![no_std]','// Generated upstream UEFI x64 compile-time checks; no Omega execution.','use core::mem::{size_of, align_of, offset_of};','const _: () = {']
for key,expression in re.findall(r'println!\("([^"=]+)=\{\}",\s*(.+)\);',source):
    checks.append('assert!('+expression+' == '+str(doc['measurements'][key]['value'])+');')
# GUID to_bytes is const; preserve target byte order and check every byte.
for expression,key in re.findall(r'\{let bytes=(.+)\.to_bytes\(\);print!\("([^"=]+)="\)',source):
    vals=', '.join('0x'+doc['measurements'][key]['value'][i:i+2] for i in range(0,32,2))
    checks.append('{ let actual = ('+expression+').to_bytes(); let expected = ['+vals+']; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } }')
checks.append('};')
if len(checks)-5 != len(rows): raise ValueError('target assertion coverage differs from measurements')
(HERE/'src/lib.rs').write_text('\n'.join(checks)+'\n')
subprocess.run(['cargo','check','--locked','--quiet','--lib','--manifest-path',str(HERE/'Cargo.toml'),'--target','x86_64-unknown-uefi'],check=True)
doc['provenance']['description']='Pinned Rust measurements from aarch64-apple-darwin, each numeric fact and GUID byte independently asserted by Rust const evaluation for x86_64-unknown-uefi. No Omega inspection or execution.'
doc['provenance']['rustc']=subprocess.check_output(['rustc','--version'],text=True).strip()
vectors.validate(doc)
if args.write:path.write_text(json.dumps(doc,indent=2)+'\n')
else:
    expected=json.loads(path.read_text())
    if expected['measurements']!=doc['measurements']:raise ValueError('checked-in vectors differ; review before --write')
print(str(len(rows))+' upstream layout/value vectors measured and cross-checked on x86_64-unknown-uefi; Omega NOT RUN')
