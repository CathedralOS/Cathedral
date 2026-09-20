#!/usr/bin/env python3
import argparse,subprocess,sys,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def run(*args):subprocess.run([str(a) for a in args],cwd=ROOT,check=True)
p=argparse.ArgumentParser();p.add_argument('--omega',type=Path,default=ROOT.parent/'Omega/target/release/omega');p.add_argument('--host-only',action='store_true');p.add_argument('--controls-only',action='store_true');a=p.parse_args()
run(sys.executable,HERE/'generate.py','--check');run(sys.executable,ROOT/'tools/ports/inventory.py','check',ROOT/'source/libraries/x86_64/instruction-observations-inventory.json','--checkout',ROOT/'reference_code/rust-osdev/x86_64','--require-transcribed');run('cargo','run','--quiet','--locked','--manifest-path',HERE/'Cargo.toml')
if not a.host_only:
 omega=a.omega.resolve()
 if not a.controls_only:run(omega,'--check',HERE/'main.omg')
 for old in ['rdrand_bit(1073741824) == true','smap_bit(1048576) == true']:
  with tempfile.TemporaryDirectory(prefix='cathedral-observation-control-') as directory:
   d=Path(directory);s=(HERE/'main.omg').read_text();assert s.count(old)==1
   (d/'main.omg').write_text(s.replace(old,old.replace('true','false')))
   (d/'build.omg').write_text((HERE/'build.omg').read_text().replace('../../../source/',str(ROOT/'source')+'/'))
   r=subprocess.run([str(omega),'--check',str(d/'main.omg')],cwd=ROOT,capture_output=True,text=True);out=r.stdout+r.stderr
   if r.returncode==0 or 'cannot prove requires contract' not in out or '1 == 0' not in out:raise SystemExit(out)
  print('PASS predicate body mutation:',old,flush=True)
print('PASS selected detached instruction observation checks')
