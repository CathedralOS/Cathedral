#!/usr/bin/env python3
"""Run Field/ObjectType/ToInteger interactions through actual Omega bodies and bind exact inputs."""
import argparse
import hashlib
import json
import os
import re
import subprocess
import tempfile
import time
from pathlib import Path
import fixtures

HERE, ROOT = fixtures.HERE, fixtures.ROOT
RUNNER = Path('/tmp/cathedral-acpi-generic-checked/release/cathedral-acpi-checked-runner')
PIN = 'eaa7993a23623cd8fabf45350340479c5c9c7879'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def digest(text):
    return hashlib.sha256(text.encode()).hexdigest()


def build(root):
    source = 'machine build(builder:&mut Build){builder.package("component-integration");builder.freestanding=true;'
    for alias, path in [('aml','source/libraries/acpi/aml'), ('execution','source/libraries/acpi/interpreter/execution'), ('integer_helpers','source/libraries/acpi/interpreter'), ('pipeline','source/libraries/acpi/pipeline')]:
        source += f'builder.depend_as("{alias}",Source::Path {{location:"{root/path}"}});'
    return source + '}\n'


def snapshot():
    paths = list((ROOT/'source/libraries/acpi').rglob('*.omg'))
    paths += list(HERE.glob('*.py')) + [HERE/'cases.json']
    paths += [fixtures.GENERIC/p for p in ['fixtures.py','decoder_fixtures.py','focused/bridge-atomicity/main.omg']]
    paths += [HERE.parent/'execution'/p for p in ['fixtures.py','checked_runner.rs','runner.Cargo.lock']]
    return {str(p.relative_to(ROOT)): sha(p) for p in sorted(set(paths))}


def validate(output, selections):
    assert output.count('CHECKED authored package and dependency bodies;') == 1, output
    actual = re.findall(r'^PASS (\S+) expected=(\d+) observed=(\d+) error=None usage=', output, re.M)
    assert len(actual) == len(selections), output
    for (name, want, got), selected in zip(actual, selections):
        assert name+'='+want == selected and want == got


def verify(path):
    record = json.loads(path.read_text())
    rows, source, names = fixtures.render(record['match'])
    assert record['source_sha256'] == snapshot()
    assert record['runner_sha256'] == sha(Path(record['runner']))
    assert record['fixture_sha256'] == digest(source)
    assert record['cases'] == [r['name'] for r in rows]
    assert record['selections'] == names
    assert record['production_root'] == str(ROOT)
    assert record['build_source'] == build(ROOT) and record['build_sha256'] == digest(build(ROOT))
    assert record['source_unchanged'] and record['exit_code'] == 0
    validate(record['output'], names)
    print('PASS exact current-input receipt:', path)


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--match', default='')
    p.add_argument('--runner', type=Path, default=RUNNER)
    p.add_argument('--record', type=Path)
    p.add_argument('--verify', type=Path)
    args=p.parse_args()
    if args.verify:
        return verify(args.verify)
    root=ROOT
    rows,source,names=fixtures.render(args.match)
    inputs=snapshot(); binary=sha(args.runner); started=time.monotonic()
    with tempfile.TemporaryDirectory(prefix='cathedral-component-integration-') as directory:
        work=Path(directory); (work/'main.omg').write_text(source); (work/'build.omg').write_text(build(root))
        command=[str(args.runner),str(work/'main.omg'),str(work/'build'),*names]
        process=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,
                                 env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'))
        lines=[]
        for line in process.stdout:
            print(line,end='',flush=True);lines.append(line)
        code=process.wait(); output=''.join(lines)
        unchanged=inputs==snapshot() and binary==sha(args.runner)
        record=dict(omega_revision=PIN,production_root=str(root),group='component integration',match=args.match,
                    stage='checked interpreter',native_execution=False,runner=str(args.runner),runner_sha256=binary,
                    build_source=build(root),build_sha256=digest(build(root)),fixture_sha256=digest(source),
                    source_sha256=inputs,source_unchanged=unchanged,cases=[r['name']for r in rows],selections=names,
                    command=command,exit_code=code,output=output,elapsed_seconds=time.monotonic()-started)
        if args.record:
            args.record.parent.mkdir(parents=True,exist_ok=True)
            args.record.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
        assert unchanged,'Inputs changed during verification'
        if code:
            raise SystemExit(code)
        validate(output,names)
    print('PASS',len(rows),'component integration','behavior/control pairs; native/hardware NOT RUN')


if __name__=='__main__':
    main()
