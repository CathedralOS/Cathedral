#!/usr/bin/env python3
"""Execute direct description-aware concatenations and record exact sources and dependency roots."""
import argparse,hashlib,json,os,subprocess,tempfile,time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT;CANONICAL=Path("/Users/zcanann/Documents/projects/Cathedral");SHARED=CANONICAL/'tools/ports/acpi/interpreter/execution'
PIN='eaa7993a23623cd8fabf45350340479c5c9c7879'
SOURCE=['source/libraries/acpi/aml/'+n for n in ['build.omg','model.omg','bytes.omg','names.omg','namespace.omg','object_references.omg','byte_storage.omg','object_conversions.omg','object_concat.omg','object_descriptions.omg','implicit_conversions.omg','object_concat_described.omg']]+['source/libraries/acpi/interpreter/'+n for n in ['build.omg','integers.omg','conversions.omg','buffer_fields.omg','string_numbers.omg','implicit_integer.omg','implicit_strings.omg','byte_concat.omg']]
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def snapshot():
 paths=[ROOT/p for p in SOURCE]+list(HERE.glob('*.py'))+[HERE/'cases.json']+[SHARED/n for n in ['checked_runner.rs','runner.Cargo.lock']]
 return {str(p.relative_to(ROOT))if p.is_relative_to(ROOT)else str(p):sha(p)for p in paths}
def build_text(stage):
 assert stage in ['runtime','constant']
 application='cathedral-object-concat-described'if stage=='runtime'else'object-concat-described-constant'
 root=ROOT.resolve()
 return 'machine build(builder:&mut Build){builder.application("'+application+'");builder.freestanding=true;builder.depend_as("aml",Source::Path {location:"'+str(root/'source/libraries/acpi/aml')+'"});builder.depend_as("integer_helpers",Source::Path {location:"'+str(root/'source/libraries/acpi/interpreter')+'"});}'
def text_sha(text):return hashlib.sha256(text.encode()).hexdigest()
def build_runner(omega):
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=omega,text=True).strip()==PIN
 assert not subprocess.check_output(['git','status','--porcelain'],cwd=omega,text=True).strip()
 target=Path('/tmp/cathedral-acpi-object-concat-described-runner')
 with tempfile.TemporaryDirectory(prefix='cathedral-object-concat-described-runner-')as d:
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
 p=argparse.ArgumentParser();p.add_argument('--omega-source',type=Path,default=CANONICAL.parent/'Omega');p.add_argument('--match',default='');p.add_argument('--batch',type=int,default=24);p.add_argument('--workers',type=int,default=2);p.add_argument('--record',type=Path);a=p.parse_args();assert a.batch>0 and 1<=a.workers<=3
 subprocess.run(['python3',str(HERE/'fixtures.py'),'--check'],check=True)
 rows=[r for r in fixtures.cases()if any(x in r['name']for x in a.match.split(','))];assert rows
 execution_root=str(ROOT.resolve());build=build_text('runtime');build_sha256=text_sha(build)
 inputs=snapshot();runner=build_runner(a.omega_source.resolve());binary=sha(runner);records=[]
 def run_batch(batch):
  with tempfile.TemporaryDirectory(prefix='cathedral-object-concat-described-')as d:
   work=Path(d);(work/'build.omg').write_text(build);assert sha(work/'build.omg')==build_sha256
   text,names=source(batch);(work/'main.omg').write_text(text);assert sha(runner)==binary;start=time.monotonic()
   run=subprocess.run([str(runner),str(work/'main.omg'),str(work/'build'),*names],env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'),capture_output=True,text=True)
   output=run.stdout+run.stderr;print(output,flush=True);assert run.returncode==0,output
   assert snapshot()==inputs and sha(runner)==binary and sha(work/'build.omg')==build_sha256,'inputs changed during batch'
   return dict(execution_root=execution_root,build_sha256=build_sha256,cases=[r['name']for r in batch],source_sha256=hashlib.sha256(text.encode()).hexdigest(),runner_sha256=binary,seconds=round(time.monotonic()-start,3),output=output)
 with ThreadPoolExecutor(max_workers=a.workers)as executor:records=list(executor.map(run_batch,[rows[i:i+a.batch]for i in range(0,len(rows),a.batch)]))
 if a.record:a.record.write_text(json.dumps(dict(execution_root=execution_root,build_text=build,build_sha256=build_sha256,workers=a.workers,stage='checked interpreter actual Omega bodies and changed-body controls; native/hardware not run',omega_revision=PIN,runner_sha256=binary,input_sha256=inputs,cases=[r['name']for r in rows],positive_count=len(rows),control_count=len(rows),batches=records),indent=2,sort_keys=True)+'\n')
 print('PASS',len(rows),'description-aware concatenation behavior/control pairs')
if __name__=='__main__':main()
