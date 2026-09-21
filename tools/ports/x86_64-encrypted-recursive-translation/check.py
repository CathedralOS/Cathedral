#!/usr/bin/env python3
"""Check actual pinned Rust mapper results and Omega semantic bodies."""
import argparse,hashlib,subprocess,sys,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def main():
 p=argparse.ArgumentParser();p.add_argument('--omega',type=Path,default=Path('/tmp/cathedral-omega-eaa7993/release/omega'));p.add_argument('--host-only',action='store_true');p.add_argument('--start',type=int,default=0);p.add_argument('--end',type=int,default=55);args=p.parse_args()
 subprocess.run([sys.executable,str(HERE/'map_inventory.py'),'--check'],cwd=ROOT,check=True)
 subprocess.run([sys.executable,str(HERE/'generate_reference.py'),'--check'],cwd=ROOT,check=True)
 actual=subprocess.check_output(['cargo','run','--quiet','--locked','--manifest-path',str(HERE/'Cargo.toml')],cwd=ROOT)
 assert actual==(HERE/'reference.jsonl').read_bytes(),'Rust reference drift'
 subprocess.run([sys.executable,str(HERE/'generate.py'),'--check'],cwd=ROOT,check=True)
 print('PASS 330 adapted pinned recursive translations across11 isolated profiles',flush=True)
 if args.host_only:return
 compiler=args.omega.resolve();print('Omega SHA256:',hashlib.sha256(compiler.read_bytes()).hexdigest(),flush=True)
 build=(HERE/'build.omg').read_text().replace('../../../source/',str(ROOT/'source')+'/')
 def evaluate(body,mutation=None):
  if mutation:
   old,new=mutation;assert body.count(old)==1,old;body=body.replace(old,new)
  with tempfile.TemporaryDirectory(prefix='cathedral-encrypted-recursive-translation-') as folder:
   work=Path(folder);(work/'build.omg').write_text(build);(work/'main.omg').write_text(body)
   result=subprocess.run([str(compiler),'--check',str(work/'main.omg')],cwd=ROOT,capture_output=True,text=True);out=result.stdout+result.stderr
   if mutation:assert result.returncode and 'cannot prove requires contract' in out and '1 == 0' in out,out
   else:assert result.returncode==0,out
 for n in range(args.start,args.end):
  body=(HERE/'cases'/f'batch-{n:03}.omg').read_text();evaluate(body);print('PASS batch',n,flush=True)
  if n in [0,5,15,30,35,50]:
   # Mutate the computed leaf comparison itself, keeping the success contract.
   evaluate(body,('flags==want_flags','flags!=want_flags'));print('PASS leaf flags mutation',n,flush=True)
 if args.end==55:
  body=(HERE/'extras.omg').read_text();evaluate(body);print('PASS invalid virtual input',flush=True)
  evaluate(body,('value in Translation::InvalidVirtual','value in Translation::NotMapped'));print('PASS invalid virtual body mutation',flush=True)
  evaluate((HERE/'cases'/'batch-000.omg').read_text(),('kind==3','kind==2'));print('PASS leaf huge failure body mutation',flush=True)
 print('PASS selected encrypted recursive translation cases',flush=True)
if __name__=='__main__':main()
