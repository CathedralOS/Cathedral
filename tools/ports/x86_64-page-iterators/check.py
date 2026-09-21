#!/usr/bin/env python3
"""Pinned public iterators and checked Omega bodies, with immutable hash records."""
import argparse,hashlib,json,os,subprocess,sys,tempfile,time
from pathlib import Path
import generate
HERE=generate.HERE;ROOT=generate.ROOT;UP=ROOT/'reference_code/rust-osdev/x86_64'
SHARED=ROOT/'tools/ports/acpi/interpreter/execution'
RUNNER=Path('/tmp/cathedral-acpi-execution-checked/release/cathedral-acpi-checked-runner')
RUNNER_SHA='e6d0aee6b4dddbbf34a60cffbe8f158643cc5c4d100f9e20f0481f75f52890be'
OMEGA=Path('/tmp/cathedral-omega-eaa7993/release/omega')
def snapshot():
 paths={ROOT/'source/libraries/x86_64'/name for name in ['build.omg','page_iterators.omg','encrypted_frames.omg','pages.omg','addresses.omg','memory_encryption.omg','page_entries.omg']}
 paths.update([ROOT/'source/drivers/facts/build.omg',ROOT/'source/drivers/facts/x86_page_table_entry.omg',SHARED/'checked_runner.rs',SHARED/'runner.Cargo.lock'])
 paths.update(p for p in HERE.rglob('*')if p.is_file() and not any(s in p.parts for s in ['target','__pycache__']) and 'verification'not in p.name)
 return {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in sorted(paths)}
def main():
 p=argparse.ArgumentParser();p.add_argument('--host-only',action='store_true');p.add_argument('--match',default='');p.add_argument('--const',dest='constant',action='store_true');a=p.parse_args()
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=UP,text=True).strip()=='cc35c876d3badb57df54a66e22f7768a52be95f2'
 for name in ['generate_inputs.py','generate.py','generate_inventory.py']:subprocess.run([sys.executable,str(HERE/name),'--check'],check=True,cwd=ROOT)
 actual=subprocess.check_output(['cargo','run','--quiet','--offline','--locked','--release','--manifest-path',str(HERE/'Cargo.toml')],cwd=ROOT)
 assert actual==(HERE/'reference.jsonl').read_bytes(),'actual pinned public iterator observations differ'
 if a.host_only:return
 all_groups=generate.groups();groups=[g for g in all_groups if a.match in g['name']];assert groups
 before=snapshot();started=time.monotonic();outputs=[];selections=[];suites=[]
 with tempfile.TemporaryDirectory(prefix='cathedral-page-iterators-')as temp:
  work=Path(temp);(work/'build.omg').write_text((HERE/'build.omg').read_text().replace('../../../source/',str(ROOT/'source')+'/'))
  if a.constant:
   # Representative normal/gap/MAX groups, each paired with a changed body.
   groups=[all_groups[n]for n in [0,5,11,22,34,49]] if not a.match else groups
   for group in groups:
    for control in [False,True]:
     source=generate.const_source(group,control);(work/'main.omg').write_text(source);suites.append(hashlib.sha256(source.encode()).hexdigest())
     result=subprocess.run([str(OMEGA),'--check',str(work/'main.omg')],cwd=ROOT,text=True,capture_output=True);output=result.stdout+result.stderr
     assert (result.returncode!=0 and 'cannot prove requires contract'in output and '1 == 0'in output)if control else result.returncode==0,output
     label=group['name']+('_control'if control else'_positive');print('PASS const',label,flush=True);outputs.append(label+'\n'+output);selections.append(label)
   binary_sha=hashlib.sha256(OMEGA.read_bytes()).hexdigest();stage='actual Omega constant evaluation; native not run';record='constant-verification.json'
  else:
   assert hashlib.sha256(RUNNER.read_bytes()).hexdigest()==RUNNER_SHA
   source=generate.HEAD+'data Suite {}\n'
   for group in groups:
    for control in [False,True]:source+=generate.render(group,control);selections.append('Suite::'+group['name']+('_control=1'if control else'_positive=0'))
   (work/'main.omg').write_text(source);suites.append(hashlib.sha256(source.encode()).hexdigest())
   process=subprocess.Popen([str(RUNNER),str(work/'main.omg'),str(work/'build'),*selections],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'))
   for line in process.stdout:print(line,end='',flush=True);outputs.append(line)
   assert process.wait()==0,'checked iterator regression failed'
   binary_sha=RUNNER_SHA;stage='checked-interpreter execution; native not run';record='checked-verification.json'
  assert snapshot()==before,'selected source/harness input changed during verification'
  if not a.match:
   (HERE/record).write_text(json.dumps(dict(format='cathedral-page-iterators-v1',stage=stage,omega_revision='eaa7993a23623cd8fabf45350340479c5c9c7879',binary_sha256=binary_sha,source_sha256=before,suite_sha256=suites,case_count=sum(len(g['rows'])for g in groups),group_count=len(groups),control_count=len(groups),elapsed_seconds=round(time.monotonic()-started,3),selections=selections,output=''.join(outputs)),indent=2,sort_keys=True)+'\n')
 print('PASS',sum(len(g['rows'])for g in groups),'iterator cases;',len(groups),'body controls',flush=True)
if __name__=='__main__':main()
