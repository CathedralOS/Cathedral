#!/usr/bin/env python3
"""Execute original body pairs in bounded packages with full dependency checks."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time
import fixtures

HERE, ROOT = fixtures.HERE, fixtures.ROOT
PIN = 'eaa7993a23623cd8fabf45350340479c5c9c7879'
RUNNER = Path('/tmp/cathedral-field-protocol-target/release/cathedral-acpi-checked-runner')
GROUPS = ['execution', 'bridge', 'mid', 'mid_bridge', 'integer', 'generic', 'pipeline', 'to_integer']


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def text_sha(source):
    return hashlib.sha256(source.encode()).hexdigest()


def snapshot():
    paths=set((ROOT/'source/libraries/acpi').rglob('*.omg')) | set(HERE.glob('*.py')) | set(HERE.glob('*-cases.json')) | {HERE/'toolchain.json'}
    paths |= {HERE.parent/'execution'/name for name in ['fixtures.py','checked_runner.rs','runner.Cargo.lock']}
    paths |= {fixtures.GENERIC/name for name in ['fixtures.py','decoder_fixtures.py','focused/bridge-atomicity/main.omg']}
    paths |= {HERE.parent/'mid-execution/fixtures.py',fixtures.NAMED/'fixtures.py',HERE.parent/'to-integer-execution/fixtures.py',HERE.parent.parent/'pipeline/fixtures.py'}
    return {str(path.relative_to(ROOT)):sha(path) for path in sorted(paths)}


def rows(group):
    return fixtures.rows(group)


def module_source(group, selected):
    source, entries = fixtures.render_rows(group, selected)
    receiver = ''.join(part.title() for part in group.split('_'))+'LogicalSuite'
    source = 'module authored_'+group+';\n'+re.sub(r'\bSuite\b', receiver, source)
    return source, [entry.replace('Suite::', receiver+'::', 1) for entry in entries]


def driver_source(groups):
    return ''.join('use authored_'+group+';\n' for group in groups)


def build_text(root=ROOT):
    source = 'machine build(builder:&mut Build){builder.application("cathedral-logical-execution");builder.freestanding=true;'
    for alias, folder in [('aml','aml'),('execution','interpreter/execution'),('integer_helpers','interpreter'),('pipeline','pipeline')]:
        source += 'builder.depend_as("'+alias+'",Source::Path {location:"'+str(root/'source/libraries/acpi'/folder)+'"});'
    return source+'}'


def validate(output, entries):
    assert 'CHECKED authored package and dependency bodies;' in output
    actual = re.findall(r'^PASS (\S+) expected=(\d+) observed=(\d+) error=None usage=', output, re.M)
    assert len(actual) == len(entries)
    assert [name+'='+expected for name,expected,observed in actual] == entries
    assert all(expected == observed for name,expected,observed in actual)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--group', choices=GROUPS, action='append')
    parser.add_argument('--match', default='')
    parser.add_argument('--runner', type=Path, default=RUNNER)
    parser.add_argument('--record', type=Path, default=HERE/'checked-verification.json')
    parser.add_argument('--batch-size', type=int, default=10)
    parser.add_argument('--workers', type=int, default=1)
    args = parser.parse_args()
    toolchain=json.loads((HERE/'toolchain.json').read_text())
    omega=Path(toolchain['omega_source'])
    assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=omega,text=True).strip()==PIN
    assert not subprocess.check_output(['git','status','--porcelain'],cwd=omega,text=True).strip()
    assert sha(args.runner)==toolchain['sha256'][str(args.runner)]
    groups = args.group or GROUPS
    assert len(groups) == len(set(groups))
    assert args.batch_size>0 and 1<=args.workers<=3
    for group in ['execution','bridge']:
        assert json.loads((HERE/(group+'-cases.json')).read_text()) == rows(group)
    before, binary = snapshot(), sha(args.runner)
    selections, batches = [], []
    for group in groups:
        selected = [row for row in rows(group) if not args.match or row['name'] in args.match.split(',')]
        assert selected
        selections.append(dict(group=group,cases=[row['name'] for row in selected]))
        batches.extend((group,selected[index:index+args.batch_size]) for index in range(0,len(selected),args.batch_size))
    build = build_text()
    started = time.monotonic()
    def run_batch(batch):
        group,selected=batch
        batch_started=time.monotonic()
        source,entries=module_source(group,selected)
        driver=driver_source([group])
        assert before==snapshot() and binary==sha(args.runner), 'Inputs changed before batch'
        with tempfile.TemporaryDirectory(prefix='cathedral-logical-execution-') as directory:
            work=Path(directory)
            (work/'main.omg').write_text(driver)
            (work/'build.omg').write_text(build)
            (work/('authored_'+group+'.omg')).write_text(source)
            run=subprocess.run([str(args.runner),str(work/'main.omg'),str(work/'build'),*entries],
                               capture_output=True,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'))
        unchanged=before==snapshot() and binary==sha(args.runner)
        return dict(group=group,cases=[row['name'] for row in selected],source_sha256=text_sha(source),
                    driver_sha256=text_sha(driver),selections=entries,exit_code=run.returncode,
                    source_unchanged=unchanged,elapsed_seconds=round(time.monotonic()-batch_started,3),output=run.stdout+run.stderr)
    completed=[]
    with ThreadPoolExecutor(max_workers=args.workers) as pool:
        for batch in pool.map(run_batch,batches):
            completed.append(batch)
            print('BATCH',batch['group'],len(batch['cases']),'pairs; exit',batch['exit_code'],';',batch['elapsed_seconds'],'seconds',flush=True)
            if batch['exit_code']:
                print(batch['output'],flush=True)
    unchanged = before == snapshot() and binary == sha(args.runner)
    assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=omega,text=True).strip()==PIN
    assert not subprocess.check_output(['git','status','--porcelain'],cwd=omega,text=True).strip()
    count=sum(len(selection['cases']) for selection in selections)
    record = dict(stage='checked interpreter; complete authored and dependency bodies; changed-expectation controls',
                  omega_revision=PIN, execution_root=str(ROOT), groups=groups, scope='selected' if args.match else 'full',
                  input_sha256=before, runner_path=str(args.runner.resolve()), runner_sha256=binary,
                  source_unchanged=unchanged, native_execution=False,
                  exit_code=0 if all(batch['exit_code']==0 for batch in completed) else 1,
                  groups_selected=selections,batches=completed,batch_size=args.batch_size,workers=args.workers,
                  positive_count=count,control_count=count,build_source=build,build_sha256=text_sha(build),
                  elapsed_seconds=round(time.monotonic()-started,3),
                  batch_elapsed_seconds_sum=round(sum(batch['elapsed_seconds'] for batch in completed),3))
    args.record.parent.mkdir(parents=True, exist_ok=True)
    args.record.write_text(json.dumps(record, indent=2, sort_keys=True)+'\n')
    assert unchanged, 'Inputs changed during verification'
    for batch in completed:
        assert batch['source_unchanged'], 'Inputs changed during batch'
    assert record['exit_code']==0, 'Compilation or execution failed; see retained diagnostic'
    for batch in completed:
        validate(batch['output'],batch['selections'])
    print('PASS', count, 'checked pairs;', record['elapsed_seconds'], 'seconds', flush=True)


if __name__ == '__main__':
    main()
