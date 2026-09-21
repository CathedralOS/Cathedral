#!/usr/bin/env python3
"""Check all inert Field binding boundaries against exact source inputs."""
import argparse,hashlib,json,os,re,subprocess,tempfile,time
from pathlib import Path
import fixtures
HERE=fixtures.HERE;ROOT=fixtures.ROOT
def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def digest(text):return hashlib.sha256(text.encode()).hexdigest()
def snapshot():
 paths=set()
 for folder in ['source/libraries/acpi/aml','source/libraries/acpi/interpreter','source/libraries/acpi/interpreter/execution']:
  paths.update((ROOT/folder).glob('*.omg'))
 paths.update(p for p in HERE.iterdir()if p.suffix in ['.py','.omg','.txt'])
 paths.add(HERE.parent/'field-namespace/fixtures.py')
 paths.update(ROOT/'tools/ports/acpi/interpreter/execution'/name for name in ['checked_runner.rs','runner.Cargo.lock'])
 return {str(p.relative_to(ROOT)):sha(p)for p in sorted(paths)}
def build():
 text='machine build(builder:&mut Build){builder.package("field-binding-boundaries");builder.freestanding=true;'
 for alias,folder in [('aml','aml'),('execution','interpreter/execution'),('integers','interpreter')]:text+='builder.depend_as("'+alias+'",Source::Path {location:"'+str(ROOT/'source/libraries/acpi'/folder)+'"});'
 return text+'}\n'
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--runner',type=Path,default=Path('/tmp/cathedral-acpi-generic-checked/release/cathedral-acpi-checked-runner'));p.add_argument('--verify',action='store_true');p.add_argument('--record',type=Path,default=HERE/'checked-verification.json');a=p.parse_args()
 source,names=fixtures.fixture();assert (HERE/'main.omg').read_text()==source;assert (HERE/'selections.txt').read_text().splitlines()==names
 before=snapshot();build_source=build();binary_hash=sha(a.runner)
 if not a.verify:
  started=time.monotonic()
  with tempfile.TemporaryDirectory(prefix='cathedral-field-bindings-')as folder:
   work=Path(folder);(work/'build.omg').write_text(build_source);(work/'main.omg').write_text(source);command=[str(a.runner),str(work/'main.omg'),str(work/'build'),*names];lines=[]
   process=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'))
   for line in process.stdout:print(line,end='',flush=True);lines.append(line)
   code=process.wait()
  record=dict(stage='checked interpreter',execution_root=str(ROOT.resolve()),native_execution=False,source_sha256=before,source_unchanged=before==snapshot(),binary=str(a.runner),binary_sha256=binary_hash,fixture_sha256=digest(source),build_source=build_source,build_sha256=digest(build_source),command=command,exit_code=code,output=''.join(lines),elapsed_seconds=time.monotonic()-started,selections=names)
  a.record.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 record=json.loads(a.record.read_text());assert record['source_unchanged']and record['source_sha256']==before
 original_build=build_source.replace(str(ROOT),record['execution_root'])
 assert record['binary_sha256']==sha(record['binary'])and record['fixture_sha256']==digest(source)and record['build_source']==original_build and record['build_sha256']==digest(original_build)
 assert record['exit_code']==0 and record['selections']==names,record['output']
 observed=re.findall(r'^PASS (\S+) expected=(\d+) observed=(\d+) error=None usage=',record['output'],re.M)
 assert len(observed)==len(names)and [name+'='+expected for name,expected,actual in observed if actual==expected]==names
 print('PASS six Field binding behavior/control pairs'+(' (retained record only)'if a.verify else''))
if __name__=='__main__':main()
