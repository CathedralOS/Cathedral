#!/usr/bin/env python3
"""Execute actual checked generic Omega bodies; each control changes an expectation."""
import argparse,hashlib,json,os,subprocess,tempfile,time
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT
def snapshot():
 paths=sorted((ROOT/'source/libraries/acpi').rglob('*.omg'))+sorted(HERE.glob('*.py'))
 return {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in paths}
RUNNER=Path('/tmp/cathedral-acpi-generic-checked/release/cathedral-acpi-checked-runner')
def main():
 p=argparse.ArgumentParser();p.add_argument('--match',default='');p.add_argument('--record',type=Path);p.add_argument('--const',action='store_true');p.add_argument('--runner',type=Path,default=RUNNER);a=p.parse_args()
 rows=[r for r in fixtures.cases()if any(k in r['name']for k in a.match.split(','))];assert rows
 with tempfile.TemporaryDirectory(prefix='cathedral-acpi-generic-')as directory:
  root=Path(directory);source,names=fixtures.render(rows)
  if a.const:
   assert len(rows)==1
   source+='\nconst TEST_RESULT:i32=const_entry();\nmachine const_entry()->i32 {let mut suite:Suite=Suite {};let result:i32=suite.'+rows[0]['name']+'_positive();result}\nmachine require_success(value:i32) requires value==0; {}\ndata Main {}\nmachine Main::main(&mut self){require_success(TEST_RESULT);}\n'
  (root/'main.omg').write_text(source)
  build='machine build(builder:&mut Build){builder.package("cathedral-acpi-generic-fixtures");builder.freestanding=true;'
  for alias,relative in [('aml','source/libraries/acpi/aml'),('execution','source/libraries/acpi/interpreter/execution'),('integer_helpers','source/libraries/acpi/interpreter'),('pipeline','source/libraries/acpi/pipeline')]:build+=f'builder.depend_as("{alias}",Source::Path {{location:"{ROOT/relative}"}});'
  (root/'build.omg').write_text(build+'}\n')
  before=snapshot();binary=Path('/tmp/cathedral-omega-eaa7993/release/omega') if a.const else a.runner;binary_before=hashlib.sha256(binary.read_bytes()).hexdigest();start=time.monotonic();env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000')
  if a.const:command=[str(binary),'--check',str(root/'main.omg')]
  else:command=[str(binary),str(root/'main.omg'),str(root/'build'),*names]
  process=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=env);lines=[]
  for line in process.stdout:print(line,end='',flush=True);lines.append(line)
  code=process.wait();unchanged=before==snapshot() and binary_before==hashlib.sha256(binary.read_bytes()).hexdigest()
  if a.record:
   paths=sorted((ROOT/'source/libraries/acpi').rglob('*.omg'))+sorted(HERE.glob('*.py'))
   receipt={'execution_root':str(ROOT.resolve()),'build_sha256':hashlib.sha256((build+'}\n').encode()).hexdigest(),'stage':'constant evaluator'if a.const else'checked interpreter','scenarios':len(rows),'controls':0 if a.const else len(rows),'elapsed_seconds':time.monotonic()-start,'exit_code':code,'binary_sha256':binary_before,'binary':str(binary),'command':command,'source_unchanged':unchanged,'native_execution':False,'build_source':build+'}\n','fixture_sha256':hashlib.sha256(source.encode()).hexdigest(),'source_sha256':before,'cases':[r['name']for r in rows],'output':''.join(lines)}
   a.record.write_text(json.dumps(receipt,indent=2,sort_keys=True)+'\n')
  assert unchanged,'Source or runner changed during verification'
  if code:raise SystemExit(code)
 print('PASS',len(rows),'generic cases and controls'if not a.const else'const case')
if __name__=='__main__':main()
