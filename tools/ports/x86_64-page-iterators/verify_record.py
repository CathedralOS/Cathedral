#!/usr/bin/env python3
"""Read-only validation of retained source-bound interpreter and constant evidence."""
import hashlib,json,re,subprocess,sys
import check,check_const,generate

def digest(text):return hashlib.sha256(text.encode()).hexdigest()
def main():
 for name in ['generate_inputs.py','generate.py','generate_inventory.py']:
  subprocess.run([sys.executable,str(generate.HERE/name),'--check'],check=True,cwd=generate.ROOT)
 current=check.snapshot();groups=generate.groups()
 checked=json.loads((generate.HERE/'checked-verification.json').read_text())
 constant=json.loads((generate.HERE/'constant-verification.json').read_text())
 for record in [checked,constant]:
  assert record['source_sha256']==current,'recorded source/harness closure differs from current files'
  assert record['omega_revision']=='eaa7993a23623cd8fabf45350340479c5c9c7879'
 source=generate.HEAD+'data Suite {}\n';selections=[]
 for group in groups:
  for control in [False,True]:
   source+=generate.render(group,control)
   name='Suite::'+group['name']+('_control'if control else'_positive');expected=int(control)
   selections.append(name+'='+str(expected))
   assert checked['output'].count(f'PASS {name} expected={expected} observed={expected} error=None ')==1
 assert checked['suite_sha256']==[digest(source)]
 assert checked['selections']==selections
 assert checked['case_count']==sum(len(g['rows'])for g in groups)==1630
 assert checked['group_count']==checked['control_count']==len(groups)==51
 assert len(re.findall(r'^PASS ',checked['output'],re.M))==102
 assert len(re.findall(r'filesystem_operation_attempts: 0',checked['output']))==102
 assert checked['binary_sha256']==hashlib.sha256(check.RUNNER.read_bytes()).hexdigest()==check.RUNNER_SHA
 assert constant['suite_sha256']==[digest(check_const.source(c))for c in [False,True]]
 assert constant['case_count']==constant['group_count']==constant['control_count']==1
 assert constant['observation_id']==352
 assert 'positive\n' in constant['output'] and 'body_control\n' in constant['output']
 assert 'cannot prove requires contract' in constant['output'] and '1 == 0' in constant['output']
 assert constant['binary_sha256']==hashlib.sha256(check.OMEGA.read_bytes()).hexdigest()
 print('PASS current',len(current),'input hashes, 1630 checked rows/51 controls, and direct constant/control pair')
if __name__=='__main__':main()
