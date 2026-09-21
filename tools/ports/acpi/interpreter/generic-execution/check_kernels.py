#!/usr/bin/env python3
"""Independent actual canonical data-copy kernels, checked and optional const controls."""
import argparse,hashlib,json,os,subprocess,tempfile,time
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
RUNNER=Path('/tmp/cathedral-acpi-generic-checked/release/cathedral-acpi-checked-runner');OMEGA=Path('/tmp/cathedral-omega-eaa7993/release/omega')
NAMES=['bytes','self','clear','invalid','package','capacity']
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('--record',type=Path);p.add_argument('--const',action='store_true');a=p.parse_args()
 paths=list((ROOT/'source/libraries/acpi').rglob('*.omg'))+[HERE/'kernels.omg',Path(__file__).resolve(),HERE/'build.omg']
 before={str(p.relative_to(ROOT)):sha(p)for p in sorted(paths)};source=(HERE/'kernels.omg').read_text();binary=OMEGA if a.const else RUNNER;binary_hash=sha(binary)
 results=[];started=time.monotonic()
 with tempfile.TemporaryDirectory(prefix='cathedral-generic-kernels-')as directory:
  root=Path(directory);build=(HERE/'build.omg').read_text()
  for relative in ['../../../../../source/libraries/acpi/aml','../../../../../source/libraries/acpi/interpreter/execution','../../../../../source/libraries/acpi/interpreter']:build=build.replace(relative,str((HERE/relative).resolve()))
  (root/'build.omg').write_text(build)
  if a.const:
   for control in [False,True]:
    text=source+f'\nconst TEST_RESULT:i32=gk_check(0,{66 if control else 65});\nmachine require_success(value:i32) requires value==0; {{}}\ndata Main {{}}\nmachine Main::main(&mut self) {{require_success(TEST_RESULT);}}\n';(root/'main.omg').write_text(text)
    command=[str(OMEGA),'--check',str(root/'main.omg')];done=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
    print(done.stdout,flush=True);assert (done.returncode!=0)==control,done.stdout
    if control:assert 'require' in done.stdout.lower() and ('prov' in done.stdout.lower()or'precondition'in done.stdout.lower()),done.stdout
    results.append({'control':control,'fixture_sha256':hashlib.sha256(text.encode()).hexdigest(),'command':command,'exit_code':done.returncode,'output':done.stdout})
  else:
   (root/'main.omg').write_text(source);selection=[f'Kernels::{name}_{"control"if c else"positive"}={int(c)}'for name in NAMES for c in [False,True]];command=[str(RUNNER),str(root/'main.omg'),str(root/'build'),*selection]
   done=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'));print(done.stdout,flush=True);done.check_returncode();results.append({'fixture_sha256':hashlib.sha256(source.encode()).hexdigest(),'exit_code':done.returncode,'output':done.stdout})
 assert before=={str(p.relative_to(ROOT)):sha(p)for p in sorted(paths)},'Source changed during kernel verification'
 assert binary_hash==sha(binary),'Runner or compiler changed during kernel verification'
 if a.record:a.record.write_text(json.dumps({'execution_root':str(ROOT.resolve()),'build_source':build,'build_sha256':hashlib.sha256(build.encode()).hexdigest(),'stage':'constant evaluator'if a.const else'checked interpreter; generic_values only, engine/pipeline not imported','cases':1 if a.const else len(NAMES),'controls':1 if a.const else len(NAMES),'binary_sha256':binary_hash,'elapsed_seconds':time.monotonic()-started,'source_snapshot_sha256':before,'results':results},indent=2,sort_keys=True)+'\n')
 print('PASS kernel bodies and expectation controls')
if __name__=='__main__':main()
