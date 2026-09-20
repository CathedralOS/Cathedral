#!/usr/bin/env python3
"""Observe pinned Rust values and assert const-compatible values on UEFI x64."""
from pathlib import Path
import argparse
import json
import subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
parser=argparse.ArgumentParser();parser.add_argument('--write',action='store_true');args=parser.parse_args()
expected=json.loads((HERE/'measurements.json').read_text())
output=subprocess.check_output(['cargo','run','--locked','--quiet','--manifest-path',str(HERE/'Cargo.toml')],cwd=ROOT,text=True)
actual={name:int(raw) for name,raw in (line.split('=',1) for line in output.splitlines())}
if actual!={name:row['value'] for name,row in expected.items()}:raise SystemExit('upstream value observations disagree with source transcription')
source=['#![no_std]','// Generated upstream-only target assertions; no privileged instructions.']
for name,row in expected.items():
 if name=='MXCSR_RESET_BITS':continue # Trait Default is not const; actual body observed on host.
 source.append('const _: () = assert!(('+row['expression']+') as u64 == '+str(row['value'])+'u64);')
(HERE/'src/lib.rs').write_text('\n'.join(source)+'\n')
subprocess.run(['cargo','check','--locked','--quiet','--lib','--manifest-path',str(HERE/'Cargo.toml'),'--target','x86_64-unknown-uefi'],cwd=ROOT,check=True)
vector={'format':'cathedral-port-vectors-v1','target':{'pointer_bits':64,'endian':'little','abi':'x86 register numeric values; no aggregate ABI'},'provenance':{'kind':'upstream','description':f'{len(actual)} actual pinned Rust value observations on host; {len(actual)-1} const-compatible facts independently asserted for x86_64-unknown-uefi. MXCSR Default body is host-observed only. MSR one-field private Rust carriers are inspected by test-only transmute, not adopted as foreign ABI. No Omega observation or hardware execution.','sources':json.loads((HERE/'schema.json').read_text())['files'],'revision':'cc35c876d3badb57df54a66e22f7768a52be95f2'},'measurements':{name:{'kind':'value','value':raw} for name,raw in actual.items()}}
p=ROOT/'source/drivers/facts/x86_registers.vectors.json'
if args.write:p.write_text(json.dumps(vector,indent=2)+'\n')
elif not p.exists() or json.loads(p.read_text())!=vector:raise SystemExit('register vectors changed; review before --write')
print(f'PASS {len(actual)} Rust-observed values; {len(actual)-1} UEFI-x64 const assertions; MXCSR default host-only')
