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
GROUPS = ['transfer', 'chunk', 'bulk']


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def text_sha(source):
    return hashlib.sha256(source.encode()).hexdigest()


def snapshot():
    paths = set(HERE.glob('*.py')) | {HERE/'cases.json', HERE/'toolchain.json'}
    paths |= {fixtures.BASE/name for name in ['fixtures.py', 'vectors.py', 'geometry_vectors.py', 'cases.json']}
    paths |= {HERE.parent/'field-write-chunks/fixtures.py', HERE.parent/'field-access/fixtures.py', fixtures.BRIDGE}
    for folder in ['aml', 'field_access', 'field_writes']:
        paths |= set((ROOT/'source/libraries/acpi'/folder).rglob('*.omg'))
    paths |= {ROOT/'source/libraries/acpi/interpreter'/name for name in ['build.omg', 'integers.omg', 'buffer_fields.omg']}
    paths |= {ROOT/'tools/ports/acpi/interpreter/execution'/name for name in ['checked_runner.rs', 'runner.Cargo.lock']}
    return {str(path.relative_to(ROOT)):sha(path) for path in sorted(paths)}


def rows(group):
    return (fixtures if group == 'transfer' else fixtures.chunk if group == 'chunk' else fixtures.chunk.bulk).cases()


def module_source(group, selected):
    source, entries = (fixtures if group == 'transfer' else fixtures.chunk if group == 'chunk' else fixtures.chunk.bulk).render(selected)
    receiver = group.title()+'WriteSuite'
    source = 'module authored_'+group+';\n'+re.sub(r'\bSuite\b', receiver, source)
    return source, [entry.replace('Suite::', receiver+'::', 1) for entry in entries]


def driver_source(groups):
    return ''.join('use authored_'+group+';\n' for group in groups)


def build_text(root=ROOT):
    source = 'machine build(builder:&mut Build){builder.application("cathedral-field-write-transfer");builder.freestanding=true;'
    for alias, folder in [('writes','field_writes'), ('access','field_access'), ('fields','aml/fields'), ('integers','interpreter'), ('aml','aml')]:
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
    parser.add_argument('--workers', type=int, default=3)
    args = parser.parse_args()
    groups = args.group or ['transfer']
    assert len(groups) == len(set(groups))
    assert args.batch_size>0 and 1<=args.workers<=3
    assert json.loads((HERE/'cases.json').read_text()) == rows('transfer')
    assert json.loads((fixtures.BASE/'cases.json').read_text()) == rows('bulk')
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
        with tempfile.TemporaryDirectory(prefix='cathedral-field-write-transfer-') as directory:
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
    unchanged = before == snapshot() and binary == sha(args.runner)
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
        if batch['exit_code']:
            print(batch['output'],flush=True)
    assert record['exit_code']==0, 'Compilation or execution failed; see retained diagnostic'
    for batch in completed:
        validate(batch['output'],batch['selections'])
    print('PASS', count, 'checked pairs;', record['elapsed_seconds'], 'seconds', flush=True)


if __name__ == '__main__':
    main()
