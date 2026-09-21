#!/usr/bin/env python3
"""Verify current Omega and Rust stage records without relabelling their evidence."""
import hashlib,json
from pathlib import Path
import fixtures,check
HERE=fixtures.HERE;ROOT=fixtures.ROOT
record=json.loads((HERE/'verification.json').read_text());names=[r['name']for r in fixtures.cases()]
assert record['format']=='cathedral-acpi-string-numbers-checked-v1'
assert record['cases']==names and record['scenario_count']==record['control_count']==len(names)
assert record['source_sha256']==check.snapshot()
for name in names:
 for suffix,value in [('positive',0),('control',1)]:assert record['output'].count(f'PASS Suite::{name}_{suffix} expected={value} observed={value} error=None ')==1
assert '\nFAIL 'not in record['output']
constant=json.loads((HERE/'const-verification.json').read_text())
for path,want in constant['source_sha256'].items():assert hashlib.sha256((ROOT/path).read_bytes()).hexdigest()==want,path
assert len(constant['runs'])==2*len(constant['cases'])
reference=json.loads((HERE/'reference-verification.json').read_text())
for path,want in reference['source_sha256'].items():assert hashlib.sha256((HERE/path).read_bytes()).hexdigest()==want,path
for path,want in reference['upstream_sha256'].items():assert hashlib.sha256((ROOT/'reference_code/rust-osdev/acpi'/path).read_bytes()).hexdigest()==want,path
assert reference['actual_public_method_count']==116 and reference['formatter_mirror_count']==76
print(f'Current hashes: {len(names)} Omega execution/control pairs; {len(constant["cases"])} const pairs; 116 public Rust calls and 76 labelled formatting mirrors. Native Omega/hardware not run.')
