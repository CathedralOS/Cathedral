#!/usr/bin/env python3
import hashlib,json,subprocess
from pathlib import Path
import check,fixtures,reference
HERE=fixtures.HERE;ROOT=fixtures.ROOT

def verify_build_binding(record,stage):
 assert record['execution_root']==str(ROOT.resolve()),'execution root differs from current repository'
 expected=check.build_text(stage)
 assert record['build_text']==expected,'generated dependency build differs'
 assert record['build_sha256']==check.text_sha(expected),'generated build hash differs'

def main():
 subprocess.run(['python3',str(HERE/'fixtures.py'),'--check'],check=True)
 subprocess.run(['python3',str(HERE/'inventory.py'),'--check'],check=True)
 runtime=json.loads((HERE/'verification.json').read_text());constant=json.loads((HERE/'const-verification.json').read_text());rows=fixtures.cases();mapping={r['name']:r for r in rows}
 verify_build_binding(runtime,'runtime');verify_build_binding(constant,'constant')
 assert runtime['input_sha256']==constant['input_sha256']==check.snapshot()
 assert runtime['positive_count']==runtime['control_count']==len(rows)
 assert runtime['runner_sha256']==check.sha(Path('/tmp/cathedral-acpi-object-description-runner/release/cathedral-acpi-checked-runner'))
 assert runtime['runner_sha256']=='e6d0aee6b4dddbbf34a60cffbe8f158643cc5c4d100f9e20f0481f75f52890be'
 assert runtime['cases']==[r['name']for r in rows];seen=[]
 for batch in runtime['batches']:
  assert batch['execution_root']==runtime['execution_root'] and batch['build_sha256']==runtime['build_sha256']
  text,names=check.source([mapping[n]for n in batch['cases']]);assert batch['source_sha256']==hashlib.sha256(text.encode()).hexdigest();assert batch['runner_sha256']==runtime['runner_sha256'];assert '\nFAIL 'not in batch['output']
  lines=batch['output'].splitlines()
  assert lines.count('CHECKED authored package and dependency bodies; native publication NOT requested')==1
  assert sum(line.startswith('PASS ')for line in lines)==2*len(batch['cases'])
  for n in batch['cases']:
   for c in [False,True]:assert batch['output'].count(f'PASS Suite::{n}_{"control"if c else"positive"} expected={int(c)} observed={int(c)} error=None ')==1
  seen+=batch['cases']
 assert seen==runtime['cases']
 assert constant['cases']==['Uninitialized_63_True','BufferField_31_True','invalid_Device_count_max'];assert len(constant['proofs'])==6
 for proof in constant['proofs']:
  assert proof['execution_root']==constant['execution_root'] and proof['build_sha256']==constant['build_sha256']
  row=mapping[proof['case']];text=fixtures.IMPORTS+fixtures.HELPERS+fixtures.render(row,proof['control'])+'const TEST_RESULT:i32=test_result();\nmachine require_ok(value:i32) requires value==0;{}\ndata Main{}\nmachine Main::main(&mut self){require_ok(TEST_RESULT);}\n'
  assert proof['source_sha256']==hashlib.sha256(text.encode()).hexdigest()
  if proof['control']:assert 'cannot prove requires contract'in proof['output']and'1 == 0'in proof['output']
  else:assert 'compiled 'in proof['output']
 assert constant['omega_revision']==runtime['omega_revision']==check.PIN
 assert constant['compiler_sha256']==check.sha(Path('/tmp/cathedral-omega-eaa7993/release/omega'))
 reference.verify()
 history=HERE/'history/prepublication'
 prior=json.loads((history/'tools/ports/acpi/aml/object-descriptions/manifest.json').read_text())
 for path,digest in prior['files_sha256'].items():assert check.sha(history/path)==digest,'historical artifact changed: '+path
 archived=HERE/'history/root-unbound'
 former=json.loads((archived/'tools/ports/acpi/aml/object-descriptions/manifest.json').read_text())
 for path,digest in former['files_sha256'].items():assert check.sha(archived/path)==digest,'earlier final artifact changed: '+path
 print('PASS',len(rows),'object description behavior/control pairs, 3 constant pairs and 11 static pinned labels; exact closure verified')
if __name__=='__main__':main()
