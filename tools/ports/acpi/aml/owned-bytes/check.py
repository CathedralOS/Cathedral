#!/usr/bin/env python3
"""Execute selected real storage bodies with full-store changed-body controls."""
import argparse,json,hashlib,os,subprocess,time,tempfile,shutil
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT
SHARED=ROOT/'tools/ports/acpi/interpreter/execution'
SOURCE_FILES=['source/libraries/acpi/aml/'+n for n in ['build.omg','model.omg','byte_storage.omg','object_references.omg','namespace.omg','names.omg','bytes.omg']]+['source/libraries/acpi/interpreter/'+n for n in ['build.omg','integers.omg','conversions.omg','buffer_fields.omg']]
def snapshot():
 paths=[ROOT/name for name in SOURCE_FILES]+[HERE/name for name in ['fixtures.py','cases.json','check.py']]+[SHARED/'checked_runner.rs',SHARED/'runner.Cargo.lock']
 return {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in paths}
def prepare_runner():
 omega=ROOT.parent/'Omega';revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=omega,text=True).strip();assert revision=='eaa7993a23623cd8fabf45350340479c5c9c7879';assert not subprocess.check_output(['git','status','--porcelain'],cwd=omega,text=True).strip()
 with tempfile.TemporaryDirectory(prefix='cathedral-owned-byte-runner-')as directory:
  work=Path(directory);manifest=['[package]','name="cathedral-acpi-checked-runner"','version="0.1.0"','edition="2024"','[dependencies]']
  for name,path in {'checked-interpreter':'psi/semantics/checked-interpreter','package-manager':'omega/packages/manager','target':'omega/representations/target'}.items():manifest.append(f'{name}={{path="{omega}/omega-rust/{path}"}}')
  manifest+=['[[bin]]','name="cathedral-acpi-checked-runner"',f'path="{SHARED}/checked_runner.rs"'];(work/'Cargo.toml').write_text('\n'.join(manifest)+'\n');(work/'Cargo.lock').write_bytes((SHARED/'runner.Cargo.lock').read_bytes())
  subprocess.run([shutil.which('mbx')or'cargo','build','--offline','--locked','--release','--manifest-path',str(work/'Cargo.toml'),'--target-dir','/tmp/cathedral-acpi-execution-checked'],cwd=omega,check=True)
 return revision
RUNNER=Path('/tmp/cathedral-acpi-execution-checked/release/cathedral-acpi-checked-runner')
def main():
 p=argparse.ArgumentParser();p.add_argument('--match',default='');p.add_argument('--batch',type=int,default=8);p.add_argument('--record',type=Path);a=p.parse_args();rows=[r for r in fixtures.cases()if any(s in r['name']for s in a.match.split(','))];assert rows
 revision=prepare_runner();sources=snapshot();results=[]
 work=Path('/tmp/cathedral-aml-owned-byte-canonical-runs');work.mkdir(exist_ok=True)
 for i in range(0,len(rows),a.batch):
  batch=rows[i:i+a.batch];directory=work/('batch-'+hashlib.sha256(','.join(r['name']for r in batch).encode()).hexdigest()[:12]);directory.mkdir(exist_ok=True)
  text=fixtures.IMPORTS+fixtures.HELPERS+'data Suite {}\n';selections=[]
  for r in batch:
   for control in [False,True]:
    name='Suite::'+r['name']+('_control'if control else'_positive');text+=fixtures.render(r,control,name);selections.append(name+'='+str(int(control)))
  (directory/'main.omg').write_text(text);(directory/'build.omg').write_text('machine build(builder:&mut Build){builder.application("cathedral-aml-owned-bytes-tests");builder.freestanding=true;builder.depend_as("aml",Source::Path {location:"'+str(ROOT/'source/libraries/acpi/aml')+'"});builder.depend_as("helpers",Source::Path {location:"'+str(ROOT/'source/libraries/acpi/interpreter')+'"});}\n')
  start=time.monotonic();process=subprocess.Popen([str(RUNNER),str(directory/'main.omg'),str(directory/'build'),*selections],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'));lines=[]
  for line in process.stdout:print(line,end='',flush=True);lines.append(line)
  result=dict(cases=[r['name']for r in batch],suite_sha256=hashlib.sha256(text.encode()).hexdigest(),output=''.join(lines),seconds=round(time.monotonic()-start,3),exit=process.wait());results.append(result)
  (directory/'result.json').write_text(json.dumps(result,indent=2)+'\n')
  if result['exit']:raise SystemExit('FAIL batch '+str(directory))
  assert sources==snapshot(),'source or harness changed during run'
 if a.record:a.record.write_text(json.dumps(dict(stage='checked-interpreter actual bodies and body-mutating controls; native/hardware NOT RUN',scenario_count=len(rows),control_count=len(rows),cases=[r['name']for r in rows],omega_revision=revision,runner_sha256=hashlib.sha256(RUNNER.read_bytes()).hexdigest(),source_sha256=sources,batches=results),indent=2)+'\n')
 print('PASS',len(rows),'owned storage cases + controls')
if __name__=='__main__':main()
