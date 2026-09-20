#!/usr/bin/env python3
"""PIC inventory, recorded-upstream, translated semantic, and existing-source checks."""
from pathlib import Path
import subprocess
import sys
ROOT=Path(__file__).resolve().parents[3]
HERE=Path(__file__).resolve().parent
OMEGA=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else ROOT.parent/'Omega/target/release/omega'
def run(*args):
    subprocess.run([str(x) for x in args],cwd=ROOT,check=True)
positive=(HERE/'main.omg').read_text()
negative=(HERE/'negative.omg').read_text()
if negative != positive.replace('plan.writes[0].value == 0x11','plan.writes[0].value == 0x12') or negative==positive:
    raise SystemExit('negative fixture must mutate the real initialization expectation, keeping final assertion unchanged')
run(sys.executable,'tools/ports/inventory.py','check','source/libraries/pic8259/inventory.json','--checkout','reference_code/rust-osdev/pic8259')
run(sys.executable,'tools/ports/vectors.py','source/libraries/pic8259/vectors.json')
run(sys.executable,HERE/'check_upstream.py')
run(OMEGA,'--check','source/libraries/pic8259/plans.omg')
run(OMEGA,'--check',HERE/'main.omg')
result=subprocess.run([str(OMEGA),'--check',str(HERE/'negative.omg')],cwd=ROOT,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
if result.returncode==0 or '1 == 0' not in result.stdout:
    raise SystemExit('negative body mutation did not fail expected computed contract:\n'+result.stdout)
print('PASS mutated first initialization expectation evaluates test failure1; unchanged final contract rejects1==0',flush=True)
run(sys.executable,HERE/'check_existing.py',OMEGA)
