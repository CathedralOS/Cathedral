#!/usr/bin/env python3
"""Execute bounded inert topology scenarios and body-mutating controls."""
import argparse,hashlib,subprocess,tempfile
from pathlib import Path
import topology_fixtures as fixtures
from fixed_model import HERE,ROOT

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--omega',type=Path,default=Path('/tmp/cathedral-omega-eaa7993/release/omega'));p.add_argument('--match',default='');args=p.parse_args();compiler=args.omega.resolve()
 print('Omega binary SHA-256:',hashlib.sha256(compiler.read_bytes()).hexdigest(),flush=True)
 subprocess.run(['python3',str(HERE/'topology_fixtures.py'),'--check'],check=True,cwd=ROOT)
 subprocess.run([str(compiler),'--check',str(HERE/'topology_main.omg')],check=True,cwd=ROOT)
 rows=fixtures.cases();selected=[r for r in rows if args.match in r['name']]
 if not selected:raise SystemExit('No matching cases')
 build=(HERE/'build.omg').read_text().replace('../../../source/libraries/acpi',str(ROOT/'source/libraries/acpi'))
 def check(source,negative=False):
  with tempfile.TemporaryDirectory(prefix='cathedral-acpi-topology-')as directory:
   path=Path(directory);(path/'build.omg').write_text(build);(path/'main.omg').write_text(source)
   run=subprocess.run([str(compiler),'--check',str(path/'main.omg')],capture_output=True,text=True,cwd=ROOT);output=run.stdout+run.stderr
   valid=run.returncode!=0 and 'cannot prove requires contract'in output and '1 == 0'in output if negative else run.returncode==0
   if not valid:raise SystemExit(output)
 for i in range(0,len(selected),4):
  group=selected[i:i+4];print('CHECK '+', '.join(r['name']for r in group),flush=True);check(fixtures.render(group))
 rows={r['name']:r for r in rows}
 for name,marker,replacement in [('cpu_1_True_42','value.enabled == true','value.enabled == false'),('ecam_nonzero_first','result.address == 2281701376','result.address == 2147483648'),('topology_malformed','result.error == 14','result.error == 0')]:
  source=fixtures.render([rows[name]])
  if source.count(marker)!=1:raise SystemExit('Control marker needs one occurrence')
  check(source.replace(marker,replacement),True);print('PASS body-mutating control: '+name,flush=True)
 print(f'PASS {len(selected)} topology semantic cases; native/hardware NOT RUN.')

if __name__=='__main__':main()
