#!/usr/bin/env python3
"""Check actual Omega sources once, then execute each case and changed-body control."""
import argparse, hashlib, json, os, subprocess, tempfile, time
from pathlib import Path
import fixtures
HERE=fixtures.HERE

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--omega-source',type=Path,default=fixtures.ROOT.parent/'Omega');p.add_argument('--runner',type=Path);p.add_argument('--match',default='');p.add_argument('--record',type=Path);p.add_argument('--target-dir',type=Path,default=Path('/tmp/cathedral-acpi-generic-checked'));a=p.parse_args();omega=a.omega_source.resolve();a.target_dir=a.target_dir.resolve()
 revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=omega,text=True).strip();print('Omega source:',revision,flush=True)
 if revision!='eaa7993a23623cd8fabf45350340479c5c9c7879':raise SystemExit('Runner requires the audited Omega pin')
 if subprocess.check_output(['git','status','--porcelain'],cwd=omega,text=True).strip():raise SystemExit('Omega source must be clean')
 subprocess.run(['python3',str(HERE/'fixtures.py'),'--check'],check=True)
 rows=[r for r in fixtures.cases()if any(s in r['name']for s in a.match.split(','))]
 if not rows:raise SystemExit('No matching cases')
 with tempfile.TemporaryDirectory(prefix='cathedral-acpi-checked-')as directory:
  root=Path(directory);manifest=['[package]','name="cathedral-acpi-checked-runner"','version="0.1.0"','edition="2024"','[dependencies]']
  for name,path in {'checked-interpreter':'psi/semantics/checked-interpreter','package-manager':'omega/packages/manager','target':'omega/representations/target'}.items():manifest.append(f'{name} = {{ path = "{omega}/omega-rust/{path}" }}')
  manifest +=['[[bin]]','name="cathedral-acpi-checked-runner"',f'path="{HERE}/checked_runner.rs"']
  (root/'Cargo.toml').write_text('\n'.join(manifest)+'\n')
  (root/'Cargo.lock').write_bytes((HERE/'runner.Cargo.lock').read_bytes())
  if a.runner is None:subprocess.run(['cargo','build','--offline','--locked','--release','--manifest-path',str(root/'Cargo.toml'),'--target-dir',str(a.target_dir)],check=True,cwd=omega)
  runner=a.runner.resolve() if a.runner else a.target_dir/'release/cathedral-acpi-checked-runner';runner_before=hashlib.sha256(runner.read_bytes()).hexdigest();print('Harness SHA-256:',hashlib.sha256(runner.read_bytes()).hexdigest(),flush=True)
  build=(HERE/'build.omg').read_text()
  for relative in ['../../../../../source/libraries/acpi/interpreter/execution','../../../../../source/libraries/acpi/interpreter','../../../../../source/libraries/acpi/aml']:build=build.replace(relative,str((HERE/relative).resolve()))
  (root/'build.omg').write_text(build)
  imports=[];bodies=[];selections=[]
  for row in rows:
   for negative in [False,True]:
    source=fixtures.render(row,False)
    if negative:
     old=(f'result.value.number == {row["expected"]}'if row['expected']is not None and row['error']=='Success'else f'result.outcome == ExecutionOutcome::{row["error"]}')
     new=(f'result.value.number == {(row["expected"]+1)&((1<<row["bits"])-1)}'if row['expected']is not None and row['error']=='Success'else f'result.outcome == ExecutionOutcome::{"BadEncoding"if row["error"]=="Success"else"Success"}')
     assert source.count(old)==1;source=source.replace(old,new)
    prefix=source[:source.index('machine test_result()')]
    if not imports:imports=[prefix,'data Suite {}\n']
    machine='Suite::'+row['name']+('_control'if negative else'_positive')
    body=source[source.index('machine test_result()'):source.index('data Main {}')].replace('machine test_result()','machine '+machine+'(&mut self)',1)
    bodies.append(body);selections.append(machine+'='+str(int(negative)))
  (root/'main.omg').write_text(''.join(imports+bodies))
  env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000')
  paths=set()
  for package in ['source/libraries/acpi/aml','source/libraries/acpi/interpreter']:
   paths.update((fixtures.ROOT/package).rglob('*.omg'))
  paths.update(path for path in HERE.iterdir()if path.suffix in ['.py','.rs','.omg','.json']and path.name!='verification.json')
  before={str(path.relative_to(fixtures.ROOT)):hashlib.sha256(path.read_bytes()).hexdigest()for path in sorted(paths)}
  start=time.monotonic();command=[str(runner),str(root/'main.omg'),str(root/'build'),*selections]
  process=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=env);lines=[]
  for line in process.stdout:print(line,end='',flush=True);lines.append(line)
  completed=subprocess.CompletedProcess(command,process.wait(),''.join(lines));completed.check_returncode()
  assert before=={str(path.relative_to(fixtures.ROOT)):hashlib.sha256(path.read_bytes()).hexdigest()for path in sorted(paths)},'Source changed during regression'
  assert runner_before==hashlib.sha256(runner.read_bytes()).hexdigest(),'Runner changed during regression'
  if a.record:
   record={'execution_root':str(fixtures.ROOT.resolve()),'build_source':build,'build_sha256':hashlib.sha256(build.encode()).hexdigest(),'fixture_sha256':hashlib.sha256((root/'main.omg').read_bytes()).hexdigest(),'command':command,'source_unchanged':True,'format':'cathedral-aml-execution-checked-v1','stage':'checked-interpreter execution; native/hardware not run','omega_revision':revision,'rustc':subprocess.check_output(['rustc','--version'],cwd=omega,text=True).strip(),'runner_sha256':hashlib.sha256(runner.read_bytes()).hexdigest(),'cargo_lock_sha256':hashlib.sha256((root/'Cargo.lock').read_bytes()).hexdigest(),'evaluator_step_limit':10000000,'scenario_count':len(rows),'control_count':len(rows),'cases':[row['name']for row in rows],'elapsed_seconds':round(time.monotonic()-start,3),'source_sha256':before,'output':completed.stdout}
   a.record.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 print(f'PASS {len(rows)} checked-interpreter scenarios + {len(rows)} changed-body controls. Native/hardware NOT RUN.',flush=True)
if __name__=='__main__':main()
