#!/usr/bin/env python3
"""Evaluate real detached GDT storage machines and rejecting body controls."""
import argparse,hashlib,subprocess,sys,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def main():
 p=argparse.ArgumentParser();p.add_argument('--omega',type=Path,default=ROOT.parent/'Omega/target/release/omega');p.add_argument('--host-only',action='store_true');p.add_argument('--case');p.add_argument('--controls-only',action='store_true');args=p.parse_args()
 subprocess.run([sys.executable,str(HERE/'generate.py'),'--check'],cwd=ROOT,check=True)
 subprocess.run([sys.executable,str(HERE/'map_inventory.py'),'--check'],cwd=ROOT,check=True)
 if args.host_only:return
 compiler=args.omega.resolve();print('Omega SHA-256:',hashlib.sha256(compiler.read_bytes()).hexdigest(),flush=True)
 build=(HERE/'build.omg').read_text().replace('../../../source/',str(ROOT/'source')+'/')
 def evaluate(path,mutation=None):
  body=path.read_text()
  if mutation:
   old,new=mutation;assert old in body,(path,old);body=body.replace(old,new,1)
  with tempfile.TemporaryDirectory(prefix='cathedral-gdt-storage-') as directory:
   out=Path(directory);(out/'build.omg').write_text(build);(out/'main.omg').write_text(body)
   result=subprocess.run([str(compiler),'--check',str(out/'main.omg')],cwd=ROOT,text=True,capture_output=True);message=result.stdout+result.stderr
   if mutation:
    if result.returncode==0 or 'cannot prove requires contract' not in message or '1 == 0' not in message:raise SystemExit('bad control '+str(path)+'\n'+message)
   elif result.returncode:raise SystemExit('positive failed '+str(path)+'\n'+message)
  print('PASS',path.name,'body mutation' if mutation else 'semantic evaluation',flush=True)
 cases=sorted((HERE/'cases').glob('*.omg'))
 if args.case:cases=[HERE/'cases'/f'{args.case}.omg']
 if not args.controls_only:
  for path in cases:evaluate(path)
  if not args.case:evaluate(HERE/'extras.omg')
 if not args.case:
  for file,old,new in [('system_fits','table.words[2] == 18446744073709551615','table.words[2] == 0'),('single_slot','result0 in AppendResult::Full','result0 in AppendResult::InvalidTable'),('mixed','added(result1, 19)','added(result1, 16)')]:evaluate(HERE/'cases'/f'{file}.omg',(old,new))
 if not args.case:evaluate(HERE/'extras.omg',('table.words[8191] == 102','table.words[8191] == 0'))
 print('PASS selected GDT storage fixtures and controls',flush=True)
if __name__=='__main__':main()
