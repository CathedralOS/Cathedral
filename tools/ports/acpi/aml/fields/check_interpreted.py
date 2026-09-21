#!/usr/bin/env python3
"""Execute unchanged field fixtures and mutations with the pinned checked runner."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[4]
RUNNER_SHA='e6d0aee6b4dddbbf34a60cffbe8f158643cc5c4d100f9e20f0481f75f52890be'

def snapshot():
    sources=set((ROOT/'source/libraries/acpi/aml').rglob('*.omg'))
    sources.update((HERE/'cases').glob('*.omg'))
    sources.update([HERE/'cases.json',HERE/'build.omg',Path(__file__).resolve(),ROOT/'tools/ports/acpi/interpreter/execution/checked_runner.rs',ROOT/'tools/ports/acpi/interpreter/execution/runner.Cargo.lock'])
    return {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in sorted(sources)}

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--runner',type=Path,default=Path('/tmp/cathedral-acpi-execution-checked/release/cathedral-acpi-checked-runner'))
    parser.add_argument('--record',type=Path,default=HERE/'checked-verification.json')
    args=parser.parse_args()
    assert hashlib.sha256(args.runner.read_bytes()).hexdigest()==RUNNER_SHA,'Build the pinned execution checked runner first'
    metadata=json.loads((HERE/'cases.json').read_text());before=snapshot()
    imports=set();helpers={};bodies=[];selections=[]
    for name,item in metadata.items():
        source=(HERE/'cases'/f'{name}.omg').read_text()
        imports.update(re.findall(r'^use [^;]+;',source,re.M))
        start=source.index('machine test()');end=source.index('const TEST_RESULT');body=source[start:end]
        assert body.count(item['mutation'][0])==1,name
        for found in re.finditer(r'^machine (\w+)\(',source,re.M):
            if found[1]in ['test','require_ok']:continue
            opening=source.index('{',found.start());depth=1;closing=opening+1
            while depth:
                depth+=(source[closing]=='{')-(source[closing]=='}');closing+=1
            helper=source[found.start():closing]
            if found[1]in helpers:assert helpers[found[1]]==helper,name
            else:helpers[found[1]]=helper
        for control in [False,True]:
            machine='FieldSuite::'+name.replace('-','_')+('_control'if control else'_positive')
            selected=body.replace(*item['mutation'])if control else body
            bodies.append(selected.replace('machine test()',f'machine {machine}(&mut self)',1))
            selections.append(machine+'='+str(int(control)))
    with tempfile.TemporaryDirectory(prefix='cathedral-acpi-fields-checked-')as directory:
        work=Path(directory)
        (work/'main.omg').write_text('\n'.join(sorted(imports))+'\n'+'\n'.join(helpers.values())+'\ndata FieldSuite {}\n'+'\n'.join(bodies))
        (work/'build.omg').write_text((HERE/'build.omg').read_text().replace('../../../../../source/libraries/acpi/aml',str(ROOT/'source/libraries/acpi/aml')))
        started=time.monotonic();lines=[]
        process=subprocess.Popen([str(args.runner),str(work/'main.omg'),str(work/'build'),*selections],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'))
        for line in process.stdout:print(line,end='',flush=True);lines.append(line)
        if process.wait():raise SystemExit('Field regression failed')
        assert snapshot()==before,'Source changed during regression'
        record={'format':'cathedral-acpi-fields-checked-v1','stage':'checked-interpreter execution; native/hardware not run','omega_revision':'eaa7993a23623cd8fabf45350340479c5c9c7879','runner_sha256':RUNNER_SHA,'scenario_count':len(metadata),'control_count':len(metadata),'cases':list(metadata),'evaluator_step_limit':10000000,'elapsed_seconds':round(time.monotonic()-started,3),'source_sha256':before,'output':''.join(lines)}
        args.record.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
    print(f'PASS {len(metadata)} unchanged field bodies + {len(metadata)} original mutations; native/hardware NOT RUN.')

if __name__=='__main__':main()
