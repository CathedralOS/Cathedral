#!/usr/bin/env python3
"""Audit current executable inputs, generated controls and exact pinned public evidence."""
import argparse,json,hashlib
from pathlib import Path
import check,reference
HERE=Path(__file__).resolve().parent
p=argparse.ArgumentParser();p.add_argument('--repository',type=Path,default=check.ROOT);a=p.parse_args()
check.verify();assert json.loads((HERE/'verification.json').read_text())['execution_root']==str(check.ROOT)
r=json.loads((HERE/'reference-verification.json').read_text());up,hashes=reference.upstream(a.repository)
assert r['pin']==reference.PIN and r['upstream_sha256']==hashes
assert r['source_sha256']=={n:reference.sha(HERE/n)for n in ['reference.py','reference.rs','reference.Cargo.lock','aml_encoding.py']}
expected=reference.cases();assert len(r['rows'])==len(expected)==19
for row,fixture in zip(r['rows'],expected):
 assert {k:row[k]for k in fixture}==fixture
 assert row['aml_hex']==reference.table(fixture).hex()
 reference.validate(fixture,row['observed'])
print('PASS 19 actual public Index observations, exact four probe inputs and 27 upstream hashes')
