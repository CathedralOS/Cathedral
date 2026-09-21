#!/usr/bin/env python3
"""Execute actual canonical reference kernels and expectation-body controls."""
import argparse,hashlib,json,os,shutil,subprocess,tempfile,time
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT;SHARED=ROOT/'tools/ports/acpi/interpreter/execution'
def snapshot():
 paths=set((ROOT/'source/libraries/acpi/aml').rglob('*.omg'))
 paths.update(p for p in HERE.iterdir()if p.suffix in ['.py','.omg','.rs','.lock','.json']and 'verification'not in p.name)
 paths.update([SHARED/'checked_runner.rs',SHARED/'runner.Cargo.lock'])
 return {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in sorted(paths)}
def main():
 p=argparse.ArgumentParser();p.add_argument('--match',default='');p.add_argument('--record',type=Path);p.add_argument('--const',dest='constant',action='store_true');a=p.parse_args()
 subprocess.run(['python3',str(HERE/'fixtures.py'),'--check'],check=True)
 omega=ROOT.parent/'Omega';revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=omega,text=True).strip();assert revision=='eaa7993a23623cd8fabf45350340479c5c9c7879';assert not subprocess.check_output(['git','status','--porcelain'],cwd=omega,text=True).strip()
 rows=[r for r in fixtures.cases()if any(m in r['name']for m in a.match.split(','))];assert rows;before=snapshot();started=time.monotonic();lines=[];proofs=[]
 with tempfile.TemporaryDirectory(prefix='cathedral-object-references-')as directory:
  work=Path(directory);(work/'build.omg').write_text((HERE/'build.omg').read_text().replace('../../../../../source/',str(ROOT/'source')+'/'))
  if a.constant:
   runner=Path('/tmp/cathedral-omega-eaa7993/release/omega')
   for row in rows:
    for control in [False,True]:
     text=fixtures.IMPORTS+fixtures.HELPERS+fixtures.render(row,control)+'''const TEST_RESULT:i32=test_result();
machine require_ok(value:i32) requires value==0;{}
data Main{}
machine Main::main(&mut self){require_ok(TEST_RESULT);}
''';(work/'main.omg').write_text(text)
     result=subprocess.run([str(runner),'--check',str(work/'main.omg')],capture_output=True,text=True);output=result.stdout+result.stderr
     if control:assert result.returncode and 'cannot prove requires contract'in output and '1 == 0'in output,output
     else:assert result.returncode==0,output
     proofs.append(dict(case=row['name'],control=control,fixture_sha256=hashlib.sha256(text.encode()).hexdigest(),output=output));line=f'PASS const {row["name"]} control={control}\n';lines.append(line);print(line,end='',flush=True)
  else:
   manifest=['[package]','name="cathedral-acpi-checked-runner"','version="0.1.0"','edition="2024"','[dependencies]']
   for name,path in {'checked-interpreter':'psi/semantics/checked-interpreter','package-manager':'omega/packages/manager','target':'omega/representations/target'}.items():manifest.append(f'{name}={{path="{omega}/omega-rust/{path}"}}')
   manifest+=['[[bin]]','name="cathedral-acpi-checked-runner"',f'path="{SHARED}/checked_runner.rs"'];(work/'Cargo.toml').write_text('\n'.join(manifest)+'\n');(work/'Cargo.lock').write_bytes((SHARED/'runner.Cargo.lock').read_bytes());target=Path('/tmp/cathedral-acpi-execution-checked')
   subprocess.run([shutil.which('mbx')or'cargo','build','--offline','--locked','--release','--manifest-path',str(work/'Cargo.toml'),'--target-dir',str(target)],cwd=omega,check=True)
   source=fixtures.IMPORTS+fixtures.HELPERS+'data ReferenceSuite{}\n';selections=[]
   for row in rows:
    for control in [False,True]:
     machine='ReferenceSuite::'+row['name']+('_control'if control else'_positive');source+=fixtures.render(row,control,machine);selections.append(machine+'='+str(int(control)))
   (work/'main.omg').write_text(source);runner=target/'release/cathedral-acpi-checked-runner';process=subprocess.Popen([str(runner),str(work/'main.omg'),str(work/'build'),*selections],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'))
   for line in process.stdout:print(line,end='',flush=True);lines.append(line)
   if process.wait():raise SystemExit('Object reference semantics failed')
 assert snapshot()==before,'Sources changed during test run'
 if a.record:a.record.write_text(json.dumps(dict(format='cathedral-object-references-const-v1'if a.constant else'cathedral-object-references-checked-v1',stage='Omega constant evaluation with requires checking'if a.constant else'checked-interpreter execution; native/hardware not run',omega_revision=revision,runner_sha256=hashlib.sha256(runner.read_bytes()).hexdigest(),cases=[r['name']for r in rows],scenario_count=len(rows),control_count=len(rows),evaluator_step_limit=None if a.constant else 10000000,source_sha256=before,proofs=proofs,output=''.join(lines),elapsed_seconds=round(time.monotonic()-started,3)),indent=2,sort_keys=True)+'\n')
 print('PASS',len(rows),'object reference scenarios +',len(rows),'body controls; native/hardware not run')
if __name__=='__main__':main()
