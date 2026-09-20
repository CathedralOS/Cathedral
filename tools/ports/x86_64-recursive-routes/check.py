#!/usr/bin/env python3
"""Execute adapted pinned bodies and actual Omega captured algorithms with controls."""
import argparse,hashlib,json,subprocess,sys,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def run(*args):subprocess.run([str(a) for a in args],cwd=ROOT,check=True)
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--omega',type=Path,default=ROOT.parent/'Omega/target/release/omega');p.add_argument('--host-only',action='store_true');g=p.add_mutually_exclusive_group();g.add_argument('--positive-only',action='store_true');g.add_argument('--controls-only',action='store_true');p.add_argument('--route-batch',type=int);a=p.parse_args()
 f=json.loads((HERE/'fixtures.json').read_text())
 if a.route_batch is not None and (a.controls_only or not 0<=a.route_batch<len(f['routes'])):p.error('route-batch must select an existing positive batch')
 for script in ['generate_reference.py','generate_cases.py','generate_main.py','generate_fixtures.py','generate_inventory.py']:run(sys.executable,HERE/script,'--check')
 actual=json.loads(subprocess.check_output(['cargo','run','--quiet','--locked','--manifest-path',str(HERE/'Cargo.toml')],cwd=ROOT,text=True));assert actual==json.loads((HERE/'observations.json').read_text()),'fresh adapted-body observations changed'
 print('PASS232 adapted source-body route observations and90 translation observations',flush=True)
 if a.host_only:return
 omega=a.omega.resolve();print('Omega SHA256:',hashlib.sha256(omega.read_bytes()).hexdigest(),flush=True)
 if not a.controls_only:
  if a.route_batch is None:
   for module in ['recursive_routes.omg','recursive_translation.omg']:run(omega,'--check',ROOT/'source/libraries/x86_64'/module)
  for n,row in enumerate(f['routes']):
   if a.route_batch is None or a.route_batch==n:run(omega,'--check',HERE/row['path'])
  if a.route_batch is None:
   for row in f['translations']:run(omega,'--check',HERE/row['path'])
   run(omega,'--check',HERE/'extras.omg')
 if not a.positive_only and a.route_batch is None:
  build=(HERE/'build.omg').read_text().replace('../../../source/',str(ROOT/'source')+'/')
  controls=f['controls']+[{'path':'extras.omg','old':'result.edits.write_mask == 1','new':'result.edits.write_mask == 0','family':'capture-mismatch-retained-write'}]
  for c in controls:
   with tempfile.TemporaryDirectory(prefix='cathedral-recursive-route-control-') as temp:
    d=Path(temp);s=(HERE/c['path']).read_text();assert s.count(c['old'])==1
    (d/'main.omg').write_text(s.replace(c['old'],c['new']));(d/'build.omg').write_text(build)
    for helper in ['checks.omg','translation_checks.omg']:(d/helper).write_text((HERE/helper).read_text())
    r=subprocess.run([str(omega),'--check',str(d/'main.omg')],cwd=ROOT,text=True,capture_output=True);out=r.stdout+r.stderr
    if r.returncode==0 or 'cannot prove requires contract' not in out or '1 == 0' not in out:raise SystemExit('body mutation did not compute failure:\n'+out)
   print('PASS body mutation:',c['family'],flush=True)
 print('PASS requested recursive route/translation checks',flush=True)
if __name__=='__main__':main()
