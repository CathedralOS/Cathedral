#!/usr/bin/env python3
"""Execute actual PTE codecs/index helpers and compare pinned Rust witnesses."""
import argparse,hashlib,json,subprocess,sys,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def run(*args):subprocess.run([str(x) for x in args],cwd=ROOT,check=True)
def main():
 p=argparse.ArgumentParser();p.add_argument('--omega',type=Path,default=ROOT.parent/'Omega/target/release/omega');p.add_argument('--host-only',action='store_true');args=p.parse_args()
 run(sys.executable,HERE/'generate.py','--check')
 run(sys.executable,HERE/'map_inventory.py','--check')
 run(sys.executable,ROOT/'tools/ports/inventory.py','check',ROOT/'source/libraries/x86_64/page-entries-inventory.json','--checkout',ROOT/'reference_code/rust-osdev/x86_64')
 run('cargo','+nightly-2026-09-04','run','--quiet','--locked','--manifest-path',HERE/'Cargo.toml')
 run('cargo','+nightly-2026-09-04','test','--quiet','--manifest-path',ROOT/'reference_code/rust-osdev/x86_64/Cargo.toml','--no-default-features','--features','step_trait','--lib','structures::paging::page_table::tests')
 if args.host_only:return
 compiler=args.omega.resolve();print('Omega SHA-256:',hashlib.sha256(compiler.read_bytes()).hexdigest(),flush=True)
 run(sys.executable,ROOT/'tools/x86-page-table-layout-canary/check-schema.py')
 source=(HERE/'main.omg').read_text();prefix=source[:source.index('machine case_0(')]
 cases=json.loads((HERE/'cases.json').read_text())
 build=(HERE/'build.omg').read_text().replace('../../../source/libraries/x86_64',str(ROOT/'source/libraries/x86_64')).replace('../../../source/drivers/facts',str(ROOT/'source/drivers/facts'))
 def evaluate(indices,mutation=None):
  body=prefix+'\n'.join(f'machine case_{i}() -> i32 {{ {cases[i]["body"]} }}' for i in indices)
  if mutation:
   old,new=mutation;assert body.count(old)==1;body=body.replace(old,new)
  body+='\nmachine entry_test_result() -> i32 {\n'+'\n'.join(f'let value{i}: bool = case_{i}() == 0;' for i in indices)+'\ntransition '+' && '.join(f'value{i}' for i in indices)+' { true -> (0) _ -> (1) } }\nconst RESULT: i32 = entry_test_result();\nmachine require_success(value: i32) requires value == 0; {}\ndata Main {}\nmachine Main::main(&mut self) { require_success(RESULT); }\n'
  with tempfile.TemporaryDirectory(prefix='cathedral-x86-entries-') as directory:
   path=Path(directory);(path/'main.omg').write_text(body);(path/'build.omg').write_text(build)
   r=subprocess.run([str(compiler),'--check',str(path/'main.omg')],cwd=ROOT,capture_output=True,text=True);out=r.stdout+r.stderr
   if mutation:
    if r.returncode==0 or 'cannot prove requires contract' not in out or '1 == 0' not in out:raise SystemExit('mutation failed:\n'+out)
   elif r.returncode:raise SystemExit('positive failed:\n'+out)
 for start in range(0,len(cases),8):
  evaluate(list(range(start,min(start+8,len(cases)))));print('PASS cases',start,'through',min(start+7,len(cases)-1),flush=True)
 controls=[(0,'value.present == false','value.present == true'),(1,'value.protection_key == 15','value.protection_key == 14')]
 for i,old,new in controls:evaluate([i],(old,new));print('PASS body control:',old,flush=True)
 i=next(i for i,c in enumerate(cases) if 'index_overflowing(511, 1, false)' in c['body'])
 evaluate([i],('value.overflow == true','value.overflow == false'));print('PASS overflow body control',flush=True)
 print('PASS actual Omega bodies and mutation controls; no native ABI/access claim.')
if __name__=='__main__':main()
