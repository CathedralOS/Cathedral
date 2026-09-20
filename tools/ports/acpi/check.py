#!/usr/bin/env python3
"""Check ACPI header evidence and execute Omega semantic assertions/negative controls."""
import argparse
import hashlib
from pathlib import Path
import subprocess
import tempfile
import header_fixtures

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--omega',type=Path,default=ROOT.parent/'Omega/target/release/omega')
    args=parser.parse_args();compiler=args.omega.resolve()
    print('Omega binary SHA-256:',hashlib.sha256(compiler.read_bytes()).hexdigest(),flush=True)
    for script in ['header_evidence.py','header_fixtures.py']:
        subprocess.run(['python3',str(HERE/script),'--check'],cwd=ROOT,check=True)
    subprocess.run([str(compiler),'--check',str(HERE/'main.omg')],cwd=ROOT,check=True)
    cases=header_fixtures.cases()
    rows={row['name']:row for row in cases}
    build=(HERE/'build.omg').read_text().replace('../../../source/libraries/acpi',str(ROOT/'source/libraries/acpi'))
    groups=[]; pending=[]
    for row in cases:
        if len(bytes.fromhex(row['bytes'])) > 512:
            if pending:groups.append(pending);pending=[]
            groups.append([row])
        else:
            pending.append(row)
            if len(pending)==4:groups.append(pending);pending=[]
    if pending:groups.append(pending)
    for number,group in enumerate(groups,1):
        with tempfile.TemporaryDirectory(prefix='cathedral-acpi-positive-') as directory:
            path=Path(directory);(path/'build.omg').write_text(build);(path/'main.omg').write_text(header_fixtures.render(group))
            run=subprocess.run([str(compiler),'--check',str(path/'main.omg')],cwd=ROOT,capture_output=True,text=True)
            if run.returncode:raise SystemExit('Case group '+', '.join(r['name'] for r in group)+' failed:\n'+run.stdout+run.stderr)
        print(f'PASS semantic group {number}/{len(groups)}: '+', '.join(r['name'] for r in group),flush=True)
    controls=[('rsdp_legacy','result.value.rsdt_address == 2166572391','result.value.rsdt_address == 2166572392'),
              ('rsdp_compensated_legacy_checksum','result.error == 6','result.error == 0'),
              ('xsdt_entries','entry1.address == 18364758544493064720','entry1.address == 18364758544493064721')]
    build=(HERE/'build.omg').read_text().replace('../../../source/libraries/acpi',str(ROOT/'source/libraries/acpi'))
    for name,marker,replacement in controls:
        source=header_fixtures.render([rows[name]])
        if source.count(marker)!=1:raise SystemExit(f'control {name} needs one marker: {marker}')
        with tempfile.TemporaryDirectory(prefix='cathedral-acpi-negative-') as directory:
            path=Path(directory);(path/'build.omg').write_text(build);(path/'main.omg').write_text(source.replace(marker,replacement))
            run=subprocess.run([str(compiler),'--check',str(path/'main.omg')],cwd=ROOT,capture_output=True,text=True)
            output=run.stdout+run.stderr
            if not run.returncode or 'cannot prove requires contract' not in output or '1 == 0' not in output:
                raise SystemExit(f'{name} control failed to reject computed false expectation:\n{output}')
        print(f'PASS body-mutating negative control: {name}',flush=True)
    print(f'PASS {len(rows)} original ACPI header scenarios in Omega semantic evaluation; native/hardware NOT RUN.')


if __name__=='__main__':main()
