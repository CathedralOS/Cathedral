#!/usr/bin/env python3
"""Measure pinned network Rust layouts and verify every fact on UEFI x86-64."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(HERE.parent))
import inventory
import vectors

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--write', action='store_true')
args = parser.parse_args()
document = inventory.read_json(HERE / 'expected-template.json')
inventory.sources(ROOT / 'reference_code/rust-osdev/uefi-rs', document['provenance']['revision'], document['provenance']['sources'])
output = subprocess.check_output(['cargo','run','--quiet','--locked','--manifest-path',str(HERE/'Cargo.toml')],text=True)
observed = {}
for line in output.splitlines():
    name, value = line.split('=', 1)
    if name in observed:
        raise ValueError('duplicate probe row '+name)
    observed[name] = value
if set(observed) != set(document['measurements']):
    raise ValueError('probe coverage mismatch')
for name, row in document['measurements'].items():
    row['value'] = observed[name] if row['kind'] == 'bytes' else int(observed[name])
probe = (HERE/'src/main.rs').read_text()
checks = ['#![no_std]', '// SPDX-License-Identifier: MIT OR Apache-2.0',
          '// Cross-target assertions of independently measured pinned Rust; not Omega execution.',
          'use core::mem::{size_of,align_of,offset_of};', 'const _: () = {']
for name, expression in re.findall(r'println!\("([^"=]+)=\{\}", (.+)\);', probe):
    checks.append('assert!('+expression+' == '+str(document['measurements'][name]['value'])+');')
for expression, name in re.findall(r'\{ let bytes = (.+)\.to_bytes\(\); print!\("([^"=]+)="\)', probe):
    expected = ', '.join('0x'+document['measurements'][name]['value'][i:i+2] for i in range(0,32,2))
    checks.append('{ let actual = '+expression+'.to_bytes(); let expected = ['+expected+']; let mut i = 0; while i < 16 { assert!(actual[i] == expected[i]); i += 1; } }')
checks.append('};')
(HERE/'src/lib.rs').write_text('\n'.join(checks)+'\n')
subprocess.run(['cargo','check','--quiet','--locked','--lib','--manifest-path',str(HERE/'Cargo.toml'),'--target','x86_64-unknown-uefi'],check=True)
document['provenance']['description'] = 'Pinned Rust host measurements; every numeric fact and GUID byte independently asserted under x86_64-unknown-uefi. No Omega inspection or execution.'
document['provenance']['rustc'] = subprocess.check_output(['rustc','--version'],text=True).strip()
vectors.validate(document)
path = ROOT/'source/contracts/uefi/raw/network.vectors.json'
if args.write:
    path.write_text(json.dumps(document,indent=2)+'\n')
elif inventory.read_json(path)['measurements'] != document['measurements']:
    raise ValueError('vectors differ; review source before --write')
print(f'{len(observed)} pinned upstream network vectors verified on UEFI x86-64; no Omega ABI claim.')
