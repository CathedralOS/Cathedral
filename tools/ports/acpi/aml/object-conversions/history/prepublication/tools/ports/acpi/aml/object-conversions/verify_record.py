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
 assert constant['cases']==['integer_32_1311768467139281697','owned_dirty_tail_Buffer_buffer','invalid_id_max_integer'];assert len(constant['proofs'])==6
 for proof in constant['proofs']:
  row=mapping[proof['case']];text=fixtures.IMPORTS+fixtures.HELPERS+fixtures.render(row,proof['control'])+'const TEST_RESULT:i32=test_result();\nmachine require_ok(value:i32) requires value==0;{}\ndata Main{}\nmachine Main::main(&mut self){require_ok(TEST_RESULT);}\n'
  assert proof['source_sha256']==hashlib.sha256(text.encode()).hexdigest()
  if proof['control']:assert 'cannot prove requires contract'in proof['output']and'1 == 0'in proof['output']
  else:assert 'compiled 'in proof['output']
 assert constant['omega_revision']==runtime['omega_revision']==check.PIN
 assert constant['compiler_sha256']==check.sha(Path('/tmp/cathedral-omega-eaa7993/release/omega'))
 assert host['generator_sha256']==check.sha(HERE/'reference.py')and host['fixtures_sha256']==check.sha(HERE/'fixtures.py')
 assert len(host['rows'])==100 and sum('explicit_interpreter'in r for r in host['rows'])==66
 for p,h in host['sources'].items():assert check.sha(Path(p))==h
 for p,h in host['upstream_sha256'].items():assert check.sha(reference.UP/p)==h
 baseline=json.loads((ROOT/'canonical-baseline.json').read_text())
 for path,digest in baseline.items():assert check.sha(ROOT/path)==digest,'copied canonical source changed: '+path
 print('PASS',len(rows),'conversion preflight pairs,3constant pairs,',len(host['rows']),'Object observations plus66 explicit observations; copied canonical baseline unchanged')
if __name__=='__main__':main()
