#!/usr/bin/env python3
"""Checked equal-extent Buffer Store and regression receipts with bound inputs."""
import argparse
import ast
import hashlib
import json
import os
import re
import subprocess
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from fixtures import BASE, HELPERS, IMPORTS, body, cases, regression_cases

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
RUNNER = Path('/tmp/cathedral-acpi-generic-checked/release/cathedral-acpi-checked-runner')
PIN = 'eaa7993a23623cd8fabf45350340479c5c9c7879'
SOURCE = next(ast.literal_eval(node.value) for node in ast.parse((BASE/'check.py').read_text()).body
              if isinstance(node, ast.Assign) and any(isinstance(t, ast.Name) and t.id=='SOURCE' for t in node.targets))


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def digest(text):
    return hashlib.sha256(text.encode()).hexdigest()


def snapshot():
    paths = [HERE/n for n in ['check.py', 'fixtures.py', 'cases.json', 'regression-cases.json']]
    paths += [BASE/n for n in ['check.py', 'fixtures.py', 'baseline_fixtures.py', 'baseline_cases.json']]
    paths += [ROOT/p for p in SOURCE]
    paths += [ROOT/'tools/ports/acpi/interpreter/execution'/n for n in ['checked_runner.rs', 'runner.Cargo.lock']]
    return {str(p.relative_to(ROOT)): sha(p) for p in paths}


def build():
    return ('machine build(builder:&mut Build){builder.application("cathedral-named-buffer-store-checks");'
            'builder.freestanding=true;builder.depend_as("aml",Source::Path {location:"'+str(ROOT/'source/libraries/acpi/aml')+'"});'
            'builder.depend_as("integer_helpers",Source::Path {location:"'+str(ROOT/'source/libraries/acpi/interpreter')+'"});}')


def authored(rows):
    text = IMPORTS+HELPERS+'data Suite{}\n'
    selections = []
    for row in rows:
        for control in [False, True]:
            name = 'Suite::'+row['name']+('_control' if control else '_positive')
            selections.append(name+'='+str(int(control)))
            text += 'machine '+name+'(&mut self)->i32{'+body(row,control)+'}\n'
    return text, selections


def validate(output, selections):
    assert output.count('CHECKED authored package and dependency bodies;') == 1
    actual = re.findall(r'^PASS (\S+) expected=(\d+) observed=(\d+) error=None usage=', output, re.M)
    assert len(actual) == len(selections), output
    for (name, want, got), selection in zip(actual, selections):
        assert name+'='+want == selection and want == got


def checked_batch(rows):
    source, selections = authored(rows)
    start = time.monotonic()
    with tempfile.TemporaryDirectory(prefix='cathedral-named-buffer-store-') as directory:
        work = Path(directory)
        (work/'build.omg').write_text(build())
        (work/'main.omg').write_text(source)
        run = subprocess.run([str(RUNNER), str(work/'main.omg'), str(work/'build'), *selections],
                             capture_output=True, text=True,
                             env=dict(os.environ, OMEGA_INTERP_STEP_BUDGET='10000000'))
    output = run.stdout+run.stderr
    return {'cases': [r['name'] for r in rows], 'source_sha256': digest(source),
            'selections': selections, 'exit_code': run.returncode, 'output': output,
            'seconds': round(time.monotonic()-start, 3)}


def run(rows, record, workers):
    inputs = snapshot()
    runner = sha(RUNNER)
    start = time.monotonic()
    batches = []
    with ThreadPoolExecutor(max_workers=workers) as pool:
        for batch in pool.map(checked_batch, [rows[i:i+10] for i in range(0,len(rows),10)]):
            batches.append(batch)
            print('COMPLETED', sum(len(b['cases']) for b in batches), '/', len(rows), 'pairs', flush=True)
    receipt = dict(omega_revision=PIN, input_sha256=inputs, runner_sha256=runner,
                   execution_root=str(ROOT), build_text=build(), build_sha256=digest(build()),
                   workers=workers, positive_count=len(rows), control_count=len(rows),
                   batches=batches, source_unchanged=inputs==snapshot(), runner_unchanged=runner==sha(RUNNER),
                   seconds=round(time.monotonic()-start,3))
    record.write_text(json.dumps(receipt,indent=2,sort_keys=True)+'\n')
    verify(rows, record)


def verify(rows, record):
    receipt = json.loads(record.read_text())
    assert receipt['omega_revision'] == PIN
    assert receipt['execution_root'] == str(ROOT) and receipt['build_text'] == build()
    assert receipt['build_sha256'] == digest(build())
    assert receipt['source_unchanged'] and receipt['runner_unchanged']
    assert receipt['input_sha256'] == snapshot() and receipt['runner_sha256'] == sha(RUNNER)
    assert receipt['positive_count'] == receipt['control_count'] == len(rows)
    lookup = {row['name']: row for row in rows}
    seen = []
    for batch in receipt['batches']:
        source, selections = authored([lookup[name] for name in batch['cases']])
        assert batch['source_sha256'] == digest(source) and batch['selections'] == selections
        assert batch['exit_code'] == 0, batch['output']
        validate(batch['output'], selections)
        seen += batch['cases']
    assert seen == [r['name'] for r in rows]
    print('PASS', len(rows), 'checked behavior/control pairs; exact source and runner hashes')


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--group', choices=['buffer','regression'], default='buffer')
    parser.add_argument('--match')
    parser.add_argument('--record', type=Path)
    parser.add_argument('--verify-record', action='store_true')
    parser.add_argument('--write-cases', action='store_true')
    parser.add_argument('--workers', type=int, choices=[1,2,3], default=2)
    args = parser.parse_args()
    groups = {'buffer': ('cases.json', cases()), 'regression': ('regression-cases.json', regression_cases())}
    if args.write_cases:
        for name, rows in groups.values():
            (HERE/name).write_text(json.dumps(rows,indent=2,sort_keys=True)+'\n')
    else:
        for name, rows in groups.values():
            assert json.loads((HERE/name).read_text()) == rows
        rows = groups[args.group][1]
        if args.match:
            rows = [r for r in rows if args.match in r['name']]
        assert rows
        record = args.record or HERE/(args.group+'-verification.json')
        if args.verify_record:
            verify(rows, record)
        else:
            run(rows, record, args.workers)
