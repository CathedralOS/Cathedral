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
 assert runtime['runner_sha256']==check.sha(Path('/tmp/cathedral-acpi-object-numeric-string-runner/release/cathedral-acpi-checked-runner'))
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
 assert constant['cases']==['integer_Hexadecimal_32_18446744073709551615','buffer_pattern_Decimal_16_15_True','encoding_Decimal_256_255_True'];assert len(constant['proofs'])==6
 for proof in constant['proofs']:
  row=mapping[proof['case']];text=fixtures.IMPORTS+fixtures.HELPERS+fixtures.render(row,proof['control'])+'const TEST_RESULT:i32=test_result();\nmachine require_ok(value:i32) requires value==0;{}\ndata Main{}\nmachine Main::main(&mut self){require_ok(TEST_RESULT);}\n'
  assert proof['source_sha256']==hashlib.sha256(text.encode()).hexdigest()
  if proof['control']:assert 'cannot prove requires contract'in proof['output']and'1 == 0'in proof['output']
  else:assert 'compiled 'in proof['output']
 assert constant['omega_revision']==runtime['omega_revision']==check.PIN
 assert constant['compiler_sha256']==check.sha(Path('/tmp/cathedral-omega-eaa7993/release/omega'))
 assert host['generator_sha256']==check.sha(HERE/'reference.py')and host['fixtures_sha256']==check.sha(HERE/'fixtures.py')
 assert [r['case']for r in host['rows']]==reference.selections()
 for p,h in host['sources'].items():assert check.sha(Path(p))==h
 for p,h in host['upstream_sha256'].items():assert check.sha(reference.UP/p)==h
 for row in host['rows']:
  r=row['case'];observed=row['public_interpreter'];assert observed['forbidden_calls']=='0'and observed['created_mutexes']=='1'
  if r['kind']=='Buffer'and r['declared']<len(r['data']):assert observed['panic']=='true' and 'result'not in observed;continue
  raw=r['number'];data=r['data']+([0]*max(0,r['declared']-len(r['data']))if r['kind']=='Buffer'else[])
  expected=(str(raw)if r['format']=='Decimal'else f'0x{raw:X}')if r['kind']=='Integer'else(bytes(r['data']).decode('ascii')if r['kind']=='String'else ','.join(str(v)if r['format']=='Decimal'else f'0x{v:02X}'for v in data))
  assert observed['result']=='string:'+expected.encode().hex(),(r['name'],observed,expected)
 history=HERE/'history/prepublication'
 prior=json.loads((history/'tools/ports/acpi/aml/object-numeric-strings/manifest.json').read_text())
 for path,digest in prior['files_sha256'].items():assert check.sha(history/path)==digest,'historical artifact changed: '+path
 print('PASS',len(rows),'numeric-string behavior/control pairs, 3 constant pairs,',len(host['rows']),'actual public opcode observations; exact closure verified')
if __name__=='__main__':main()
