#!/usr/bin/env python3
"""Evaluate actual detached mapping-plan bodies and pinned Rust effects."""
import argparse,hashlib,re,subprocess,sys,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def run(*args):subprocess.run([str(x) for x in args],cwd=ROOT,check=True)
def main():
 p=argparse.ArgumentParser();p.add_argument('--omega',type=Path,default=ROOT.parent/'Omega/target/release/omega');p.add_argument('--host-only',action='store_true');args=p.parse_args()
 run(sys.executable,HERE/'generate.py','--check')
 run(sys.executable,HERE/'map_inventory.py','--check')
 run(sys.executable,ROOT/'tools/ports/inventory.py','check',ROOT/'source/libraries/x86_64/mapping-plans-inventory.json','--checkout',ROOT/'reference_code/rust-osdev/x86_64')
 run('cargo','run','--quiet','--locked','--manifest-path',HERE/'Cargo.toml')
 if args.host_only:return
 compiler=args.omega.resolve();print('Omega SHA-256:',hashlib.sha256(compiler.read_bytes()).hexdigest(),flush=True)
 source=(HERE/'main.omg').read_text();prefix=source[:source.index('machine leaf_0')]
 blocks=re.findall(r'machine (?:leaf|child)_\d+\(\)->i32 \{.*?(?=\nmachine (?:leaf|child)_|\ndata Main)',source,re.S)
 names=[re.search(r'machine (\w+)\(',block)[1] for block in blocks]
 build=(HERE/'build.omg').read_text().replace('../../../source/libraries/x86_64',str(ROOT/'source/libraries/x86_64'))
 def fixture(selected):
  body=prefix+'\n'.join(blocks[names.index(name)] for name in selected)
  body+='\nmachine plan_test_result()->i32 {\n'+'\n'.join(f'let pass{i}:bool={name}()==0;' for i,name in enumerate(selected))+'\ntransition '+' && '.join(f'pass{i}' for i in range(len(selected)))+' { true -> (0) _ -> (1) } }\nconst RESULT:i32=plan_test_result();\nmachine require_success(value:i32) requires value == 0; {}\ndata Main{}\nmachine Main::main(&mut self){require_success(RESULT);}\n'
  return body
 def evaluate(body,mutation=None):
  if mutation:
   old,new=mutation;assert body.count(old)==1;body=body.replace(old,new)
  with tempfile.TemporaryDirectory(prefix='cathedral-mapping-plan-') as directory:
   path=Path(directory);(path/'main.omg').write_text(body);(path/'build.omg').write_text(build)
   r=subprocess.run([str(compiler),'--check',str(path/'main.omg')],cwd=ROOT,capture_output=True,text=True);out=r.stdout+r.stderr
   if mutation:
    if not r.returncode or 'cannot prove requires contract' not in out or '1 == 0' not in out:raise SystemExit('mutation failed:\n'+out)
   elif r.returncode:raise SystemExit('positive failed:\n'+out)
 for start in range(0,len(names),4):
  selected=names[start:start+4];evaluate(fixture(selected));print('PASS',', '.join(selected),flush=True)
 extra=(HERE/'extras.omg').read_text();evaluate(extra);print('PASS malformed inputs and parent flags',flush=True)
 for name,old,new in [('child_0','zero == true','zero == false'),('child_6','value.word == 4225','value.word == 4097'),('leaf_10','value.write == true','value.write == false')]:
  evaluate(fixture([name]),(old,new));print('PASS body mutation:',name,flush=True)
 evaluate(extra,('flags == 7','flags == 0'));print('PASS parent-flags body mutation',flush=True)
 print('PASS pure mapping decisions; live mutation and flush settlement remain separate.')
if __name__=='__main__':main()
