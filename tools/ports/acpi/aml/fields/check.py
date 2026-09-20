#!/usr/bin/env python3
"""Original field syntax fixtures: semantic behavior plus body-mutating negative controls."""
import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import time
ROOT=Path(__file__).resolve().parents[5]
HERE=Path(__file__).resolve().parent

def main():
 parser=argparse.ArgumentParser(description=__doc__)
 parser.add_argument('--omega',type=Path,default=Path('/tmp/cathedral-omega-eaa7993/release/omega'))
 parser.add_argument('--case',action='append')
 parser.add_argument('--jobs',type=int,default=2)
 parser.add_argument('--positive-only',action='store_true')
 args=parser.parse_args();compiler=args.omega.resolve()
 cases=json.loads((HERE/'cases.json').read_text())
 selected=args.case or list(cases)
 print('Omega SHA-256:',hashlib.sha256(compiler.read_bytes()).hexdigest(),flush=True)
 def package_hash():
  digest=hashlib.sha256()
  for file in sorted(list((ROOT/'source/libraries/acpi/aml').glob('*.omg'))+list((ROOT/'source/libraries/acpi/aml/fields').glob('*.omg'))):
   digest.update(str(file.relative_to(ROOT/'source/libraries/acpi/aml')).encode());digest.update(b'\0');digest.update(file.read_bytes())
  return digest.hexdigest()
 before=package_hash();print('Field syntax closure SHA-256:',before,flush=True)
 build=(HERE/'build.omg').read_text().replace('../../../../../source/libraries/acpi/aml',str(ROOT/'source/libraries/acpi/aml'))
 def check(name):
  started=time.monotonic()
  source=(HERE/'cases'/f'{name}.omg').read_text();old,new=cases[name]['mutation']
  assert source.count(old)==1,(name,'nonunique mutation')
  with tempfile.TemporaryDirectory(prefix='cathedral-fields-'+name+'-') as folder:
   work=Path(folder);(work/'build.omg').write_text(build);(work/'main.omg').write_text(source)
   result=subprocess.run([str(compiler),'--check',str(work/'main.omg')],cwd=ROOT,capture_output=True,text=True)
   output=result.stdout+result.stderr
   if result.returncode:raise RuntimeError(name+' positive failed:\n'+output)
   print(f'{name}: positive PASS ({time.monotonic()-started:.1f}s)',flush=True)
   if not args.positive_only:
    (work/'main.omg').write_text(source.replace(old,new))
    result=subprocess.run([str(compiler),'--check',str(work/'main.omg')],cwd=ROOT,capture_output=True,text=True)
    output=result.stdout+result.stderr
    if result.returncode==0 or 'cannot prove requires contract' not in output or '1 == 0' not in output:raise RuntimeError(name+' negative failed to compute1:\n'+output)
    print(name+': changed behavior computes1 and is rejected',flush=True)
  return name
 results=[];errors=[]
 with ThreadPoolExecutor(max_workers=args.jobs) as pool:
  futures={pool.submit(check,name):name for name in selected}
  for future in as_completed(futures):
   try:results.append(future.result())
   except Exception as error:
    errors.append(str(error));print(str(error),flush=True)
 if package_hash()!=before:raise SystemExit('Field syntax closure changed during checks; repeat against a fixed source hash.')
 if errors:raise SystemExit(f'{len(errors)} cases failed; {len(results)} cases passed.')
 print(f'{len(results)} positive cases passed'+('' if args.positive_only else f'; {len(results)} body-mutating controls rejected')+'.',flush=True)
 print('Semantic evaluation only; no native ABI, interpreter execution, firmware, table mapping or handler access.')
if __name__=='__main__':main()
