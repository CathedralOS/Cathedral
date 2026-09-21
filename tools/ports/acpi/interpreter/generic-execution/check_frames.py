#!/usr/bin/env python3
"""Focused frame-selection/publication bodies and expected-value controls."""
import argparse,hashlib,json,os,subprocess,tempfile,time
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
RUNNER=Path('/tmp/cathedral-acpi-generic-checked/release/cathedral-acpi-checked-runner');OMEGA=Path('/tmp/cathedral-omega-eaa7993/release/omega')
NAMES=[str(i)for i in range(8)]
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('--record',type=Path);a=p.parse_args();a.const=False
 paths=list((ROOT/'source/libraries/acpi').rglob('*.omg'))+[HERE/'frames.omg',Path(__file__).resolve(),HERE/'build.omg']
 before={str(p.relative_to(ROOT)):sha(p)for p in sorted(paths)};source=(HERE/'frames.omg').read_text();binary=OMEGA if a.const else RUNNER;binary_hash=sha(binary)
 results=[];started=time.monotonic()
 with tempfile.TemporaryDirectory(prefix='cathedral-generic-kernels-')as directory:
  root=Path(directory);build=(HERE/'build.omg').read_text()
  for relative in ['../../../../../source/libraries/acpi/aml','../../../../../source/libraries/acpi/interpreter/execution','../../../../../source/libraries/acpi/interpreter']:build=build.replace(relative,str((HERE/relative).resolve()))
  (root/'build.omg').write_text(build)
  (root/'main.omg').write_text(source);selection=[f'Suite::frame_{name}_{int(c)}={int(c)}'for name in NAMES for c in [False,True]];command=[str(RUNNER),str(root/'main.omg'),str(root/'build'),*selection]
  done=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'));print(done.stdout,flush=True);done.check_returncode();results.append({'fixture_sha256':hashlib.sha256(source.encode()).hexdigest(),'exit_code':done.returncode,'output':done.stdout})
 assert before=={str(p.relative_to(ROOT)):sha(p)for p in sorted(paths)},'Source changed during kernel verification'
 if a.record:a.record.write_text(json.dumps({'stage':'constant evaluator'if a.const else'checked interpreter; generic target and copy helpers, engine/pipeline not imported','cases':1 if a.const else len(NAMES),'controls':1 if a.const else len(NAMES),'binary_sha256':binary_hash,'elapsed_seconds':time.monotonic()-started,'source_snapshot_sha256':before,'results':results},indent=2,sort_keys=True)+'\n')
 print('PASS kernel bodies and expectation controls')
if __name__=='__main__':main()
