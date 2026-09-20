#!/usr/bin/env python3
"""Execute original fixed ACPI scenarios in Omega constant semantics."""
import argparse
import hashlib
from pathlib import Path
import subprocess
import tempfile
import fixed_fixtures
from fixed_model import HERE,ROOT


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--omega',type=Path,default=Path('/tmp/cathedral-omega-eaa7993/release/omega'))
    parser.add_argument('--match',default='',help='Optional scenario-name filter for focused diagnosis')
    args=parser.parse_args();compiler=args.omega.resolve()
    print('Omega binary SHA-256:',hashlib.sha256(compiler.read_bytes()).hexdigest(),flush=True)
    for script in ['fixed_generate.py','madt_generate.py','fixed_fixtures.py']:
        subprocess.run(['python3',str(HERE/script),'--check'],cwd=ROOT,check=True)
    subprocess.run([str(compiler),'--check',str(HERE/'fixed_main.omg')],cwd=ROOT,check=True)
    rows=fixed_fixtures.cases();selected=[r for r in rows if args.match in r['name']]
    if not selected:raise SystemExit('No matching scenarios')
    build=(HERE/'build.omg').read_text().replace('../../../source/libraries/acpi',str(ROOT/'source/libraries/acpi'))
    groups=[];pending=[]
    for row in selected:
        if len(bytes.fromhex(row['bytes']))>128:
            if pending:groups.append(pending);pending=[]
            groups.append([row])
        else:
            pending.append(row)
            if len(pending)==4:groups.append(pending);pending=[]
    if pending:groups.append(pending)
    def check(source,negative=False):
        with tempfile.TemporaryDirectory(prefix='cathedral-acpi-fixed-')as directory:
            path=Path(directory);(path/'build.omg').write_text(build);(path/'main.omg').write_text(source)
            result=subprocess.run([str(compiler),'--check',str(path/'main.omg')],cwd=ROOT,capture_output=True,text=True)
            output=result.stdout+result.stderr
            valid=result.returncode!=0 and 'cannot prove requires contract'in output and '1 == 0'in output if negative else result.returncode==0
            if not valid:raise SystemExit(output)
    for number,group in enumerate(groups,1):
        print(f'CHECK {number}/{len(groups)} '+', '.join(r['name']for r in group),flush=True)
        check(fixed_fixtures.render(group))
    byname={r['name']:r for r in rows}
    for name,marker,replacement in [('gas_offset_0','result.value.address == 18364758544493064720','result.value.address == 18364758544493064721'),('hpet_count_2','result.num_comparators == 3','result.num_comparators == 2'),('madt_zero','result.error == 14','result.error == 0')]:
        source=fixed_fixtures.render([byname[name]])
        if source.count(marker)!=1:raise SystemExit('Negative marker needs exactly one match')
        check(source.replace(marker,replacement),True)
        print('PASS body-mutating negative control: '+name,flush=True)
    print(f'PASS {len(selected)} fixed ACPI scenarios in {len(groups)} semantic groups. Native/hardware NOT RUN.')

if __name__=='__main__':main()
