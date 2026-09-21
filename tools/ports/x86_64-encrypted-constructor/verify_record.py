#!/usr/bin/env python3
"""Verify retained hashes/counts, without implying fresh execution."""
import hashlib,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
for name in ['checked-verification.json','constant-verification.json']:
 r=json.loads((HERE/name).read_text())
 for p,h in r['source_sha256'].items():assert hashlib.sha256((ROOT/p).read_bytes()).hexdigest()==h,(name,p)
 if name.startswith('checked'):assert r['constructor_case_count']==610 and r['positive_fixture_count']==10 and r['control_count']==10 and r['output'].count('PASS Suite::')==20
 else:assert len(r['runs'])==4
print('PASS current610constructor cases/10controls,2const pairs and source hashes')
