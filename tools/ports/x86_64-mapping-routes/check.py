#!/usr/bin/env python3
"""Check complete detached routes against Rust and actual Omega evaluation."""
import argparse,hashlib,re,subprocess,sys,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def run(*args):subprocess.run([str(x) for x in args],cwd=ROOT,check=True)
def main():
 p=argparse.ArgumentParser();p.add_argument('--omega',type=Path,default=ROOT.parent/'Omega/target/release/omega');p.add_argument('--host-only',action='store_true');p.add_argument('--start',type=int,default=0);p.add_argument('--end',type=int);p.add_argument('--controls-only',action='store_true');args=p.parse_args()
 run(sys.executable,HERE/'generate.py','--check');run(sys.executable,HERE/'map_inventory.py','--check')
 run(sys.executable,ROOT/'tools/ports/inventory.py','check',ROOT/'source/libraries/x86_64/mapping-routes-inventory.json','--checkout',ROOT/'reference_code/rust-osdev/x86_64')
 run('cargo','run','--quiet','--locked','--manifest-path',HERE/'Cargo.toml')
 if args.host_only:return
 compiler=args.omega.resolve();print('Omega SHA-256:',hashlib.sha256(compiler.read_bytes()).hexdigest(),flush=True)
 source=(HERE/'main.omg').read_text();prefix=source[:source.index('machine route_0')]
 blocks=re.findall(r'machine route_\d+\(\)->i32 \{.*?(?=\nmachine route_|\ndata Main)',source,re.S)
 assert len(blocks)==208, f'expected 208 route bodies, got {len(blocks)}'
 build=(HERE/'build.omg').read_text().replace('../../../source/libraries/x86_64',str(ROOT/'source/libraries/x86_64'))
 def fixture(selected):
  body=prefix+'\n'.join(blocks[n] for n in selected)
  body+='\nmachine route_test_result()->i32 {\n'+'\n'.join(f'let pass{i}:bool=route_{n}()==0;' for i,n in enumerate(selected))+'\ntransition '+' && '.join(f'pass{i}' for i in range(len(selected)))+' { true -> (0) _ -> (1) } }\nconst RESULT:i32=route_test_result();\nmachine require_success(value:i32) requires value == 0; {}\ndata Main{}\nmachine Main::main(&mut self){require_success(RESULT);}\n'
  return body
 def evaluate(body,mutation=None):
  if mutation:
   old,new=mutation;assert body.count(old)==1;body=body.replace(old,new)
  with tempfile.TemporaryDirectory(prefix='cathedral-mapping-route-') as directory:
   path=Path(directory);(path/'main.omg').write_text(body);(path/'build.omg').write_text(build)
   r=subprocess.run([str(compiler),'--check',str(path/'main.omg')],cwd=ROOT,capture_output=True,text=True);out=r.stdout+r.stderr
   if mutation:
    if not r.returncode or 'cannot prove requires contract' not in out or '1 == 0' not in out:raise SystemExit('mutation failed:\n'+out)
   elif r.returncode:raise SystemExit('positive failed:\n'+out)
 if not args.controls_only:
  for start in range(args.start,min(args.end or len(blocks),len(blocks)),3):
   selected=list(range(start,min(start+3,args.end or len(blocks),len(blocks))));evaluate(fixture(selected));print('PASS routes',selected,flush=True)
 if args.end is None:
  extra=(HERE/'extras.omg').read_text()
  prefix_extra=extra[:extra.index('machine invalids')]
  extra_blocks=re.findall(r'machine (?:invalids|mismatch|bad_supply|replaced_capture)\(\)->i32 \{.*?(?=\nmachine (?:invalids|mismatch|bad_supply|replaced_capture)|\Z)',extra,re.S)
  for name,block in zip(['invalids','mismatch','bad_supply','replaced_capture'],extra_blocks):
   body=prefix_extra+block+f'\nconst RESULT:i32={name}();\nmachine require_success(value:i32) requires value == 0; {{}}\ndata Main{{}}\nmachine Main::main(&mut self){{require_success(RESULT);}}\n'
   evaluate(body);print('PASS extra',name,flush=True)
   if name=='mismatch':evaluate(body,('expected==4096','expected==8192'));print('PASS mismatch body mutation',flush=True)
   if name=='replaced_capture':evaluate(body,('value.edits.zero_mask==2','value.edits.zero_mask==0'));print('PASS zero request body mutation',flush=True)
  for n,old,new in [(0,'value.edits.write_mask == 3','value.edits.write_mask == 0'),(1,'result in ChildPlan::AllocationFailed','result in ChildPlan::HugeParent')]:
   evaluate(fixture([n]),(old,new));print('PASS body mutation',n,flush=True)
 print('PASS selected complete-route scenarios and controls',flush=True)
if __name__=='__main__':main()
