#!/usr/bin/env python3
"""Check source binding, actual pinned observations and Omega body evaluation."""
import argparse,hashlib,json,subprocess,sys,tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def run(*args):subprocess.run([str(x) for x in args],cwd=ROOT,check=True)
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--omega',type=Path,default=ROOT.parent/'Omega/target/release/omega');p.add_argument('--host-only',action='store_true');g=p.add_mutually_exclusive_group();g.add_argument('--positive-only',action='store_true');g.add_argument('--controls-only',action='store_true');p.add_argument('--batch',type=int);p.add_argument('--jobs',type=int,default=4);a=p.parse_args();
 if not 1<=a.jobs<=8:p.error('jobs must be1..8')
 fixtures=json.loads((HERE/'fixtures.json').read_text())
 if a.batch is not None and (a.controls_only or not 0<=a.batch<len(fixtures['batches'])):p.error('batch must select an existing positive fixture')
 for script in ['generate_reference.py','generate_cases.py','generate_main.py','generate_fixtures.py','generate_inventory.py','observe.py']:run(sys.executable,HERE/script,'--check')
 if a.host_only:return
 omega=a.omega.resolve();print('Omega SHA256:',hashlib.sha256(omega.read_bytes()).hexdigest(),flush=True)
 if not a.controls_only:
  if a.batch is None:
   for n in ['encrypted_mapping_plans.omg','encrypted_mapping_routes.omg','encrypted_recursive_routes.omg']:run(omega,'--check',ROOT/'source/libraries/x86_64'/n)
  selected=[b for j,b in enumerate(fixtures['batches']) if a.batch is None or a.batch==j]
  with ThreadPoolExecutor(max_workers=a.jobs) as pool:
   list(pool.map(lambda b:run(omega,'--check',HERE/b['path']),selected))
  if a.batch is None:
   run(omega,'--check',HERE/'leaf_checks.omg');run(omega,'--check',HERE/'extras.omg')
 if not a.positive_only and a.batch is None:
  controls=fixtures['controls']+[{'path':'extras.omg','old':'expected==327680','new':'expected==327681','family':'capture-mismatch-payload'},{'path':'extras.omg','old':'result.edits.allocation_calls==1','new':'result.edits.allocation_calls==0','family':'invalid-allocation-attempt-retained'}]
  def control(c):
   with tempfile.TemporaryDirectory(prefix='cathedral-encrypted-mapping-control-') as temp:
    d=Path(temp);s=(HERE/c['path']).read_text();assert s.count(c['old'])==1
    (d/'main.omg').write_text(s.replace(c['old'],c['new']));(d/'checks.omg').write_text((HERE/'checks.omg').read_text());(d/'build.omg').write_text((HERE/'build.omg').read_text().replace('../../../source/',str(ROOT/'source')+'/'))
    r=subprocess.run([str(omega),'--check',str(d/'main.omg')],cwd=ROOT,text=True,capture_output=True);out=r.stdout+r.stderr
    if r.returncode==0 or 'cannot prove requires contract' not in out or '1 == 0' not in out:raise SystemExit('mutation did not compute failure:\n'+out)
   print('PASS body mutation:',c['family'],flush=True)
  with ThreadPoolExecutor(max_workers=a.jobs) as pool:list(pool.map(control,controls))
 print('PASS requested encrypted mapping checks',flush=True)
if __name__=='__main__':main()
