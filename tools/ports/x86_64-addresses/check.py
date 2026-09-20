#!/usr/bin/env python3
"""Pin audit, real Rust numeric witnesses, and Omega semantic behavior controls."""
import argparse
import hashlib
from pathlib import Path
import subprocess
import tempfile
import sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def run(*args):subprocess.run([str(a) for a in args],cwd=ROOT,check=True)
def main():
 parser=argparse.ArgumentParser(description=__doc__)
 parser.add_argument('--omega',type=Path,default=ROOT.parent/'Omega/target/release/omega')
 parser.add_argument('--host-only',action='store_true')
 args=parser.parse_args()
 run(sys.executable,ROOT/'tools/ports/inventory.py','check',ROOT/'source/libraries/x86_64/addresses-inventory.json','--checkout',ROOT/'reference_code/rust-osdev/x86_64','--require-transcribed')
 run(sys.executable,HERE/'generate.py','--check')
 run('cargo','+nightly-2026-09-04','run','--quiet','--locked','--manifest-path',HERE/'Cargo.toml')
 if args.host_only:return
 compiler=args.omega.resolve();print('Omega binary SHA-256:',hashlib.sha256(compiler.read_bytes()).hexdigest(),flush=True)
 run(compiler,'--check',HERE/'main.omg')
 build=(HERE/'build.omg').read_text().replace('../../../source/libraries/x86_64',str(ROOT/'source/libraries/x86_64'))
 controls=[('main.omg','equal(result0, 0)','equal(result0, 1)'),('extras.omg','forward.value == 0xffff800000000000','forward.value == 0xffff800000000001'),('extras.omg','overflow.value == 0xffffffffffffffff && overflow.overflow','overflow.value == 0xffffffffffffffff && !overflow.overflow')]
 for filename,old,new in controls:
  with tempfile.TemporaryDirectory(prefix='cathedral-x86-address-negative-') as directory:
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
 print('Address semantic tests passed. No native execution, pointer validity, or ABI comparison claimed.')
if __name__=='__main__':main()
