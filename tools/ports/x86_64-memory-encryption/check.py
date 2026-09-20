#!/usr/bin/env python3
"""Check source-derived profile witnesses through actual Omega bodies."""
import argparse,hashlib,json,subprocess,sys,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def main():
 p=argparse.ArgumentParser();p.add_argument('--omega',type=Path,default=ROOT.parent/'Omega/target/release/omega');p.add_argument('--host-only',action='store_true');p.add_argument('--start',type=int,default=0);p.add_argument('--end',type=int);p.add_argument('--controls-only',action='store_true');args=p.parse_args()
 subprocess.run([sys.executable,str(HERE/'generate.py'),'--check'],cwd=ROOT,check=True)
 subprocess.run([sys.executable,str(HERE/'map_inventory.py'),'--check'],cwd=ROOT,check=True)
 subprocess.run(['cargo','test','--quiet','--locked','--manifest-path',str(HERE/'Cargo.toml'),'--test','reconfiguration'],cwd=ROOT,check=True)
 if args.host_only:return
 compiler=args.omega.resolve();print('Omega SHA-256:',hashlib.sha256(compiler.read_bytes()).hexdigest(),flush=True)
 build=(HERE/'build.omg').read_text().replace('../../../source/',str(ROOT/'source')+'/')
 def evaluate(body,mutation=None):
  if mutation:
   old,new=mutation;assert old in body,old;body=body.replace(old,new,1)
  with tempfile.TemporaryDirectory(prefix='cathedral-memory-encryption-') as directory:
   path=Path(directory);(path/'build.omg').write_text(build);(path/'main.omg').write_text(body)
   r=subprocess.run([str(compiler),'--check',str(path/'main.omg')],cwd=ROOT,text=True,capture_output=True);out=r.stdout+r.stderr
   if mutation:
    if not r.returncode or 'cannot prove requires contract' not in out or '1 == 0' not in out:raise SystemExit('mutation failed:\n'+out)
   elif r.returncode:raise SystemExit('positive failed:\n'+out)
 rows=json.loads((HERE/'cases.json').read_text());assert len(rows)==133
 if not args.controls_only:
  for start in range(args.start,min(args.end or len(rows),len(rows)),4):
   selected=rows[start:min(start+4,args.end or len(rows),len(rows))];prefix='';bodies=[]
   for n,row in enumerate(selected):
    source=(HERE/'cases'/f"{row['profile']}.omg").read_text();a=source.index('machine scenario()');b=source.index('const RESULT:');prefix=source[:a];bodies.append(source[a:b].replace('machine scenario()',f'machine profile_{n}()'))
   body=prefix+'\n'.join(bodies)+'\nmachine scenario()->i32 {\n'+'\n'.join(f'let pass{n}: bool=profile_{n}()==0;' for n in range(len(selected)))+'\ntransition '+' && '.join(f'pass{n}' for n in range(len(selected)))+' { true -> (0) _ -> (1) } }\nconst RESULT:i32=scenario();\nmachine require_success(value:i32) requires value == 0; {}\ndata Main {}\nmachine Main::main(&mut self){require_success(RESULT);}\n'
   evaluate(body);print('PASS profiles',list(range(start,start+len(selected))),flush=True)
 if args.end is None:
  extra=(HERE/'extras.omg').read_text();evaluate(extra);print('PASS malformed shifts and disabled/profile extras',flush=True)
  for profile,old,new in [('e47','number(set0, 140737488355328)','number(set0, 0)'),('s47','encrypted0 == true','encrypted0 == false'),('e47_s48_e47','address9 == 4081387162300416','address9 == 0')]:
   evaluate((HERE/'cases'/f'{profile}.omg').read_text(),(old,new));print('PASS body mutation',profile,flush=True)
  evaluate(extra,('profile.bit_mask == 140737488355328','profile.bit_mask == 0'));print('PASS invalid-shift preservation body mutation',flush=True)
 print('PASS selected memory-encryption profiles and controls',flush=True)
if __name__=='__main__':main()
