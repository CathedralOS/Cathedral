#!/usr/bin/env python3
"""Verify current production closure, selected bodies/controls and public pin receipt."""
import argparse,json
from pathlib import Path
import check,reference
HERE=Path(__file__).resolve().parent
p=argparse.ArgumentParser();p.add_argument('--repository',type=Path,default=check.ROOT);args=p.parse_args()
check.verify()
assert json.loads((HERE/"verification.json").read_text())["execution_root"]==str(check.ROOT)
r=json.loads((HERE/'reference-verification.json').read_text());up,hashes=reference.upstream(args.repository)
assert r['pin']==reference.PIN and r['upstream_sha256']==hashes
assert r['source_sha256']=={n:reference.sha(HERE/n)for n in ['reference.py','reference.rs','aml_encoding.py','reference.Cargo.lock']}
assert len(r['rows'])==44 and [dict((k,v)for k,v in row.items()if k in expected)for row,expected in zip(r['rows'],reference.cases())]==reference.cases()
for row in r['rows']:
 data=reference.aml.method('MAIN',b'\xa4\x9c'+reference.aml.value(dict(buffer=row['data']))+reference.aml.integer(row['maximum'])+b'\x00')
 assert row['aml_hex']==data.hex() and row['observed']['forbidden_calls']=='0' and row['observed']['load']=='ok'
 buf=bytes.fromhex(row['data']);nul=buf.find(b'\0');raw=(buf if nul<0 else buf[:nul+1])[:row['maximum']]
 try:raw.decode('utf8');assert row['observed'].get('result')=='string:'+raw.hex()
 except UnicodeDecodeError:assert row['observed'].get('result','').startswith('error:InvalidOperationOnObject')
 matches=row['observed'].get('result')=='string:'+row['strict'].get('hex','!')or('error'in row['strict']and row['observed'].get('result','').startswith('error:'))
 assert row['matches_strict']==matches
assert sum(row['matches_strict']for row in r['rows'])==33
print('PASS 44 public observations, 33 matching and 11 differences; exact probe and all pinned source hashes')
