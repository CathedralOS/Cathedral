#!/usr/bin/env python3
"""Pinned source/value audit and pure-body execution with live mutation controls."""
import argparse,hashlib,json,re,subprocess,sys,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def run(*args):subprocess.run([str(a) for a in args],cwd=ROOT,check=True)
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--omega',type=Path,default=ROOT.parent/'Omega/target/release/omega');parser.add_argument('--host-only',action='store_true');args=parser.parse_args()
 for f in ('generate.py','generate_inventory.py','generate_reference_tests.py','generate_fixtures.py'):run(sys.executable,HERE/f,'--check')
 run(sys.executable,ROOT/'tools/ports/inventory.py','check',ROOT/'source/drivers/facts/x86_descriptors-inventory.json','--checkout',ROOT/'reference_code/rust-osdev/x86_64','--require-transcribed')
 run(sys.executable,HERE/'measure.py');run(sys.executable,ROOT/'tools/ports/vectors.py',ROOT/'source/drivers/facts/x86_descriptors.vectors.json')
 run('cargo','test','--quiet','--locked','--manifest-path',HERE/'Cargo.toml')
 ist=(ROOT/'source/drivers/facts/x86_interrupt_stacks.omg').read_text()
 for n,name in enumerate(('DOUBLE_FAULT','NMI','MACHINE_CHECK','MASKABLE_IRQ'),1):
  pattern=r'pub const X86_'+name+r'_STACK: X86IstStackClass = X86IstStackClass \{\s*stack_class: '+str(n)+r',\s*ist_index: '+str(n)+r',\s*\};'
  if not re.search(pattern,ist):raise SystemExit('Existing IST coupling changed: '+name)
 print('PASS existing four IST policy pairs remain coupled and unchanged',flush=True)
 if args.host_only:return
 omega=args.omega.resolve();print('Omega SHA-256:',hashlib.sha256(omega.read_bytes()).hexdigest(),flush=True)
 run(omega,'--check',ROOT/'source/libraries/x86_64/descriptors.omg')
 run(omega,'--check',HERE/'main.omg')
 run(omega,'--check',HERE/'layout_local_projection.omg')
 build=(HERE/'build.omg').read_text().replace('../../../source/',str(ROOT/'source')+'/')
 controls=[('main.omg','system_high_equals(system, 0x123456789abcdef0)','system_high_equals(system, 0x123456789abcdeff)'),('main.omg','accepted.limit == 104','accepted.limit == 103'),('main.omg','ring3.selector.raw == 65531','ring3.selector.raw == 65528'),('cases.omg','encoded[102] == 103','encoded[102] == 104')]
 for filename,old,new in controls:
  with tempfile.TemporaryDirectory(prefix='cathedral-descriptor-negative-') as directory:
   p=Path(directory)
   for f in HERE.glob('*.omg'):
    s=build if f.name=='build.omg' else f.read_text()
    if f.name==filename:
     assert s.count(old)==1,(filename,old);s=s.replace(old,new)
    (p/f.name).write_text(s)
   result=subprocess.run([str(omega),'--check',str(p/'main.omg')],cwd=ROOT,capture_output=True,text=True);output=result.stdout+result.stderr
   if result.returncode==0 or 'cannot prove requires contract' not in output or '1 == 0' not in output:raise SystemExit('body control failed to reject by evaluated result:\n'+output)
  print('PASS body mutation rejected:',old,flush=True)
 print('PASS pure descriptor evidence; native imported-layout and CPU integration claims excluded')
if __name__=='__main__':main()
