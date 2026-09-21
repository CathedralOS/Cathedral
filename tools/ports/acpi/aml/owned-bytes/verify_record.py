#!/usr/bin/env python3
"""Verify current owned-storage receipts and preserved historical migration evidence."""
import hashlib,json,re,subprocess
from pathlib import Path
import fixtures,extras,check
HERE=fixtures.HERE;ROOT=fixtures.ROOT
PIN='eaa7993a23623cd8fabf45350340479c5c9c7879';RUNNER='e6d0aee6b4dddbbf34a60cffbe8f158643cc5c4d100f9e20f0481f75f52890be'
def read(name):return json.loads((HERE/name).read_text())
def hashes(value):
 for path,digest in value['source_sha256'].items():assert hashlib.sha256((ROOT/path).read_bytes()).hexdigest()==digest,('stale',path)
def pairs(output,names,prefix):
 assert '\nFAIL 'not in output
 for name in names:
  for suffix,number in [('positive',0),('control',1)]:assert output.count(f'PASS {prefix}::{name}_{suffix} expected={number} observed={number} error=None ')==1,(prefix,name,suffix)
actual=read('verification.json');assert actual['cases']==[r['name']for r in fixtures.cases()];assert actual['scenario_count']==actual['control_count']==128;assert actual['source_sha256']==check.snapshot();assert actual['omega_revision']==PIN and actual['runner_sha256']==RUNNER
all_names=[]
for batch in actual['batches']:
 assert batch['exit']==0;pairs(batch['output'],batch['cases'],'Suite');all_names+=batch['cases']
 assert batch['suite_sha256']==hashlib.sha256((fixtures.IMPORTS+fixtures.HELPERS+'data Suite {}\n'+''.join(fixtures.render(next(r for r in fixtures.cases()if r['name']==name),control,'Suite::'+name+('_control'if control else'_positive'))for name in batch['cases']for control in [False,True])).encode()).hexdigest()
assert all_names==actual['cases']
if 'composition'in actual:
 provenance=actual['composition'];raw=read(provenance['original_receipt']);corrected=read(provenance['replacement_receipt']);assert corrected['runner_sha256']==RUNNER and corrected['source_sha256']==actual['source_sha256'];assert corrected['scenario_count']==corrected['control_count']==8
 for key in ['original_receipt','replacement_receipt','incident_receipt']:assert hashlib.sha256((HERE/provenance[key]).read_bytes()).hexdigest()==provenance[key+'_sha256']
 assert corrected['batches'][0]['cases']==raw['batches'][provenance['replaced_batch_index']]['cases'];assert corrected['batches'][0]['suite_sha256']==raw['batches'][provenance['replaced_batch_index']]['suite_sha256']
 assert actual['batches'][provenance['replaced_batch_index']]==corrected['batches'][0]
extra=read('extras-verification.json');hashes(extra);assert extra['scenario_count']==extra['control_count']==10;assert extra['cases']==[r['name']for r in extras.cases()];assert extra['runner_sha256']==RUNNER;pairs(extra['records'][0]['output'],extra['cases'],'Extras')
constant=read('const-verification.json');hashes(constant);assert constant['scenario_count']==constant['control_count']==3;assert len(constant['records'])==6;assert constant['runner_sha256']=='2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4'
for proof in constant['records']:
 if proof['control']:assert 'cannot prove requires contract'in proof['output']and'1 == 0'in proof['output']
 else:assert 'compiled 'in proof['output']and'wrote_output=false'in proof['output']
for name,count,prefix in [('parser',27,'PipelineSuite'),('pipeline',22,'PipelineSuite'),('references',160,'ReferenceSuite'),('execution',79,'Suite'),('fields',18,'FieldSuite')]:
 value=read('regressions/'+name+'.json');hashes(value);assert value['scenario_count']==value['control_count']==count;assert value['omega_revision']==PIN and value['runner_sha256']==RUNNER;pairs(value['output'],[name.replace('-','_')for name in value['cases']],prefix)
const_reference=read('regressions/reference-const.json');hashes(const_reference);assert const_reference['scenario_count']==const_reference['control_count']==5;assert const_reference['runner_sha256']=='2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4'
for proof in const_reference['proofs']:
 if proof['control']:assert 'cannot prove requires contract'in proof['output']and'1 == 0'in proof['output']
 else:assert 'compiled 'in proof['output']
reference=read('reference-verification.json');hashes(reference);assert len(reference['observations'])==21
migration=read('migration.json')
for path,digest in migration['historical_sha256'].items():assert hashlib.sha256((ROOT/path).read_bytes()).hexdigest()==digest,('historical receipt changed',path)
subprocess.run(['python3',str(ROOT/'tools/ports/inventory.py'),'check',str(ROOT/'source/libraries/acpi/aml/byte-storage-inventory.json'),'--checkout',str(ROOT/'reference_code/rust-osdev/acpi')],check=True)
assert json.loads((HERE/'cases.json').read_text())==fixtures.cases()
print('PASS current138 owned-byte/composition pairs,306 regression pairs,8 const pairs,21 labelled Rust observations; old receipts preserved; native/hardware NOT RUN.')
