#!/usr/bin/env python3
"""Pinned pure witnesses and actual Omega execution, with body mutation controls."""
import argparse,hashlib,subprocess,sys,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def run(*args):subprocess.run([str(a) for a in args],cwd=ROOT,check=True)
def expect(omega,path,needle):
 r=subprocess.run([str(omega),'--check',str(path)],cwd=ROOT,capture_output=True,text=True);out=r.stdout+r.stderr
 if r.returncode==0 or needle not in out:raise SystemExit('missing expected diagnostic '+needle+'\n'+out)
 print('REPRODUCED',path.name,needle,flush=True)
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--omega',type=Path,default=ROOT.parent/'Omega/target/release/omega');p.add_argument('--host-only',action='store_true');a=p.parse_args()
 for name in ('generate_reference.py','generate_fixtures.py','generate_inventory.py'):run(sys.executable,HERE/name,'--check')
 run(sys.executable,ROOT/'tools/ports/inventory.py','check',ROOT/'source/libraries/x86_64/tlb-operands-inventory.json','--checkout',ROOT/'reference_code/rust-osdev/x86_64','--require-transcribed')
 run(sys.executable,HERE/'measure.py');run(sys.executable,ROOT/'tools/ports/vectors.py',ROOT/'source/libraries/x86_64/tlb-operands.vectors.json')
 run('cargo','+nightly-2026-09-04','test','--quiet','--locked','--manifest-path',HERE/'Cargo.toml')
 if a.host_only:return
 omega=a.omega.resolve();print('Omega SHA256:',hashlib.sha256(omega.read_bytes()).hexdigest(),flush=True)
 run(omega,'--check',ROOT/'source/libraries/x86_64/tlb_operands.omg')
 run(omega,'--check',HERE/'main.omg');run(omega,'--check',HERE/'layout_local_projection.omg')
 build=(HERE/'build.omg').read_text().replace('../../../source/',str(ROOT/'source')+'/')
 controls=[('main.omg','bytes[13] == 128','bytes[13] == 129'),('main.omg','operand_is(b, 1, 4095, 0)','operand_is(b, 2, 4095, 0)'),('cases.omg','chunk_is(new, start, 0, 1, new_next)','chunk_is(new, start, 1, 1, new_next)'),('cases.omg','regs_are(result, expected, 2147549183, 268435455)','regs_are(result, expected, 2147549183, 268435454)')]
 for file,old,new in controls:
  with tempfile.TemporaryDirectory(prefix='cathedral-tlb-control-') as directory:
   d=Path(directory)
   for f in HERE.glob('*.omg'):
    text=build if f.name=='build.omg' else f.read_text()
    if f.name==file:
     assert old in text,(file,old);text=text.replace(old,new,1)
    (d/f.name).write_text(text)
   expect(omega,d/'main.omg','1 == 0')
  print('PASS body mutation:',old,flush=True)
 expect(omega,HERE/'layout_imported_probe.omg','selects private data')
 print('PASS pure TLB operand evidence; no CPU/invalidation/native Omega ABI claims')
if __name__=='__main__':main()
