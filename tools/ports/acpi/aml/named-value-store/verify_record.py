#!/usr/bin/env python3
"""Verify final checked stores and exact pinned public Store observations."""
import hashlib,json
from pathlib import Path
import check,reference
HERE=check.HERE;ROOT=check.ROOT
checked=json.loads((HERE/'verification.json').read_text())
assert checked['execution_root']==str(ROOT.resolve()) and checked['build_text']==check.build()
check.verify()
r=json.loads((HERE/'reference-verification.json').read_text());up,hashes=reference.upstream(ROOT)
assert r['execution_root']==str(ROOT.resolve()) and r['probe_root']==str(HERE.resolve())
assert r['pin']==reference.PIN and r['upstream_sha256']==hashes
assert r['source_sha256']=={n:reference.sha(HERE/n)for n in ['reference.py','reference.rs','reference.Cargo.lock','aml_encoding.py']}
assert r['production_mapping_sha256']=={n:reference.sha(ROOT/n)for n in reference.MAPPING}
build=f'[package]\nname="cathedral-named-value-store-reference"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nacpi={{path="{up}"}}\n[[bin]]\nname="reference"\npath="{HERE}/reference.rs"\n'
assert r['build_text']==build and r['build_sha256']==hashlib.sha256(build.encode()).hexdigest()
assert r['binary_sha256']==reference.sha(Path('/tmp/cathedral-named-value-store-public/release/reference'))
expected=reference.cases();assert len(r['rows'])==len(expected)==297
for row,fixture in zip(r['rows'],expected):
 assert {k:row[k]for k in fixture}==fixture
 assert row['aml_hex']==reference.table(fixture).hex()
 reference.validate(fixture,row['observed'])
assert {k:sum(row['classification']==k for row in r['rows'])for k in ['matches','conversion-policy-difference','excluded-profile-observation','primary-rejection-pin-success']}=={'matches':68,'conversion-policy-difference':116,'excluded-profile-observation':97,'primary-rejection-pin-success':16}
print('PASS 297 public named Store observations; exact current inputs/root/build and27upstream hashes')
