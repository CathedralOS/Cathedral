#!/usr/bin/env python3
"""Verify retained evidence against exact current sources, probes and fixtures."""
import hashlib,json
import check,check_const,generate
HERE=generate.HERE
checked=json.loads((HERE/'verification.json').read_text());constant=json.loads((HERE/'const-verification.json').read_text());snapshot=check.snapshot()
assert checked['source_sha256']==snapshot and constant['source_sha256']==snapshot,'Stale source evidence'
assert checked['reference']==check.reference_provenance(),'Pinned reference drift'
assert checked['groups']==[g['name']for g in generate.groups()]
assert checked['case_count']==len(generate.cases()) and checked['group_count']==checked['control_count']==len(generate.groups())
for name in checked['groups']:
 assert f'PASS Suite::{name}_positive expected=0 observed=0 error=None' in checked['output']
 assert f'PASS Suite::{name}_control expected=1 observed=1 error=None' in checked['output']
assert [entry['case']for entry in constant['cases']]==check_const.chosen()
assert constant['compiler_sha256']==hashlib.sha256(check_const.COMPILER.read_bytes()).hexdigest()
for entry in constant['cases']:
 assert len(entry['observations'])==2
 for observation in entry['observations']:
  if observation['control']:assert 'cannot prove requires contract'in observation['output'] and '1 == 0'in observation['output']
  else:assert 'compiled 'in observation['output'] and 'wrote_output=false'in observation['output'],observation['output']
print('PASS current encrypted frame checked/const evidence; no native or hardware execution claimed')
