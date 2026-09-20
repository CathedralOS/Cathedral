#!/usr/bin/env python3
"""Measure exact extracted pure Rust bodies and cross-check actual target Pcid constructors."""
import argparse,json,subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
p=argparse.ArgumentParser();p.add_argument('--write',action='store_true');args=p.parse_args()
command=['cargo','+nightly-2026-09-04'];tail=['--quiet','--locked','--manifest-path',str(HERE/'Cargo.toml')]
output=subprocess.check_output(command+['run']+tail,cwd=ROOT,text=True);rows={}
for line in output.splitlines():
 values=line.split();name=values[0]
 for i,v in enumerate(values[1:]):
  key=name if len(values)==2 else name+'.'+str(i)
  kind='size' if name.endswith('_size') else 'alignment' if name.endswith('_align') else 'offset' if name.endswith('_offset') else 'value'
  rows[key]={'kind':kind,'value':int(v)}
subprocess.run(command+['check','--lib','--target','x86_64-unknown-uefi']+tail,cwd=ROOT,check=True)
vector={'format':'cathedral-port-vectors-v1','target':{'pointer_bits':64,'endian':'little','abi':'raw INVPCID repr(C) mirror; actual Pcid UEFI-x64 assertions'},'provenance':{'kind':'upstream','revision':'cc35c876d3badb57df54a66e22f7768a52be95f2','sources':['src/instructions/tlb.rs'],'description':'Exact extracted private pure bodies on host with real upstream address/Page/Step types. Hardware tails omitted. Actual upstream Pcid type and constructor boundaries compile-asserted for UEFI x64. Descriptor measurements use its exact private repr(C) source mirror; no Omega native ABI claim.'},'measurements':rows}
p=ROOT/'source/libraries/x86_64/tlb-operands.vectors.json'
if args.write:p.write_text(json.dumps(vector,indent=2)+'\n')
elif not p.exists() or json.loads(p.read_text())!=vector:raise SystemExit('Rust measurements changed')
print('PASS',len(rows),'exact-body Rust observations;6 actual upstream UEFI-x64 Pcid assertions')
