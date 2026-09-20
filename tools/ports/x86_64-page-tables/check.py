#!/usr/bin/env python3
"""Execute bounded table/captured-walk behavior and actual pinned mapper witnesses."""
import argparse,hashlib,json,re,subprocess,sys,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def run(*args):subprocess.run([str(x) for x in args],cwd=ROOT,check=True)
def main():
 p=argparse.ArgumentParser();p.add_argument('--omega',type=Path,default=ROOT.parent/'Omega/target/release/omega');p.add_argument('--host-only',action='store_true');p.add_argument('--address-only',action='store_true');args=p.parse_args()
 run(sys.executable,HERE/'generate.py','--check')
 run(sys.executable,HERE/'map_inventory.py','--check')
 run(sys.executable,ROOT/'tools/ports/inventory.py','check',ROOT/'source/libraries/x86_64/tables-inventory.json','--checkout',ROOT/'reference_code/rust-osdev/x86_64')
 run('cargo','run','--quiet','--locked','--manifest-path',HERE/'Cargo.toml')
 if args.host_only:return
 compiler=args.omega.resolve();print('Omega SHA-256:',hashlib.sha256(compiler.read_bytes()).hexdigest(),flush=True)
 build=(HERE/'build.omg').read_text().replace('../../../source/libraries/x86_64',str(ROOT/'source/libraries/x86_64')).replace('../../../source/drivers/facts',str(ROOT/'source/drivers/facts'))
 def evaluate(body,mutation=None):
  if mutation:
   old,new=mutation;assert body.count(old)==1;body=body.replace(old,new)
  with tempfile.TemporaryDirectory(prefix='cathedral-table-check-') as directory:
   path=Path(directory);(path/'main.omg').write_text(body);(path/'build.omg').write_text(build)
   r=subprocess.run([str(compiler),'--check',str(path/'main.omg')],cwd=ROOT,capture_output=True,text=True);out=r.stdout+r.stderr
   if mutation:
    if not r.returncode or 'cannot prove requires contract' not in out or '1 == 0' not in out:raise SystemExit('mutation did not compute failure:\n'+out)
   elif r.returncode:raise SystemExit('positive failed:\n'+out)
 if args.address_only:
  evaluate((HERE/'address_probe.omg').read_text());evaluate((HERE/'address_probe.omg').read_text(),('equal(address,0xcafe0eef)','equal(address,0)'));print('PASS translated-address body and mutation',flush=True);return
 for name in ['main.omg','empty_probe.omg','last_probe.omg','clear_probe.omg','walk_probe.omg','address_probe.omg']:
  evaluate((HERE/name).read_text());print('PASS body:',name,flush=True)
 source=(HERE/'translations.omg').read_text();prefix=source[:source.index('machine translation_case_0')]
 blocks=re.findall(r'machine translation_case_\d+\(\) -> i32 \{.*?(?=\nmachine translation_case_|\ndata Main)',source,re.S)
 def selected(indices):
  body=prefix+'\n'.join(blocks[i] for i in indices)
  body+='\nmachine translation_test_result()->i32 {\n'+'\n'.join(f'let pass{i}:bool=translation_case_{i}()==0;' for i in indices)+'\ntransition '+' && '.join(f'pass{i}' for i in indices)+' { true -> (0) _ -> (1) } }\nconst RESULT:i32=translation_test_result();\nmachine require_success(value:i32) requires value == 0; {}\ndata Main{}\nmachine Main::main(&mut self){require_success(RESULT);}\n'
  return body
 for i in range(0,len(blocks),4):
  evaluate(selected(list(range(i,min(i+4,len(blocks))))));print('PASS translation cases',i,'through',min(i+3,len(blocks)-1),flush=True)
 controls=[('main.omg','equal(last, 0x8000000000000000)','equal(last, 0)'),('last_probe.omg','written && !empty','written && empty'),('clear_probe.omg','equal(last,0)','equal(last,1)'),('walk_probe.omg','level == 2 && expected == 8192','level == 3 && expected == 8192'),('address_probe.omg','equal(address,0xcafe0eef)','equal(address,0)')]
 for name,old,new in controls:evaluate((HERE/name).read_text(),(old,new));print('PASS mutation:',name,flush=True)
 evaluate(selected([7]),('flags == 512','flags == 0'));print('PASS non-present-leaf body mutation',flush=True)
 print('PASS detached table and translation semantics; no live mapping/ABI claim.')
if __name__=='__main__':main()
