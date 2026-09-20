#!/usr/bin/env python3
"""Demand actual cleanup range steps and pinned whole-tree outcomes."""
import argparse,hashlib,json,re,subprocess,sys,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def run(*args):subprocess.run([str(x)for x in args],cwd=ROOT,check=True)
def main():
 p=argparse.ArgumentParser();p.add_argument('--omega',type=Path,default=ROOT.parent/'Omega/target/release/omega');p.add_argument('--host-only',action='store_true');p.add_argument('--start',type=int,default=0);p.add_argument('--end',type=int);p.add_argument('--controls-only',action='store_true');args=p.parse_args()
 run(sys.executable,HERE/'generate.py','--check');run(sys.executable,HERE/'map_inventory.py','--check')
 run(sys.executable,ROOT/'tools/ports/inventory.py','check',ROOT/'source/libraries/x86_64/cleanup-ranges-inventory.json','--checkout',ROOT/'reference_code/rust-osdev/x86_64')
 run('cargo','run','--quiet','--locked','--manifest-path',HERE/'Cargo.toml')
 if args.host_only:return
 compiler=args.omega.resolve();print('Omega SHA-256:',hashlib.sha256(compiler.read_bytes()).hexdigest(),flush=True)
 source=(HERE/'main.omg').read_text();prefix=source[:source.index('machine range_0')]
 blocks=re.findall(r'machine range_\d+\(\)->i32 \{.*?(?=\nmachine range_|\ndata Main)',source,re.S)
 assert len(blocks)==len(json.loads((HERE/'cases.json').read_text())['steps'])
 build=(HERE/'build.omg').read_text().replace('../../../source/libraries/x86_64',str(ROOT/'source/libraries/x86_64')).replace('../../../source/drivers/facts',str(ROOT/'source/drivers/facts'))
 suffix='\nconst RESULT:i32=test_result();\nmachine require_success(value:i32) requires value == 0; {}\ndata Main{}\nmachine Main::main(&mut self){require_success(RESULT);}\n'
 def fixture(selected):
  body=prefix+'\n'.join(blocks[n]for n in selected)
  body+='\nmachine test_result()->i32 {\n'+'\n'.join(f'let pass{i}:bool=range_{n}()==0;'for i,n in enumerate(selected))+'\ntransition '+' && '.join(f'pass{i}'for i in range(len(selected)))+' { true -> (0) _ -> (1) } }'+suffix
  return body
 def evaluate(body,mutation=None):
  if mutation:
   old,new=mutation;assert body.count(old)==1;body=body.replace(old,new)
  with tempfile.TemporaryDirectory(prefix='cathedral-cleanup-range-')as directory:
   path=Path(directory);(path/'main.omg').write_text(body);(path/'build.omg').write_text(build)
   r=subprocess.run([str(compiler),'--check',str(path/'main.omg')],cwd=ROOT,capture_output=True,text=True);out=r.stdout+r.stderr
   if mutation:
    if not r.returncode or 'cannot prove requires contract'not in out or '1 == 0'not in out:raise SystemExit('mutation failed:\n'+out)
   elif r.returncode:raise SystemExit('positive failed:\n'+out)
 if not args.controls_only:
  for start in range(args.start,min(args.end or len(blocks),len(blocks)),4):
   selected=list(range(start,min(start+4,args.end or len(blocks),len(blocks))));evaluate(fixture(selected));print('PASS range',selected,flush=True)
 if args.end is None:
  extra=(HERE/'extras.omg').read_text();prefix_extra=extra[:extra.index('machine begin_checks')]
  names=['begin_checks','inactive','forged','mismatch','resume_two','max_budget']
  extra_blocks=re.findall(r'machine (?:'+ '|'.join(names)+r')\(\)->i32 \{.*?(?=\nmachine (?:'+ '|'.join(names)+r')|\Z)',extra,re.S)
  assert len(extra_blocks)==len(names)
  for name,block in zip(names,extra_blocks):
   body=prefix_extra+block+f'\nmachine test_result()->i32 {{ let value:i32={name}(); value }}'+suffix
   evaluate(body);print('PASS extra',name,flush=True)
   if name=='max_budget':evaluate(body,('done_budget==18446744073709551614','done_budget==18446744073709551615'));print('PASS maximum budget body mutation',flush=True)
   if name=='resume_two':evaluate(body,('second.branch.frames_to_retire[0]==20480','second.branch.frames_to_retire[0]==12288'));print('PASS resumed order body mutation',flush=True)
  for n,old,new in [(0,'value.cursor.status in RangeStatus::Done','value.cursor.status in RangeStatus::Exhausted'),(2,'value.cursor.next_page==2097152','value.cursor.next_page==4096')]:
   evaluate(fixture([n]),(old,new));print('PASS body mutation',n,old,flush=True)
 print('PASS selected cleanup ranges and controls',flush=True)
if __name__=='__main__':main()
