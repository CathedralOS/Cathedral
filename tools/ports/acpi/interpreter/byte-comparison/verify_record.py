#!/usr/bin/env python3
"""Check exact retained source hashes; no fresh execution is implied."""
import hashlib,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
records={name:json.loads((HERE/name).read_text())for name in ['verification.json','const-verification.json','reference-verification.json']}
for name,record in records.items():
 for path,digest in record['source_sha256'].items():assert hashlib.sha256((ROOT/path).read_bytes()).hexdigest()==digest,(name,path)
v=records['verification.json'];assert v['scenario_count']==81 and v['control_count']==81 and v['output'].count('PASS Suite::')==162
assert len(records['const-verification.json']['runs'])==8
assert records['reference-verification.json']['actual_public_calls']==62
print('PASS retained81checked pairs,4const pairs,62public Rust calls and source hashes')
