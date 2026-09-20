#!/usr/bin/env python3
"""Pinned source/reference checks, Omega body execution, mutation controls and exact probes."""
import argparse,hashlib,subprocess,sys,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def run(*args):subprocess.run([str(a) for a in args],cwd=ROOT,check=True)
def expect(omega,path,needle):
 result=subprocess.run([str(omega),'--check',str(path)],cwd=ROOT,capture_output=True,text=True);output=result.stdout+result.stderr
 if result.returncode==0 or needle not in output:raise SystemExit('expected diagnostic missing: '+needle+'\n'+output)
 print('REPRODUCED',path.name,':',needle,flush=True)
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--omega',type=Path,default=ROOT.parent/'Omega/target/release/omega');parser.add_argument('--host-only',action='store_true');args=parser.parse_args()
 for f in ('generate.py','generate_inventory.py','generate_reference_tests.py','generate_fixtures.py','generate_table_fixture.py'):run(sys.executable,HERE/f,'--check')
 run(sys.executable,ROOT/'tools/ports/inventory.py','check',ROOT/'source/drivers/facts/x86_interrupts-inventory.json','--checkout',ROOT/'reference_code/rust-osdev/x86_64','--require-transcribed')
 run(sys.executable,HERE/'measure.py');run(sys.executable,ROOT/'tools/ports/vectors.py',ROOT/'source/drivers/facts/x86_interrupts.vectors.json')
 run('cargo','test','--quiet','--locked','--manifest-path',HERE/'Cargo.toml');run(sys.executable,HERE/'check_gate_policy.py')
 if args.host_only:return
 omega=args.omega.resolve();print('Omega SHA-256:',hashlib.sha256(omega.read_bytes()).hexdigest(),flush=True)
 run(omega,'--check',ROOT/'source/libraries/x86_64/interrupt_bytes.omg')
 for name in ('main.omg','storage_main.omg','table_main.omg','table_encode_main.omg','table_decode_main.omg','table_reject_main.omg','layout_local_projection.omg'):run(omega,'--check',HERE/name)
 run(omega,'--check',ROOT/'tools/x86-idt-gate-layout-canary/main.omg')
 build=(HERE/'build.omg').read_text().replace('../../../source/',str(ROOT/'source')+'/')
 controls=[('main.omg','encoded[6] == 102','encoded[6] == 103','main.omg'),('main.omg','IdtRangeResult::Reversed','IdtRangeResult::IncludesException','main.omg'),('cases.omg','encoded[39] == 40','encoded[39] == 41','main.omg'),('table_impl.omg','vector == 254 && value == 1','vector == 253 && value == 1','table_main.omg'),('table_encode_main.omg','encoded[4085] == 14','encoded[4085] == 15','table_encode_main.omg'),('table_reject_main.omg','vector == 255 && value == 8','vector == 254 && value == 8','table_reject_main.omg')]
 for filename,old,new,entry in controls:
  with tempfile.TemporaryDirectory(prefix='cathedral-interrupt-negative-') as directory:
   p=Path(directory)
   for f in HERE.glob('*.omg'):
    s=build if f.name=='build.omg' else f.read_text()
    if f.name==filename:
     assert s.count(old)==1,(filename,old,s.count(old));s=s.replace(old,new)
    (p/f.name).write_text(s)
   expect(omega,p/entry,'1 == 0')
  print('PASS body mutation rejected:',old,flush=True)
 expect(omega,HERE/'full_table_budget_probe.omg','constant initializer: step budget exceeded')
 expect(omega,HERE/'layout_imported_probe.omg','selects private data')
 print('PASS pure interrupt evidence including separate complete-table encode/decode; imported-layout access remains explicitly unproved')
if __name__=='__main__':main()
