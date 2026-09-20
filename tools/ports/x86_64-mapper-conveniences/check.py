#!/usr/bin/env python3
"""Actual pinned defaults plus bounded Omega wrapper/route fixtures and body controls."""
import argparse,hashlib,subprocess,sys,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def run(*args):subprocess.run([str(a) for a in args],cwd=ROOT,check=True)
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--omega',type=Path,default=ROOT.parent/'Omega/target/release/omega');p.add_argument('--host-only',action='store_true');p.add_argument('--controls-only',action='store_true');a=p.parse_args()
 for name in ('generate_reference.py','generate_fixtures.py','generate_inventory.py'):run(sys.executable,HERE/name,'--check')
 run(sys.executable,ROOT/'tools/ports/inventory.py','check',ROOT/'source/libraries/x86_64/mapper-conveniences-inventory.json','--checkout',ROOT/'reference_code/rust-osdev/x86_64','--require-transcribed')
 run('cargo','run','--quiet','--locked','--manifest-path',HERE/'Cargo.toml')
 if a.host_only:return
 omega=a.omega.resolve();print('Omega SHA256:',hashlib.sha256(omega.read_bytes()).hexdigest(),flush=True)
 if not a.controls_only:
  run(omega,'--check',ROOT/'source/libraries/x86_64/mapper_conveniences.omg')
  for file in ('main.omg','routes.omg','translate_route.omg'):run(omega,'--check',HERE/file)
 build=(HERE/'build.omg').read_text().replace('../../../source/',str(ROOT/'source')+'/')
 controls=[('main.omg','request_is(request, 12288, flags, 7)','request_is(request, 12288, flags, 6)'),('main.omg','number_is(result, 12288)','number_is(result, 12287)'),('main.omg','bad_identity in NumberResult::Rejected','bad_identity in NumberResult::Value'),('routes.omg','value.edits.write_mask == 15','value.edits.write_mask == 14')]
 for file,old,new in controls:
  with tempfile.TemporaryDirectory(prefix='cathedral-mapper-default-control-') as directory:
   d=Path(directory);body=(HERE/file).read_text();assert old in body;body=body.replace(old,new,1)
   (d/'main.omg').write_text(body);(d/'build.omg').write_text(build)
   r=subprocess.run([str(omega),'--check',str(d/'main.omg')],cwd=ROOT,capture_output=True,text=True);out=r.stdout+r.stderr
   if r.returncode==0 or 'cannot prove requires contract' not in out or '1 == 0' not in out:raise SystemExit('expected body failure missing:\n'+out)
  print('PASS body mutation:',old,flush=True)
 print('PASS selected pure Mapper wrapper checks; no flush/custody or native ABI claim')
if __name__=='__main__':main()
