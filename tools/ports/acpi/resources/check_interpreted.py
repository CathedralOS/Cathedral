#!/usr/bin/env python3
"""Check one authored resource suite and interpret each actual body/control."""
import argparse,hashlib,json,os,re,subprocess,tempfile,time
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[3]
PIN='eaa7993a23623cd8fabf45350340479c5c9c7879'
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--omega-source',type=Path,default=ROOT.parent/'Omega');p.add_argument('--case',action='append');p.add_argument('--record',type=Path);p.add_argument('--target-dir',type=Path,default=Path('/tmp/cathedral-acpi-resources-checked'));a=p.parse_args();omega=a.omega_source.resolve();target=a.target_dir.resolve()
 revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=omega,text=True).strip();assert revision==PIN,'requires reviewed Omega pin'
 assert not subprocess.check_output(['git','status','--porcelain'],cwd=omega,text=True).strip(),'requires clean Omega source'
 subprocess.run(['python3',str(HERE/'generate.py'),'--check'],cwd=ROOT,check=True)
 subprocess.run(['python3',str(HERE/'map_inventory.py'),'--check'],cwd=ROOT,check=True)
 subprocess.run(['python3',str(ROOT/'tools/ports/inventory.py'),'check',str(ROOT/'source/libraries/acpi/resources/inventory.json'),'--checkout',str(ROOT/'reference_code/rust-osdev/acpi')],cwd=ROOT,check=True)
 cases=json.loads((HERE/'cases.json').read_text());names=a.case or list(cases)
 source_files=sorted(list((ROOT/'source/libraries/acpi/resources').glob('*.omg'))+[ROOT/name for name in ['source/libraries/acpi/build.omg','source/libraries/acpi/bytes.omg','source/libraries/acpi/fixed_bytes.omg']])
 def hashes():return {str(f.relative_to(ROOT)):hashlib.sha256(f.read_bytes()).hexdigest()for f in source_files}
 before=hashes()
 with tempfile.TemporaryDirectory(prefix='cathedral-resource-checked-')as directory:
  root=Path(directory);manifest=['[package]','name="cathedral-acpi-checked-runner"','version="0.1.0"','edition="2024"','[dependencies]']
  for name,path in {'checked-interpreter':'psi/semantics/checked-interpreter','package-manager':'omega/packages/manager','target':'omega/representations/target'}.items():manifest.append(f'{name} = {{ path = "{omega}/omega-rust/{path}" }}')
  manifest+=['[[bin]]','name="cathedral-acpi-checked-runner"',f'path="{HERE}/checked_runner.rs"'];(root/'Cargo.toml').write_text('\n'.join(manifest)+'\n');(root/'Cargo.lock').write_bytes((HERE/'runner.Cargo.lock').read_bytes())
  subprocess.run(['cargo','build','--offline','--locked','--release','--manifest-path',str(root/'Cargo.toml'),'--target-dir',str(target)],cwd=omega,check=True)
  runner=target/'release/cathedral-acpi-checked-runner';imports=[];bodies=[];selections=[]
  for name in names:
   src=(HERE/'cases'/f'{name}.omg').read_text();imports.extend(re.findall(r'^use .*;',src,re.M));src=re.sub(r'^use .*;\n','',src,flags=re.M);src=src[:src.index('const RESULT:')]
   for negative in [False,True]:
    body=src
    if negative:
     old,new=cases[name]['mutation'];assert body.count(old)==1,name;body=body.replace(old,new)
    unique=name.replace('-','_')+('_control'if negative else'_positive');machine='ResourceSuite::'+unique
    for helper in ['check_resource','check_interrupts']:body=re.sub(r'\b'+helper+r'\b',unique+'_'+helper,body)
    assert body.count('machine test()')==1,name;body=body.replace('machine test()','machine '+machine+'(&mut self)',1)
    bodies.append(body);selections.append(machine+'='+str(int(negative)))
  suite='\n'.join(sorted(set(imports)))+'\ndata ResourceSuite{}\n'+''.join(bodies)
  (root/'main.omg').write_text(suite);(root/'build.omg').write_text((HERE/'build.omg').read_text().replace('../../../../source/libraries/acpi/resources',str(ROOT/'source/libraries/acpi/resources')))
  print('Checked resource suite:',len(names),'positives and',len(names),'changed-body controls',flush=True)
  env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000');started=time.monotonic();command=[str(runner),str(root/'main.omg'),str(root/'build'),*selections];process=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=env);lines=[]
  for line in process.stdout:print(line,end='',flush=True);lines.append(line)
  code=process.wait();assert before==hashes(),'source changed during suite';output=''.join(lines)
  if code:raise SystemExit(code)
  if a.record:
   record={'format':'cathedral-resource-checked-v1','stage':'checked-interpreter execution; no native ABI or firmware execution','omega_revision':revision,'runner_sha256':hashlib.sha256(runner.read_bytes()).hexdigest(),'runner_source_sha256':hashlib.sha256((HERE/'checked_runner.rs').read_bytes()).hexdigest(),'cargo_lock_sha256':hashlib.sha256((root/'Cargo.lock').read_bytes()).hexdigest(),'suite_sha256':hashlib.sha256(suite.encode()).hexdigest(),'source_sha256':before,'fixture_sha256':{n:hashlib.sha256((HERE/'cases'/f'{n}.omg').read_bytes()).hexdigest()for n in names},'scenario_count':len(names),'control_count':len(names),'elapsed_seconds':round(time.monotonic()-started,3),'output':output,'command':'python3 tools/ports/acpi/resources/check_interpreted.py --record tools/ports/acpi/resources/checked-verification.json'}
   a.record.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 print(f'PASS {len(names)} checked resource bodies + {len(names)} changed-body controls.',flush=True)
if __name__=='__main__':main()
