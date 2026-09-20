#!/usr/bin/env python3
"""Evaluate real bounded AML execution and mutate each expected result as control."""
import argparse,concurrent.futures,hashlib,subprocess,tempfile,time
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--omega',type=Path,default=Path('/tmp/cathedral-omega-eaa7993/release/omega'));p.add_argument('--match',default='');p.add_argument('--jobs',type=int,default=2);args=p.parse_args();compiler=args.omega.resolve()
 print('Omega SHA-256:',hashlib.sha256(compiler.read_bytes()).hexdigest(),flush=True)
 subprocess.run(['python3',str(HERE/'fixtures.py'),'--check'],check=True,cwd=ROOT)
 rows=[r for r in fixtures.cases()if any(part in r['name'] for part in args.match.split(','))]
 if not rows:raise SystemExit('No matching cases')
 build=(HERE/'build.omg').read_text()
 for source in ['../../../../../source/libraries/acpi/interpreter/execution','../../../../../source/libraries/acpi/interpreter','../../../../../source/libraries/acpi/aml']:
  build=build.replace(source,str((HERE/source).resolve()))
 def run(row,negative=False):
  source=fixtures.render(row)
  if negative:
   marker=(f'result.value.number == {row["expected"]}'if row['expected']is not None and row['error']=='Success'else f'result.outcome == ExecutionOutcome::{row["error"]}')
   replacement=(f'result.value.number == {(row["expected"]+1)&((1<<row["bits"])-1)}'if row['expected']is not None and row['error']=='Success'else f'result.outcome == ExecutionOutcome::{"BadEncoding"if row["error"]=="Success"else"Success"}')
   if source.count(marker)!=1:raise RuntimeError('Expected unique control: '+row['name'])
   source=source.replace(marker,replacement)
  start=time.monotonic()
  with tempfile.TemporaryDirectory(prefix='cathedral-aml-execution-'+row['name']+'-')as directory:
   path=Path(directory);(path/'build.omg').write_text(build);(path/'main.omg').write_text(source)
   completed=subprocess.run([str(compiler),'--check',str(path/'main.omg')],capture_output=True,text=True,cwd=ROOT)
   output=completed.stdout+completed.stderr
   valid=(completed.returncode!=0 and 'cannot prove requires contract'in output and '1 == 0'in output)if negative else completed.returncode==0
   if not valid:
    print('FAIL '+row['name']+(' NEGATIVE'if negative else' POSITIVE')+'\n'+output,flush=True)
    raise RuntimeError(row['name']+' failed')
  print(('CONTROL 'if negative else'PASS ')+row['name']+f' ({time.monotonic()-start:.1f}s)',flush=True)
 def scenario(row):
  run(row);run(row,True)
 with concurrent.futures.ThreadPoolExecutor(max_workers=max(1,min(args.jobs,4)))as pool:
  futures=[pool.submit(scenario,row)for row in rows]
  for future in concurrent.futures.as_completed(futures):future.result()
 print(f'PASS {len(rows)} real AML bytecode scenarios and {len(rows)} body-mutating controls. Native/hardware NOT RUN.',flush=True)
if __name__=='__main__':main()
