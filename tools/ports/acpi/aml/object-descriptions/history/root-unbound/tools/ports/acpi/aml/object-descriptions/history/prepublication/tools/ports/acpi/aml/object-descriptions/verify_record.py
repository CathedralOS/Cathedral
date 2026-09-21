#!/usr/bin/env python3
import hashlib,json,subprocess
from pathlib import Path
import check,fixtures,reference
HERE=fixtures.HERE;ROOT=fixtures.ROOT

def main():
 subprocess.run(['python3',str(HERE/'fixtures.py'),'--check'],check=True)
 subprocess.run(['python3',str(HERE/'inventory.py'),'--check'],check=True)
 runtime=json.loads((HERE/'verification.json').read_text());constant=json.loads((HERE/'const-verification.json').read_text());rows=fixtures.cases();mapping={r['name']:r for r in rows}
 assert runtime['input_sha256']==constant['input_sha256']==check.snapshot()
 assert runtime['positive_count']==runtime['control_count']==len(rows)
 assert runtime['runner_sha256']==check.sha(Path('/tmp/cathedral-acpi-object-description-runner/release/cathedral-acpi-checked-runner'))
 assert runtime['runner_sha256']=='e6d0aee6b4dddbbf34a60cffbe8f158643cc5c4d100f9e20f0481f75f52890be'
 assert runtime['cases']==[r['name']for r in rows];seen=[]
 for batch in runtime['batches']:
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
  row=mapping[proof['case']];text=fixtures.IMPORTS+fixtures.HELPERS+fixtures.render(row,proof['control'])+'const TEST_RESULT:i32=test_result();\nmachine require_ok(value:i32) requires value==0;{}\ndata Main{}\nmachine Main::main(&mut self){require_ok(TEST_RESULT);}\n'
  assert proof['source_sha256']==hashlib.sha256(text.encode()).hexdigest()
  if proof['control']:assert 'cannot prove requires contract'in proof['output']and'1 == 0'in proof['output']
  else:assert 'compiled 'in proof['output']
 assert constant['omega_revision']==runtime['omega_revision']==check.PIN
 assert constant['compiler_sha256']==check.sha(Path('/tmp/cathedral-omega-eaa7993/release/omega'))
 reference.verify()
 baseline=json.loads((ROOT/'canonical-baseline.json').read_text())
 for path in check.SOURCE:
  if not path.endswith('/object_descriptions.omg'):assert check.sha(ROOT/path)==baseline[path]
 print('PASS',len(rows),'object description behavior/control pairs, 3 constant pairs and 11 static pinned labels; exact closure verified')
if __name__=='__main__':main()
