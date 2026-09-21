#!/usr/bin/env python3
"""Actual pinned frame probes and Omega numeric/iterator bodies, with body controls."""
import argparse,hashlib,json,os,shutil,subprocess,tempfile,time
from pathlib import Path
import generate,compact
HERE=generate.HERE;ROOT=generate.ROOT;SHARED=ROOT/'tools/ports/acpi/interpreter/execution'
def snapshot():
 sources={ROOT/'source/libraries/x86_64'/name for name in ['encrypted_frames.omg','pages.omg','addresses.omg','memory_encryption.omg','page_entries.omg','build.omg']}
 sources.update(p for p in HERE.iterdir()if p.suffix in ['.py','.omg','.json','.jsonl','.toml','.lock']and'verification'not in p.name)
 sources.update([ROOT/'source/drivers/facts/build.omg',ROOT/'source/drivers/facts/x86_page_table_entry.omg']);sources.add(HERE/'src/main.rs');sources.update([SHARED/'checked_runner.rs',SHARED/'runner.Cargo.lock'])
 return {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in sorted(sources)}
def reference_provenance():
 upstream=ROOT/'reference_code/rust-osdev/x86_64'
 revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=upstream,text=True).strip()
 assert revision=='cc35c876d3badb57df54a66e22f7768a52be95f2',revision
 assert not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=upstream,text=True).strip()
 paths=['src/structures/paging/frame.rs','src/structures/paging/page.rs','src/structures/mem_encrypt.rs','src/addr.rs','Cargo.toml','LICENSE-MIT','LICENSE-APACHE']
 return dict(revision=revision,license='MIT OR Apache-2.0',source_sha256={path:hashlib.sha256((upstream/path).read_bytes()).hexdigest()for path in paths},rustc=subprocess.check_output(['rustc','--version'],text=True).strip(),profile='release, memory_encryption feature, x86-64 usize, isolated child process per configuration sequence',public_call_count=len((HERE/'reference.jsonl').read_text().splitlines()))
def main():
 p=argparse.ArgumentParser();p.add_argument('--match',default='');p.add_argument('--record',type=Path);p.add_argument('--host-only',action='store_true');a=p.parse_args()
 provenance=reference_provenance();actual=subprocess.check_output(['cargo','run','--quiet','--offline','--locked','--release','--manifest-path',str(HERE/'Cargo.toml')],cwd=ROOT);assert actual==(HERE/'reference.jsonl').read_bytes(),'Pinned Rust observation drift'
 subprocess.run(['python3',str(HERE/'generate.py'),'--check'],check=True)
 if a.host_only:return
 omega=ROOT.parent/'Omega';revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=omega,text=True).strip();assert revision=='eaa7993a23623cd8fabf45350340479c5c9c7879';assert not subprocess.check_output(['git','status','--porcelain'],cwd=omega,text=True).strip()
 rows=[row for row in generate.groups()if any(part in row['name']for part in a.match.split(','))];assert rows;before=snapshot()
 with tempfile.TemporaryDirectory(prefix='cathedral-encrypted-frames-')as directory:
  work=Path(directory);manifest=['[package]','name="cathedral-acpi-checked-runner"','version="0.1.0"','edition="2024"','[dependencies]']
  for name,path in {'checked-interpreter':'psi/semantics/checked-interpreter','package-manager':'omega/packages/manager','target':'omega/representations/target'}.items():manifest.append(f'{name}={{path="{omega}/omega-rust/{path}"}}')
  manifest+=['[[bin]]','name="cathedral-acpi-checked-runner"',f'path="{SHARED}/checked_runner.rs"'];(work/'Cargo.toml').write_text('\n'.join(manifest)+'\n');(work/'Cargo.lock').write_bytes((SHARED/'runner.Cargo.lock').read_bytes());target=Path('/tmp/cathedral-acpi-execution-checked')
  subprocess.run([shutil.which('mbx')or'cargo','build','--offline','--locked','--release','--manifest-path',str(work/'Cargo.toml'),'--target-dir',str(target)],cwd=omega,check=True)
  source=compact.HEAD;selections=[]
  for row in rows:
   for control in [False,True]:
    machine='Suite::'+row['name']+('_control'if control else'_positive');source+=compact.render(row,control);selections.append(machine+'='+str(int(control)))
  (work/'main.omg').write_text(source);(work/'build.omg').write_text((HERE/'build.omg').read_text().replace('../../../source/',str(ROOT/'source')+'/'))
  runner=target/'release/cathedral-acpi-checked-runner';start=time.monotonic();lines=[];process=subprocess.Popen([str(runner),str(work/'main.omg'),str(work/'build'),*selections],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'))
  for line in process.stdout:print(line,end='',flush=True);lines.append(line)
  if process.wait():raise SystemExit('Encrypted frame semantics failed')
  assert snapshot()==before,'Sources changed during checks'
  if a.record:a.record.write_text(json.dumps({'format':'cathedral-encrypted-frames-checked-v1','reference':provenance,'stage':'checked-interpreter execution; native/hardware not run','omega_revision':revision,'runner_sha256':hashlib.sha256(runner.read_bytes()).hexdigest(),'evaluator_step_limit':10000000,'groups':[row['name']for row in rows],'case_count':sum(len(row['cases'])for row in rows),'group_count':len(rows),'control_count':len(rows),'elapsed_seconds':round(time.monotonic()-start,3),'source_sha256':before,'output':''.join(lines)},indent=2,sort_keys=True)+'\n')
 print(f'PASS {sum(len(row["cases"])for row in rows)} frame/cursor cases in {len(rows)} groups + {len(rows)} body controls. Native/hardware NOT RUN.')
if __name__=='__main__':main()
