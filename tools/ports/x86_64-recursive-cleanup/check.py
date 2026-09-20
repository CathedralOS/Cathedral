#!/usr/bin/env python3
"""Check pinned numeric witnesses and actual Omega recursive cleanup bodies with controls."""
import argparse
from concurrent.futures import ThreadPoolExecutor,as_completed
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
SOURCE_FILES=[
 'source/libraries/x86_64/build.omg','source/libraries/x86_64/recursive_cleanup.omg','source/libraries/x86_64/cleanup_ranges.omg','source/libraries/x86_64/cleanup_branch.omg',
 'source/libraries/x86_64/addresses.omg','source/libraries/x86_64/pages.omg',
 'source/libraries/x86_64/page_entries.omg','source/drivers/facts/build.omg',
 'source/drivers/facts/x86_page_table_entry.omg',
]
def source_hash():
 digest=hashlib.sha256()
 for name in sorted(SOURCE_FILES):digest.update(name.encode());digest.update(b'\0');digest.update((ROOT/name).read_bytes())
 return digest.hexdigest()
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--omega',type=Path,default=Path('/tmp/cathedral-omega-eaa7993/release/omega'));parser.add_argument('--case',action='append');parser.add_argument('--jobs',type=int,default=4);parser.add_argument('--positive-only',action='store_true');parser.add_argument('--host-only',action='store_true');args=parser.parse_args()
 for script in ['generate.py','generate_extras.py','generate_reference.py','map_inventory.py']:
  subprocess.run([sys.executable,str(HERE/script),'--check'],cwd=ROOT,check=True)
 subprocess.run([sys.executable,str(ROOT/'tools/ports/inventory.py'),'check',str(ROOT/'source/libraries/x86_64/recursive-cleanup-inventory.json'),'--checkout',str(ROOT/'reference_code/rust-osdev/x86_64')],cwd=ROOT,check=True)
 subprocess.run(['cargo','+nightly-2026-09-04','run','--quiet','--locked','--manifest-path',str(HERE/'Cargo.toml')],cwd=ROOT,check=True)
 if args.host_only:return
 compiler=args.omega.resolve();before=source_hash();print('Omega SHA-256:',hashlib.sha256(compiler.read_bytes()).hexdigest(),flush=True);print('Selected recursive cleanup closure SHA-256:',before,flush=True)
 rows=json.loads((HERE/'cases.json').read_text())['steps'];cases={}
 for i,row in enumerate(rows):
  old='value.has_plan=='+str(row['has_plan']).lower();new='value.has_plan=='+str(not row['has_plan']).lower();cases[f'step-{i:03}']=[old,new]
 cases.update(json.loads((HERE/'extras.json').read_text()))
 for name in ['budget-max-skipped','budget-max-ordinary']:cases[name]=['remaining==0xfffffffffffffffe','remaining==0xfffffffffffffffd']
 selected=args.case or list(cases)
 build=(HERE/'build.omg').read_text().replace('../../../source/',str(ROOT/'source')+'/')
 def check(name):
  source=(HERE/'cases'/f'{name}.omg').read_text();old,new=cases[name];assert source.count(old)==1,name
  with tempfile.TemporaryDirectory(prefix='cathedral-recursive-cleanup-'+name+'-') as folder:
   work=Path(folder);(work/'build.omg').write_text(build);(work/'main.omg').write_text(source)
   result=subprocess.run([str(compiler),'--check',str(work/'main.omg')],cwd=ROOT,capture_output=True,text=True)
   if result.returncode:raise RuntimeError(name+' positive failed:\n'+result.stdout+result.stderr)
   print(name+': positive PASS',flush=True)
   if not args.positive_only:
    (work/'main.omg').write_text(source.replace(old,new));result=subprocess.run([str(compiler),'--check',str(work/'main.omg')],cwd=ROOT,capture_output=True,text=True);output=result.stdout+result.stderr
    if result.returncode==0 or 'cannot prove requires contract' not in output or '1 == 0' not in output:raise RuntimeError(name+' body mutation failed to compute1:\n'+output)
    print(name+': changed behavior computes1 and is rejected',flush=True)
  return name
 results=[];errors=[]
 with ThreadPoolExecutor(max_workers=args.jobs) as pool:
  futures={pool.submit(check,name):name for name in selected}
  for future in as_completed(futures):
   try:results.append(future.result())
   except Exception as error:errors.append(str(error));print(str(error),flush=True)
 if source_hash()!=before:raise SystemExit('Selected source closure changed during checks.')
 if errors:raise SystemExit(f'{len(errors)} cases failed; {len(results)} passed.')
 print(f'{len(results)} positive cases passed'+('' if args.positive_only else f'; {len(results)} body-mutating controls rejected')+'.',flush=True)
 print('Numeric semantic evaluation plus extracted private Rust mirror on owned initialized tables; no live recursive table construction or custody claim.')
if __name__=='__main__':main()
