#!/usr/bin/env python3
"""Verify actual execution/reference evidence, then optionally write release hashes."""
import argparse,hashlib,json,subprocess
from pathlib import Path
import fixtures
from check import snapshot
HERE=fixtures.HERE;ROOT=fixtures.ROOT
h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def text(r):return json.dumps(r,indent=2,sort_keys=True)+'\n'
def main():
 p=argparse.ArgumentParser();p.add_argument('--write',action='store_true');a=p.parse_args()
 subprocess.run(['python3',str(HERE/'fixtures.py'),'--check'],check=True)
 subprocess.run(['python3',str(HERE/'evidence.py'),'--check'],check=True)
 runtime=json.loads((HERE/'checked-verification.json').read_text());const=json.loads((HERE/'const-verification.json').read_text());reference=json.loads((HERE/'reference-verification.json').read_text());cases=fixtures.cases();names=[r['name']for r in cases]
 assert runtime['source_sha256']==const['source_sha256']==snapshot()
 assert runtime['scenario_count']==runtime['control_count']==len(names)and runtime['cases']==names
 suite=fixtures.IMPORTS+fixtures.HELPERS+'data Suite {}\n'
 for row in cases:
  for control in [False,True]:
   name='Suite::'+row['name']+('_control'if control else'_positive');suite+=fixtures.render(row,control,name)
   assert f'PASS {name} expected={int(control)} observed={int(control)} error=None'in runtime['output']
 assert hashlib.sha256(suite.encode()).hexdigest()==runtime['suite_sha256']
 assert len(const['runs'])==2*len(const['cases'])
 assert {(r['case'],r['control'])for r in const['runs']}=={(n,b)for n in const['cases']for b in [False,True]}
 for r in const['runs']:
  if r['control']:assert 'cannot prove requires contract'in r['output']and'1 == 0'in r['output']
  else:assert 'compiled 'in r['output']
 for path,sha in reference['source_sha256'].items():assert h(ROOT/path)==sha,path
 for path,sha in reference['upstream_sha256'].items():assert h(ROOT/'reference_code/rust-osdev/acpi'/path)==sha,path
 assert set(reference['observed'])|{r['case']for r in reference['omitted']}==set(names)
 assert reference['mirror_observations']==len(reference['observed'])
 source={p:v for p,v in snapshot().items()if p.startswith('source/')};closure=hashlib.sha256()
 for path in sorted(source):closure.update(path.encode());closure.update(b'\0');closure.update((ROOT/path).read_bytes())
 artifacts=[ROOT/'source/libraries/acpi/interpreter'/n for n in ['byte_concat.omg','byte_concat.PORT.md','byte-concat-inventory.json']]+[p for p in HERE.iterdir()if p.is_file()and p.name!='verification.json']
 manifest={'format':'cathedral-aml-byte-concat-v1','status':'tested bounded same-type concatenation helpers; whole do_concat remains pending','upstream_revision':fixtures.PIN,'omega_revision':runtime['omega_revision'],'compiler_sha256':const['compiler_sha256'],'checked_runner_sha256':runtime['runner_sha256'],'source_closure_sha256':closure.hexdigest(),'source_hash_algorithm':'sorted repository-relative path + NUL + file bytes','source_sha256':source,'validation':{'checked_interpreter_pairs':len(names),'constant_pairs':len(const['cases']),'private_expression_mirror_observations':reference['mirror_observations'],'actual_public_conversion_access_calls':reference['actual_public_calls'],'unrepresentable_rust_inputs':len(reference['omitted'])},'boundary':['already-converted same-type values only','no generic Object, Store, opcode, namespace or hardware access','bounded initialized 256-byte arrays; complete preflight and unchanged output on error','no native ABI claim'],'artifacts_sha256':{str(p.relative_to(ROOT)):h(p)for p in sorted(artifacts)}}
 path=HERE/'verification.json'
 if a.write:path.write_text(text(manifest))
 else:assert path.read_text()==text(manifest),'stale release manifest'
 print(f'PASS {len(names)} interpreter pairs, {len(const["cases"])} constant pairs, reference and final hashes.')
if __name__=='__main__':main()
