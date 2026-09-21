#!/usr/bin/env python3
import hashlib,json,subprocess
from pathlib import Path
import check,fixtures,reference
HERE=fixtures.HERE;ROOT=fixtures.ROOT

def main():
 subprocess.run(['python3',str(HERE/'fixtures.py'),'--check'],check=True)
 subprocess.run(['python3',str(HERE/'inventory.py'),'--check'],check=True)
 runtime=json.loads((HERE/'verification.json').read_text());constant=json.loads((HERE/'const-verification.json').read_text());host=json.loads((HERE/'reference-verification.json').read_text());rows=fixtures.cases();mapping={r['name']:r for r in rows}
 assert runtime['input_sha256']==constant['input_sha256']==check.snapshot()
 assert runtime['positive_count']==runtime['control_count']==len(rows)
 assert runtime['cases']==[r['name']for r in rows];seen=[]
 for batch in runtime['batches']:
  text,names=check.source([mapping[n]for n in batch['cases']]);assert batch['source_sha256']==hashlib.sha256(text.encode()).hexdigest();assert batch['runner_sha256']==runtime['runner_sha256'];assert '\nFAIL 'not in batch['output']
  for n in batch['cases']:
   for c in [False,True]:assert batch['output'].count(f'PASS Suite::{n}_{"control"if c else"positive"} expected={int(c)} observed={int(c)} error=None ')==1
  seen+=batch['cases']
 assert seen==runtime['cases']
 assert constant['cases']==['string_1','buffer_width32_truncate','invalid_offset_max'];assert len(constant['proofs'])==6
 for proof in constant['proofs']:
  row=mapping[proof['case']];text=fixtures.IMPORTS+fixtures.HELPERS+fixtures.render(row,proof['control'])+'const TEST_RESULT:i32=test_result();\nmachine require_ok(value:i32) requires value==0;{}\ndata Main{}\nmachine Main::main(&mut self){require_ok(TEST_RESULT);}\n'
  assert proof['source_sha256']==hashlib.sha256(text.encode()).hexdigest()
  if proof['control']:assert 'cannot prove requires contract'in proof['output']and'1 == 0'in proof['output']
  else:assert 'compiled 'in proof['output']
 assert constant['omega_revision']==runtime['omega_revision']==check.PIN
 assert constant['compiler_sha256']==check.sha(Path('/tmp/cathedral-omega-eaa7993/release/omega'))
 assert host['generator_sha256']==check.sha(HERE/'reference.py')and host['fixtures_sha256']==check.sha(HERE/'fixtures.py')
 assert [dict((k,v)for k,v in r.items()if k not in ['observed','agreement'])for r in host['rows']]==reference.observations()
 for p,h in host['sources'].items():assert check.sha(Path(p))==h
 for p,h in host['upstream_sha256'].items():assert check.sha(reference.UP/p)==h
 history=HERE/'history/scratch';baseline=json.loads((history/'tools/ports/acpi/interpreter/byte-literals/manifest.json').read_text())
 for path,digest in baseline['files_sha256'].items():assert check.sha(history/path)==digest,'historical artifact drift: '+path
 for path in ['source/libraries/acpi/aml/names.omg','source/libraries/acpi/aml/namespace.omg']:
  old=json.loads((history/'canonical-baseline.json').read_text());assert check.sha(history/path)==old[path]
 print('PASS',len(rows),'literal preflight pairs,3constant pairs,',len(host['rows']),'public observations; current dependency hashes and historical artifacts verified')
if __name__=='__main__':main()
