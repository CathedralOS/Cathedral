#!/usr/bin/env python3
"""Current checked body/control/constant records and exact pinned public probes."""
import argparse,json
from pathlib import Path
import check,reference
HERE=Path(__file__).resolve().parent
p=argparse.ArgumentParser();p.add_argument('--repository',type=Path,default=check.ROOT);a=p.parse_args()
check.verify();assert json.loads((HERE/'verification.json').read_text())['execution_root']==str(check.ROOT)
r=json.loads((HERE/'reference-verification.json').read_text());up,hashes=reference.upstream(a.repository)
assert r['pin']==reference.PIN and r['upstream_sha256']==hashes
assert r['source_sha256']=={n:reference.sha(HERE/n)for n in ['reference.py','reference.rs','reference.Cargo.lock']}
assert len(r['rows'])==54 and [dict((k,v)for k,v in row.items()if k in expected)for row,expected in zip(r['rows'],reference.cases())]==reference.cases()
for row in r['rows']:assert row['classification']==reference.classify(row,row['observed'])
assert {k:sum(x['classification']==k for x in r['rows'])for k in ['matches','differs','excluded-profile-observation']}=={'matches':7,'differs':25,'excluded-profile-observation':22}
print('PASS 54 public observations:7matching,25differences,22excluded-profile; exact probe inputs and27upstream hashes')
