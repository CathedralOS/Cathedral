#!/usr/bin/env python3
"""Run source-bound normal Field loader scenario/control pairs."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time
import fixtures
ROOT=fixtures.ROOT
HERE=fixtures.HERE

def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def snapshot():
 # This fixture package depends only on the parent AML package.
 paths=sorted((ROOT/'source/libraries/acpi/aml').glob('*.omg'))
 paths += sorted(p for p in HERE.iterdir()if p.suffix in ['.py','.omg','.txt']or p.name=='cases.json')
 return {str(p.relative_to(ROOT)):sha(p)for p in paths}
def build():return 'machine build(builder:&mut Build){builder.package("field-namespace-fixtures");builder.freestanding=true;builder.depend_as("aml",Source::Path {location:"'+str(ROOT/'source/libraries/acpi/aml')+'"});}\n'
def main():
 p=argparse.ArgumentParser(description=__doc__)
 p.add_argument('--runner',type=Path,default=Path('/tmp/cathedral-acpi-generic-checked/release/cathedral-acpi-checked-runner'))
 p.add_argument('--omega',type=Path,default=Path('/tmp/cathedral-omega-eaa7993/release/omega'))
 p.add_argument('--match',default='');p.add_argument('--record',type=Path,required=True);p.add_argument('--const',action='store_true');a=p.parse_args()
 all_rows=fixtures.cases();assert (HERE/'main.omg').read_text()==fixtures.render(all_rows)[0]
 rows=[r for r in all_rows if not a.match or r['name']in a.match.split(',')];assert rows
 if a.const:assert len(rows)==1
 before=snapshot();binary=a.omega if a.const else a.runner;binary_hash=sha(binary);build_text=build();started=time.monotonic();results=[]
 with tempfile.TemporaryDirectory(prefix='cathedral-field-namespace-')as directory:
  work=Path(directory);(work/'build.omg').write_text(build_text)
  if not a.const:
   source,selections=fixtures.render(rows);sources=[source];commands=[[str(binary),str(work/'main.omg'),str(work/'build'),*selections]]
  else:
   sources=[];commands=[]
   for control in [False,True]:
    source=fixtures.IMPORTS+fixtures.HELPERS+'machine test()->i32{'+fixtures.fixture_body(rows[0],control)+'}\nconst RESULT:i32=test();\nmachine require_ok(value:i32) requires value==0; {}\ndata Main {}\nmachine Main::main(&mut self){require_ok(RESULT);}\n'
    sources.append(source);commands.append([str(binary),'--check',str(work/'main.omg')])
  for ordinal,(source,command)in enumerate(zip(sources,commands)):
   (work/'main.omg').write_text(source);lines=[]
   process=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'))
   for line in process.stdout:print(line,end='',flush=True);lines.append(line)
   code=process.wait();out=''.join(lines)
   results.append(dict(command=command,exit_code=code,fixture_sha256=hashlib.sha256(source.encode()).hexdigest(),output=out))
   ok=code==0 if not a.const or ordinal==0 else code!=0 and 'cannot prove requires contract'in out and ('1 == 0'in out or '1==0'in out)
   if not ok:break
 unchanged=before==snapshot()and binary_hash==sha(binary)
 receipt=dict(execution_root=str(ROOT.resolve()),stage='constant evaluator'if a.const else'checked interpreter',native_execution=False,cases=[r['name']for r in rows],scenarios=len(rows),controls=len(rows),source_unchanged=unchanged,source_sha256=before,binary=str(binary),binary_sha256=binary_hash,build_source=build_text,build_sha256=hashlib.sha256(build_text.encode()).hexdigest(),elapsed_seconds=time.monotonic()-started,results=results)
 a.record.parent.mkdir(parents=True,exist_ok=True);a.record.write_text(json.dumps(receipt,indent=2,sort_keys=True)+'\n')
 assert unchanged,'Inputs changed during verification'
 assert ok,'Execution or control failed; see receipt'
 print(f'PASS {len(rows)} behavior/control pairs; {receipt["elapsed_seconds"]:.3f}s',flush=True)
if __name__=='__main__':main()
