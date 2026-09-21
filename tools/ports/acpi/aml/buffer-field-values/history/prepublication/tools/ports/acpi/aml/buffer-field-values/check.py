#!/usr/bin/env python3
"""Execute detached BufferField reads in isolated runner builds and record exact inputs."""
import argparse,hashlib,json,os,subprocess,tempfile,time
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT;CANONICAL=Path('/Users/zcanann/Documents/projects/Cathedral');SHARED=CANONICAL/'tools/ports/acpi/interpreter/execution'
PIN='eaa7993a23623cd8fabf45350340479c5c9c7879'
SOURCE=['source/libraries/acpi/aml/'+n for n in ['build.omg','model.omg','buffer_field_values.omg','bytes.omg','names.omg','namespace.omg','object_references.omg','byte_storage.omg']]+['source/libraries/acpi/interpreter/'+n for n in ['build.omg','integers.omg','conversions.omg','buffer_fields.omg']]
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def snapshot():
 paths=[ROOT/p for p in SOURCE]+list(HERE.glob('*.py'))+[HERE/'cases.json',HERE/'object_reference.rs',HERE/'reference.Cargo.lock']+[SHARED/n for n in ['checked_runner.rs','runner.Cargo.lock']]
 return {str(p.relative_to(ROOT))if p.is_relative_to(ROOT)else str(p):sha(p)for p in paths}
def build_runner(omega):
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=omega,text=True).strip()==PIN
 assert not subprocess.check_output(['git','status','--porcelain'],cwd=omega,text=True).strip()
 target=Path('/tmp/cathedral-acpi-buffer-field-value-runner')
 with tempfile.TemporaryDirectory(prefix='cathedral-field-value-runner-')as d:
  work=Path(d);lines=['[package]','name="cathedral-acpi-checked-runner"','version="0.1.0"','edition="2024"','[dependencies]']
  for name,path in {'checked-interpreter':'psi/semantics/checked-interpreter','package-manager':'omega/packages/manager','target':'omega/representations/target'}.items():lines.append(f'{name}={{path="{omega}/omega-rust/{path}"}}')
  lines+=['[[bin]]','name="cathedral-acpi-checked-runner"',f'path="{SHARED}/checked_runner.rs"'];(work/'Cargo.toml').write_text('\n'.join(lines)+'\n');(work/'Cargo.lock').write_bytes((SHARED/'runner.Cargo.lock').read_bytes())
  subprocess.run(['cargo','build','--offline','--locked','--release','--manifest-path',str(work/'Cargo.toml'),'--target-dir',str(target)],cwd=omega,check=True)
 return target/'release/cathedral-acpi-checked-runner'
def source(rows):
 text=fixtures.IMPORTS+fixtures.HELPERS+'data Suite{}\n';names=[]
 for row in rows:
  for control in [False,True]:
   name='Suite::'+row['name']+('_control'if control else'_positive');text+=fixtures.render(row,control,name+'(&mut self)');names.append(name+'='+str(int(control)))
 return text,names

def main():
 p=argparse.ArgumentParser();p.add_argument('--omega-source',type=Path,default=CANONICAL.parent/'Omega');p.add_argument('--match',default='');p.add_argument('--batch',type=int,default=10);p.add_argument('--record',type=Path);a=p.parse_args();assert a.batch>0
 subprocess.run(['python3',str(HERE/'fixtures.py'),'--check'],check=True)
 rows=[r for r in fixtures.cases()if any(x in r['name']for x in a.match.split(','))];assert rows
 inputs=snapshot();runner=build_runner(a.omega_source.resolve());binary=sha(runner);records=[]
 with tempfile.TemporaryDirectory(prefix='cathedral-field-values-')as d:
  work=Path(d);(work/'build.omg').write_text('machine build(builder:&mut Build){builder.application("cathedral-field-values");builder.freestanding=true;builder.depend_as("aml",Source::Path {location:"'+str(ROOT/'source/libraries/acpi/aml')+'"});builder.depend_as("integer_helpers",Source::Path {location:"'+str(ROOT/'source/libraries/acpi/interpreter')+'"});}')
  for i in range(0,len(rows),a.batch):
   batch=rows[i:i+a.batch];text,names=source(batch);(work/'main.omg').write_text(text);assert sha(runner)==binary;start=time.monotonic()
   run=subprocess.run([str(runner),str(work/'main.omg'),str(work/'build'),*names],env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'),capture_output=True,text=True)
   output=run.stdout+run.stderr;print(output,flush=True);assert run.returncode==0,output
   assert snapshot()==inputs and sha(runner)==binary,'inputs changed during batch'
   records.append(dict(cases=[r['name']for r in batch],source_sha256=hashlib.sha256(text.encode()).hexdigest(),runner_sha256=binary,seconds=round(time.monotonic()-start,3),output=output))
 if a.record:a.record.write_text(json.dumps(dict(stage='checked interpreter actual Omega bodies and changed-body controls; native/hardware not run',omega_revision=PIN,runner_sha256=binary,input_sha256=inputs,cases=[r['name']for r in rows],positive_count=len(rows),control_count=len(rows),batches=records),indent=2,sort_keys=True)+'\n')
 print('PASS',len(rows),'detached field-read pairs')
if __name__=='__main__':main()
