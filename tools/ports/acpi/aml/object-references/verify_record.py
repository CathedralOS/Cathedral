#!/usr/bin/env python3
"""Retained actual reference-kernel execution, const controls and source identity."""
import hashlib,json
import fixtures,check
HERE=fixtures.HERE;names=[r['name']for r in fixtures.cases()];current=check.snapshot()
for filename,expected,kind in [('verification.json',names,'cathedral-object-references-checked-v1'),('const-verification.json',['rust_single_0_all','zero_budget','self_cycle','package_bad_tail_after_selection','copy_method'],'cathedral-object-references-const-v1')]:
 value=json.loads((HERE/filename).read_text());assert value['format']==kind;assert value['cases']==expected;assert value['scenario_count']==value['control_count']==len(expected);assert value['source_sha256']==current,'Stale object-reference source/tool evidence'
 assert value['omega_revision']=='eaa7993a23623cd8fabf45350340479c5c9c7879'
 if 'const'in filename:
  assert len(value['proofs'])==len(expected)*2
  for proof in value['proofs']:
   if proof['control']:assert 'cannot prove requires contract'in proof['output'] and '1 == 0'in proof['output']
   else:assert 'compiled 'in proof['output'] and 'wrote_output=false'in proof['output']
 else:
  for name in expected:
   for suffix,number in [('positive',0),('control',1)]:assert value['output'].count(f'PASS ReferenceSuite::{name}_{suffix} expected={number} observed={number} error=None ')==1
  assert '\nFAIL 'not in value['output']
reference=json.loads((HERE/'reference-verification.json').read_text());assert len(reference['observations'])==116
for name,digest in reference['source_sha256'].items():assert hashlib.sha256((HERE/name).read_bytes()).hexdigest()==digest
print('PASS current160 checked pairs,5 const pairs and116 actual immutable public Rust observations; native/hardware not run')
