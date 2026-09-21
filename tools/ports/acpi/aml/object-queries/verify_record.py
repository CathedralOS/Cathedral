#!/usr/bin/env python3
"""Check query receipts against current source, fixtures, controls and host data."""
import hashlib,json,subprocess
from pathlib import Path
import fixtures,check
RUNNER="e6d0aee6b4dddbbf34a60cffbe8f158643cc5c4d100f9e20f0481f75f52890be"
COMPILER="2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4"
HERE=fixtures.HERE;ROOT=fixtures.ROOT

def main():
 subprocess.run(['python3',str(HERE/'fixtures.py'),'--check'],check=True)
 runtime=json.loads((HERE/'verification.json').read_text());constant=json.loads((HERE/'const-verification.json').read_text());reference=json.loads((HERE/'reference-verification.json').read_text());rows=fixtures.cases()
 assert runtime['input_sha256']==constant['input_sha256']==check.snapshot()
 assert runtime['positive_count']==runtime['control_count']==len(rows)
 assert runtime['cases']==[r['name']for r in rows];names=[];by_name={r['name']:r for r in rows}
 for batch in runtime['batches']:
  text,selections=check.source([by_name[name]for name in batch['cases']]);assert batch['source_sha256']==hashlib.sha256(text.encode()).hexdigest();assert batch['runner_sha256']==runtime['runner_sha256'];assert '\nFAIL 'not in batch['output']
  for name in batch['cases']:
   for control in [False,True]:
    machine='Suite::'+name+('_control'if control else'_positive');assert batch['output'].count(f'PASS {machine} expected={int(control)} observed={int(control)} error=None ')==1
  names+=batch['cases']
 assert names==runtime['cases'];assert len(constant['proofs'])==2*len(constant['cases'])
 assert {(p['case'],p['control'])for p in constant['proofs']}=={(n,c)for n in constant['cases']for c in [False,True]}
 assert constant['compiler_sha256']==COMPILER
 assert constant['cases']==['resolve_mixed_cycle','source_size_Buffer_4','namespace_bind_maximum']
 for proof in constant['proofs']:
  text=fixtures.IMPORTS+fixtures.HELPERS+fixtures.render(by_name[proof['case']],proof['control'])+"const TEST_RESULT:i32=test_result();\nmachine require_ok(value:i32) requires value==0;{}\ndata Main{}\nmachine Main::main(&mut self){require_ok(TEST_RESULT);}\n"
  assert proof['source_sha256']==hashlib.sha256(text.encode()).hexdigest()
  if proof['control']:assert 'cannot prove requires contract'in proof['output']and'1 == 0'in proof['output']
  else:assert 'compiled 'in proof['output']
 assert runtime['omega_revision']==constant['omega_revision']==check.PIN
 assert reference['generator_sha256']==check.sha(HERE/'reference.py')
 for path,digest in reference['source_sha256'].items():assert check.sha(ROOT/path)==digest,path
 for path,digest in reference['upstream_sha256'].items():assert check.sha(ROOT/'reference_code/rust-osdev/acpi'/path)==digest,path
 assert len(reference['rows'])==44 and reference['agreements']==41
 for row in reference['rows'].values():assert row['observed']['forbidden_calls']=='0'and row['observed']['created_mutexes']=='1'
 for name,count,prefix in [('parser',27,'PipelineSuite'),('pipeline',22,'PipelineSuite'),('references',160,'ReferenceSuite'),('execution',79,'Suite'),('fields',18,'FieldSuite')]:
  value=json.loads((HERE/'regressions'/f'{name}.json').read_text())
  for path,digest in value['source_sha256'].items():assert check.sha(ROOT/path)==digest,path
  assert value['scenario_count']==value['control_count']==count and value['omega_revision']==check.PIN and value['runner_sha256']==RUNNER
  for case in value['cases']:
   for suffix,number in [('positive',0),('control',1)]:assert value['output'].count(f"PASS {prefix}::{case.replace('-','_')}_{suffix} expected={number} observed={number} error=None ")==1
 print('PASS',len(rows),'query behavior/control pairs,306 regression pairs,3const pairs,44public observations; current hashes verified')
if __name__=='__main__':main()
