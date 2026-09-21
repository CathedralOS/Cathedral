#!/usr/bin/env python3
"""Replay the 18 parser bodies and shared-type consumers after owner relocation."""
import argparse,hashlib,json,os,re,subprocess,tempfile,time
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def snapshot():
 paths=set((ROOT/'source/libraries/acpi').rglob('*.omg'))
 paths.update((HERE/'cases').glob('*.omg'))
 paths.update(HERE/name for name in ['cases.json','migration.py','migration-boundaries.omg'])
 paths.update(ROOT/'tools/ports/acpi/interpreter/execution'/name for name in ['checked_runner.rs','runner.Cargo.lock'])
 return {str(p.relative_to(ROOT)):sha(p)for p in sorted(paths)}
def fixture():
 imports=set();helpers={};body=[];selections=[]
 for name,item in json.loads((HERE/'cases.json').read_text()).items():
  source=(HERE/'cases'/f'{name}.omg').read_text();start=source.index('machine test()');end=source.index('const TEST_RESULT');original=source[start:end]
  imports.update(re.findall(r'^use [^;]+;',source[:start],re.M))
  for found in re.finditer(r'^machine (\w+)\(',source[:start],re.M):
   opening=source.index('{',found.start());depth=1;closing=opening+1
   while depth:depth+=(source[closing]=='{')-(source[closing]=='}');closing+=1
   text=source[found.start():closing]
   assert found[1]not in helpers or helpers[found[1]]==text
   helpers[found[1]]=text
  assert original.count(item['mutation'][0])==1
  for control in [False,True]:
   machine='Suite::'+name.replace('-','_')+('_control'if control else'_positive')
   selections.append(machine+'='+str(int(control)))
   changed=original.replace(*item['mutation'])if control else original
   body.append(changed.replace('machine test()',f'machine {machine}(&mut self)',1))
 boundary=(HERE/'migration-boundaries.omg').read_text();imports.update(re.findall(r'^use [^;]+;',boundary,re.M));boundary=re.sub(r'^use [^;]+;\n','',boundary,flags=re.M)
 for name,positive,negative in [('consumer_round_trip',171,170),('field_boundary',5,6)]:
  for control in [False,True]:
   machine='Suite::'+name+('_control'if control else'_positive');selections.append(machine+'='+str(int(control)))
   body.append(f'machine {machine}(&mut self)->u64 {{let result:u64={name}({negative if control else positive});result}}')
 return '\n'.join(sorted(imports))+ '\n'+'\n'.join(helpers.values())+'\n'+boundary+'\ndata Suite {}\n'+'\n'.join(body)+'\n',selections

def build():
 text='machine build(builder:&mut Build){builder.package("field-owner-migration");builder.freestanding=true;'
 for alias,folder in [('aml','aml'),('fields','aml/fields'),('reads','field_values'),('writes','field_writes'),('access','field_access'),('integers','interpreter'),('execution','interpreter/execution')]:text+='builder.depend_as("'+alias+'",Source::Path {location:"'+str(ROOT/'source/libraries/acpi'/folder)+'"});'
 return text+'}\n'
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--runner',type=Path,default=Path('/tmp/cathedral-acpi-generic-checked/release/cathedral-acpi-checked-runner'));p.add_argument('--record',type=Path,default=HERE/'migration-verification.json');p.add_argument('--verify',action='store_true');a=p.parse_args()
 source,selections=fixture();build_source=build();before=snapshot();binary_hash=sha(a.runner)
 if not a.verify:
  started=time.monotonic()
  with tempfile.TemporaryDirectory(prefix='cathedral-field-owner-')as folder:
   work=Path(folder);(work/'build.omg').write_text(build_source);(work/'main.omg').write_text(source);command=[str(a.runner),str(work/'main.omg'),str(work/'build'),*selections];lines=[]
   process=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'))
   for line in process.stdout:print(line,end='',flush=True);lines.append(line)
   code=process.wait()
  record=dict(stage='checked interpreter',native_execution=False,source_sha256=before,source_unchanged=before==snapshot(),binary=str(a.runner),binary_sha256=binary_hash,fixture_sha256=hashlib.sha256(source.encode()).hexdigest(),build_source=build_source,build_sha256=hashlib.sha256(build_source.encode()).hexdigest(),command=command,exit_code=code,output=''.join(lines),elapsed_seconds=time.monotonic()-started,selections=selections)
  a.record.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 record=json.loads(a.record.read_text());assert record['source_unchanged']and record['source_sha256']==before
 assert record['binary_sha256']==binary_hash==sha(record['binary'])
 assert record['fixture_sha256']==hashlib.sha256(source.encode()).hexdigest()and record['build_source']==build_source and record['build_sha256']==hashlib.sha256(build_source.encode()).hexdigest()
 assert record['selections']==selections and record['exit_code']==0,record['output']
 observed=re.findall(r'^PASS (\S+) expected=(\d+) observed=(\d+) error=None usage=',record['output'],re.M)
 assert len(observed)==len(selections)and [n+'='+e for n,e,o in observed if e==o]==selections
 print('PASS',len(selections)//2,'owner-migration scenario/control pairs'+(' retained-record verification'if a.verify else''))
if __name__=='__main__':main()
