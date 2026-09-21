#!/usr/bin/env python3
"""Check and execute real Omega string/number bodies and changed-body controls."""
import argparse,hashlib,json,os,shutil,subprocess,tempfile,time
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT;SHARED=HERE.parent/'execution'
def snapshot():
 sources=set((ROOT/'source/libraries/acpi/interpreter').glob('*.omg'))
 sources.update(p for p in HERE.iterdir()if p.suffix in ['.py','.omg','.json','.rs']and'verification'not in p.name)
 sources.update([SHARED/'checked_runner.rs',SHARED/'runner.Cargo.lock'])
 return {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in sorted(sources)}
def main():
 p=argparse.ArgumentParser();p.add_argument('--match',default='');p.add_argument('--record',type=Path);a=p.parse_args();omega=ROOT.parent/'Omega';target=Path('/tmp/cathedral-acpi-execution-checked')
 revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=omega,text=True).strip();assert revision=='eaa7993a23623cd8fabf45350340479c5c9c7879';assert not subprocess.check_output(['git','status','--porcelain'],cwd=omega,text=True).strip()
 subprocess.run(['python3',str(HERE/'fixtures.py'),'--check'],check=True)
 rows=[r for r in fixtures.cases()if any(x in r['name']for x in a.match.split(','))];assert rows
 before=snapshot()
 with tempfile.TemporaryDirectory(prefix='cathedral-acpi-string-numbers-')as directory:
  work=Path(directory);manifest=['[package]','name="cathedral-acpi-checked-runner"','version="0.1.0"','edition="2024"','[dependencies]']
  for name,path in {'checked-interpreter':'psi/semantics/checked-interpreter','package-manager':'omega/packages/manager','target':'omega/representations/target'}.items():manifest.append(f'{name}={{path="{omega}/omega-rust/{path}"}}')
  manifest+=['[[bin]]','name="cathedral-acpi-checked-runner"',f'path="{SHARED}/checked_runner.rs"'];(work/'Cargo.toml').write_text('\n'.join(manifest)+'\n');(work/'Cargo.lock').write_bytes((SHARED/'runner.Cargo.lock').read_bytes())
  subprocess.run([shutil.which('mbx')or'cargo','build','--offline','--locked','--release','--manifest-path',str(work/'Cargo.toml'),'--target-dir',str(target)],cwd=omega,check=True)
  runner=target/'release/cathedral-acpi-checked-runner';source=fixtures.IMPORTS+fixtures.HELPERS+'data Suite {}\n';selections=[]
  for row in rows:
   for control in [False,True]:
    machine='Suite::'+row['name']+('_control'if control else'_positive');source+=fixtures.render(row,control,machine);selections.append(machine+'='+str(int(control)))
  (work/'main.omg').write_text(source);(work/'build.omg').write_text((HERE/'build.omg').read_text().replace('../../../../../source/libraries/acpi/interpreter',str(ROOT/'source/libraries/acpi/interpreter')))
  start=time.monotonic();lines=[];process=subprocess.Popen([str(runner),str(work/'main.omg'),str(work/'build'),*selections],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'))
  for line in process.stdout:print(line,end='',flush=True);lines.append(line)
  if process.wait():raise SystemExit('Omega string/number cases failed')
  assert snapshot()==before,'Source changed during test run'
  if a.record:a.record.write_text(json.dumps({'format':'cathedral-acpi-string-numbers-checked-v1','stage':'checked-interpreter execution; native/hardware not run','omega_revision':revision,'runner_sha256':hashlib.sha256(runner.read_bytes()).hexdigest(),'scenario_count':len(rows),'control_count':len(rows),'cases':[r['name']for r in rows],'evaluator_step_limit':10000000,'elapsed_seconds':round(time.monotonic()-start,3),'source_sha256':before,'output':''.join(lines)},indent=2,sort_keys=True)+'\n')
 print(f'PASS {len(rows)} string/number bodies + {len(rows)} changed-body controls. Native/hardware NOT RUN.')
if __name__=='__main__':main()
