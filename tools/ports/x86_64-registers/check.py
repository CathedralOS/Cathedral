#!/usr/bin/env python3
"""Audit pinned register facts and execute real pure bodies with mutation controls."""
import argparse
import hashlib
from pathlib import Path
import subprocess
import sys
import tempfile
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def run(*args):subprocess.run([str(a) for a in args],cwd=ROOT,check=True)
def main():
 parser=argparse.ArgumentParser(description=__doc__)
 parser.add_argument('--omega',type=Path,default=ROOT.parent/'Omega/target/release/omega')
 parser.add_argument('--host-only',action='store_true');args=parser.parse_args()
 for generator in ('generate.py','generate_inventory.py','generate_reference_tests.py','generate_fixtures.py'):
  run(sys.executable,HERE/generator,'--check')
 run(sys.executable,ROOT/'tools/ports/inventory.py','check',ROOT/'source/drivers/facts/x86_registers-inventory.json','--checkout',ROOT/'reference_code/rust-osdev/x86_64','--require-transcribed')
 run(sys.executable,HERE/'measure.py')
 run(sys.executable,ROOT/'tools/ports/vectors.py',ROOT/'source/drivers/facts/x86_registers.vectors.json')
 run('cargo','test','--quiet','--locked','--manifest-path',HERE/'Cargo.toml')
 if args.host_only:return
 compiler=args.omega.resolve();print('Omega binary SHA-256:',hashlib.sha256(compiler.read_bytes()).hexdigest(),flush=True)
 run(compiler,'--check',ROOT/'source/libraries/x86_64/registers.omg')
 run(compiler,'--check',HERE/'main.omg')
 build=(HERE/'build.omg').read_text().replace('../../../source/libraries/x86_64',str(ROOT/'source/libraries/x86_64')).replace('../../../source/drivers/facts',str(ROOT/'source/drivers/facts'))
 controls=[('main.omg','registers::dr7_size(registers::dr7_set_size(base, 0, 2), 0) == 2','registers::dr7_size(registers::dr7_set_size(base, 0, 2), 0) == 3'),('main.omg','registers::star_plan(11, 3, 8, 16).error == 5','registers::star_plan(11, 3, 8, 16).error == 0'),('values.omg','x86_registers::MXCSR_RESET_BITS == 8064','x86_registers::MXCSR_RESET_BITS == 8065')]
 for filename,old,new in controls:
  with tempfile.TemporaryDirectory(prefix='cathedral-x86-register-negative-') as directory:
   p=Path(directory)
   for source in HERE.glob('*.omg'):
    text=build if source.name=='build.omg' else source.read_text()
    if source.name==filename:
     assert text.count(old)==1,(filename,old);text=text.replace(old,new)
    (p/source.name).write_text(text)
   result=subprocess.run([str(compiler),'--check',str(p/'main.omg')],cwd=ROOT,capture_output=True,text=True)
   output=result.stdout+result.stderr
   if result.returncode==0 or 'cannot prove requires contract' not in output or '1 == 0' not in output:raise SystemExit('Behavior mutation did not evaluate to failure:\n'+output)
  print('PASS body mutation rejected:',old,flush=True)
 print('PASS register semantic tests. No native execution, aggregate ABI, or live hardware claim.')
if __name__=='__main__':main()
