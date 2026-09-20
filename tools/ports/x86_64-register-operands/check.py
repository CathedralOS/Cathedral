#!/usr/bin/env python3
"""Check pinned numeric fragments and execute Omega bodies plus behavior mutations."""
import argparse,hashlib,json,subprocess,sys,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def run(*args):subprocess.run([str(a) for a in args],cwd=ROOT,check=True)
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--omega',type=Path,default=ROOT.parent/'Omega/target/release/omega');p.add_argument('--host-only',action='store_true');p.add_argument('--controls-only',action='store_true');p.add_argument('--positive-only',action='store_true');p.add_argument('--batch',type=int);a=p.parse_args()
 for script in ('generate_reference.py','generate_fixtures.py','generate_inventory.py'):run(sys.executable,HERE/script,'--check')
 observed=subprocess.check_output(['cargo','run','--quiet','--locked','--manifest-path',str(HERE/'Cargo.toml')],cwd=ROOT,text=True)
 expected=json.loads((HERE/'observations.json').read_text())
 if json.loads(observed)!=expected:raise SystemExit('actual pinned-fragment observations changed')
 print('PASS',len(expected),'actual extracted Rust observations; no instruction execution',flush=True)
 if a.host_only:return
 omega=a.omega.resolve();print('Omega SHA256:',hashlib.sha256(omega.read_bytes()).hexdigest(),flush=True)
 f=json.loads((HERE/'fixtures.json').read_text())
 if not a.controls_only:
  if a.batch is None:run(omega,'--check',ROOT/'source/libraries/x86_64/register_operands.omg')
  for n,b in enumerate(f['batches']):
   if a.batch is None or n==a.batch:run(omega,'--check',HERE/b['path'])
  if a.batch is None:run(omega,'--check',HERE/'extras.omg')
 if not a.positive_only and a.batch is None:
  build=(HERE/'build.omg').read_text().replace('../../../source/',str(ROOT/'source')+'/')
  for control in f['controls']:
   with tempfile.TemporaryDirectory(prefix='cathedral-register-operand-control-') as temp:
    d=Path(temp);source=(HERE/control['path']).read_text();assert source.count(control['old'])==1
    (d/'main.omg').write_text(source.replace(control['old'],control['new']));(d/'checks.omg').write_text((HERE/'checks.omg').read_text());(d/'build.omg').write_text(build)
    r=subprocess.run([str(omega),'--check',str(d/'main.omg')],cwd=ROOT,text=True,capture_output=True);output=r.stdout+r.stderr
    if r.returncode==0 or 'cannot prove requires contract' not in output or '1 == 0' not in output:raise SystemExit('body mutation did not compute failure:\n'+output)
   print('PASS body mutation:',control['family'],flush=True)
 print('PASS requested register-operand checks',flush=True)
if __name__=='__main__':main()
