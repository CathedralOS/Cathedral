#!/usr/bin/env python3
"""Check actual pinned Rust mapper results and Omega semantic bodies."""
import argparse,hashlib,subprocess,sys,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def main():
 p=argparse.ArgumentParser();p.add_argument('--omega',type=Path,default=Path('/tmp/cathedral-omega-eaa7993/release/omega'));p.add_argument('--host-only',action='store_true');p.add_argument('--start',type=int,default=0);p.add_argument('--end',type=int,default=8);args=p.parse_args()
 actual=subprocess.check_output(['cargo','run','--quiet','--locked','--manifest-path',str(HERE/'Cargo.toml')],cwd=ROOT)
 assert actual==(HERE/'reference.jsonl').read_bytes(),'Rust reference drift'
 subprocess.run([sys.executable,str(HERE/'generate.py'),'--check'],cwd=ROOT,check=True)
 subprocess.run([sys.executable,str(HERE/'map_inventory.py'),'--check'],cwd=ROOT,check=True)
 print('PASS 240 actual Translate-default calls across8 profiles',flush=True)
 if args.host_only:return
 compiler=args.omega.resolve();print('Omega SHA256:',hashlib.sha256(compiler.read_bytes()).hexdigest(),flush=True)
 build=(HERE/'build.omg').read_text().replace('../../../source/',str(ROOT/'source')+'/')
 def evaluate(body,mutation=None):
  if mutation:
   old,new=mutation;assert body.count(old)==1,old;body=body.replace(old,new)
  with tempfile.TemporaryDirectory(prefix='cathedral-encrypted-projection-') as folder:
   work=Path(folder);(work/'build.omg').write_text(build);(work/'main.omg').write_text(body)
   result=subprocess.run([str(compiler),'--check',str(work/'main.omg')],cwd=ROOT,capture_output=True,text=True);out=result.stdout+result.stderr
   if mutation:assert result.returncode and 'cannot prove requires contract' in out and '1 == 0' in out,out
   else:assert result.returncode==0,out
 for n in range(args.start,args.end):
  body=(HERE/'cases'/f'batch-{n:02}.omg').read_text();evaluate(body);print('PASS batch',n,flush=True)
  evaluate(body,('value==expected','value!=expected'));print('PASS operand body mutation',n,flush=True)
 print('PASS selected encrypted projection cases',flush=True)
if __name__=='__main__':main()
