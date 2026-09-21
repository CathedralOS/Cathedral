#!/usr/bin/env python3
"""Check authored bodies once, then execute all unchanged fixtures and mutations."""
import argparse,hashlib,json,os,re,subprocess,sys,tempfile,time
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
RUNNER_SHA='e6d0aee6b4dddbbf34a60cffbe8f158643cc5c4d100f9e20f0481f75f52890be'
SHARED=ROOT/'tools/ports/acpi/interpreter/execution'
SOURCES=['source/libraries/x86_64/'+n for n in ['build.omg','encrypted_cleanup_branch.omg','encrypted_cleanup_ranges.omg','encrypted_cleanup_recursive.omg','memory_encryption.omg','cleanup_branch.omg','cleanup_ranges.omg','recursive_cleanup.omg','page_entries.omg','pages.omg','addresses.omg']]+['source/drivers/facts/build.omg','source/drivers/facts/x86_page_table_entry.omg']
def snapshot():
 paths={ROOT/p for p in SOURCES};paths.update(p for p in HERE.rglob('*') if p.is_file() and not any(s in p.parts for s in ['target','__pycache__']) and p.name!='checked-verification.json')
 paths.update([SHARED/'checked_runner.rs',SHARED/'runner.Cargo.lock'])
 return {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(paths)}
def adapt(source,name):
 source=source[:source.index('const TEST_RESULT')];imports=re.findall(r'^use [^;]+;',source,re.M);source=re.sub(r'^use [^;]+;\n?','',source,flags=re.M)
 names=re.findall(r'^machine (\w+)\(',source,re.M)
 for local in names:
  if local=='test_result':continue
  source=re.sub(r'(?<!::)\b'+local+r'\(',name+'_'+local+'(',source)
 assert source.count('machine test_result()')==1
 source=source.replace('machine test_result()',f'machine Suite::{name}(&mut self)')
 return imports,source

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--runner',type=Path,default=Path('/tmp/cathedral-acpi-execution-checked/release/cathedral-acpi-checked-runner'));p.add_argument('--record',type=Path,default=HERE/'checked-verification.json');p.add_argument('--match',default='');a=p.parse_args()
 assert hashlib.sha256(a.runner.read_bytes()).hexdigest()==RUNNER_SHA,'Build the pinned Cathedral execution checked runner first'
 subprocess.run([sys.executable,str(HERE/'check.py'),'--host-only'],cwd=ROOT,check=True)
 metadata=json.loads((HERE/'fixtures.json').read_text());extras=json.loads((HERE/'extras.json').read_text());before=snapshot();imports=set();bodies=[];selections=[]
 positives=[r['path'] for r in metadata['batches']]+[r['path'] for r in extras['extras']]
 controls=metadata['controls']+extras['controls']
 counts=[0,0]
 for path,change in [(s,None) for s in positives]+[(c['path'],c) for c in controls]:
  name=(Path(path).stem.replace('-','_')+'_positive')if change is None else change['family'].replace('-','_')+'_control'
  if a.match and a.match not in name:continue
  source=(HERE/path).read_text()
  if change:assert source.count(change['old'])==1;source=source.replace(change['old'],change['new'])
  selected_imports,body=adapt(source,name);imports.update(selected_imports);bodies.append(body);selections.append('Suite::'+name+'='+str(int(change is not None)));counts[int(change is not None)]+=1
 assert selections,'No selected fixtures'
 suite='\n'.join(sorted(imports))+'\n data Suite {}\n'+'\n'.join(bodies)
 with tempfile.TemporaryDirectory(prefix='cathedral-encrypted-cleanup-checked-') as temp:
  work=Path(temp);(work/'main.omg').write_text(suite);(work/'build.omg').write_text((HERE/'build.omg').read_text().replace('../../../source/',str(ROOT/'source')+'/'))
  started=time.monotonic();lines=[]
  process=subprocess.Popen([str(a.runner),str(work/'main.omg'),str(work/'build'),*selections],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'))
  for line in process.stdout:print(line,end='',flush=True);lines.append(line)
  if process.wait():raise SystemExit('Checked interpreter regression failed')
  assert snapshot()==before,'Selected sources/fixtures changed during execution'
  record={'format':'cathedral-encrypted-cleanup-checked-v1','stage':'checked-interpreter execution; native/hardware not run','omega_revision':'eaa7993a23623cd8fabf45350340479c5c9c7879','runner_sha256':RUNNER_SHA,'suite_sha256':hashlib.sha256(suite.encode()).hexdigest(),'positive_fixture_count':counts[0],'control_count':counts[1],'whole_tree_case_count':384 if not a.match else None, 'step_case_count':424 if not a.match else None,'evaluator_step_limit':10000000,'elapsed_seconds':round(time.monotonic()-started,3),'source_sha256':before,'selections':selections,'output':''.join(lines)}
  if not a.match:a.record.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
 print('PASS',counts[0],'positive fixtures and',counts[1],'changed-body controls via checked interpreter; native/hardware NOT RUN')
if __name__=='__main__':main()
