#!/usr/bin/env python3
"""Verify retained stage-specific inputs and results; no new execution implied."""
import hashlib,json,re
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
records={name:json.loads((HERE/name).read_text())for name in ['verification.json','const-verification.json','reference-verification.json']}
for name,record in records.items():
 for path,want in record['source_sha256'].items():assert digest(ROOT/path)==want,(name,path)
# Require the complete current production closure, even for records made before
# this audit utility existed. An added utility does not rewrite old run receipts.
production=set((ROOT/'source/libraries/acpi/resource_composition').glob('*.omg'))
production.update((ROOT/'source/libraries/acpi/resources').glob('*.omg'))
production.update(ROOT/'source/libraries/acpi'/p for p in ['build.omg','fixed_bytes.omg','bytes.omg','headers.omg'])
for name in ['verification.json','const-verification.json']:
 assert {str(p.relative_to(ROOT))for p in production}<=set(records[name]['source_sha256']),name
v=records['verification.json'];names=[r['name']for r in fixtures.cases()]
assert v['cases']==names and v['scenario_count']==v['control_count']==len(names)==25
matches=re.findall(r'^PASS Suite::(\w+)_(positive|control) expected=([01]) observed=([01]) error=None ',v['output'],re.M)
assert len(matches)==50
assert {(n,s)for n,s,_,_ in matches}=={(n,s)for n in names for s in ['positive','control']}
assert all(a==b==str(int(s=='control'))for _,s,a,b in matches)
c=records['const-verification.json'];assert c['cases']==['empty','two_resources','right_one','short_capacity']
assert len(c['runs'])==8 and {(r['case'],r['control'])for r in c['runs']}=={(n,b)for n in c['cases']for b in [False,True]}
for r in c['runs']:
 if r['control']:assert 'cannot prove requires contract'in r['output']and'1 == 0'in r['output']
 else:assert 'compiled 'in r['output']
r=records['reference-verification.json'];assert r['private_mirror_calls']==23 and len(r['supported_byte_agreements'])==9
assert len(r['omitted_impossible_logical_extents'])==2
print('PASS retained25checked pairs,4const pairs,23private mirrors and complete production hashes')
