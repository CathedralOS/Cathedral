#!/usr/bin/env python3
"""Execute actual Omega protocol bodies and changed-expectation controls."""
import argparse
import hashlib
import json
import os
import re
import subprocess
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import fixtures
import vectors

HERE, ROOT = fixtures.HERE, fixtures.ROOT
PIN = 'eaa7993a23623cd8fabf45350340479c5c9c7879'
DEFAULT_RUNNER = Path('/tmp/cathedral-field-protocol-target/release/cathedral-acpi-checked-runner')


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def snapshot():
    paths = list((ROOT/'source/libraries/acpi/field_protocol').glob('*.omg'))
    for package, files in {
        'field_access': ['build.omg', 'geometry.omg', 'chunks.omg', 'model.omg'],
        'aml/fields': ['build.omg', 'field_model.omg', 'flags.omg'],
        'aml': ['build.omg', 'model.omg'],
        'interpreter': ['build.omg', 'integers.omg'],
    }.items():
        paths += [ROOT/'source/libraries/acpi'/package/name for name in files]
    paths += [HERE/name for name in ['check.py', 'check_const.py', 'verify_record.py', 'fixtures.py', 'vectors.py', 'cases.json', 'toolchain.json']]
    paths += [ROOT/'tools/ports/acpi/interpreter/execution'/name for name in ['checked_runner.rs', 'runner.Cargo.lock']]
    return {str(path.relative_to(ROOT)): sha(path) for path in sorted(paths)}


def build_text():
    text = 'machine build(builder:&mut Build){builder.application("cathedral-field-protocol-tests");builder.freestanding=true;'
    for alias, folder in [('protocol', 'field_protocol'), ('access', 'field_access'), ('fields', 'aml/fields'), ('integers', 'interpreter'), ('aml', 'aml')]:
        text += 'builder.depend_as("'+alias+'",Source::Path {location:"'+str(ROOT/'source/libraries/acpi'/folder)+'"});'
    return text + '}'


def validate(output, selections):
    assert 'CHECKED authored package and dependency bodies;' in output
    actual = re.findall(r'^PASS (\S+) expected=(\d+) observed=(\d+) error=None usage=', output, re.M)
    assert len(actual) == len(selections), output
    for (name, expected, observed), selection in zip(actual, selections):
        assert name+'='+expected == selection and expected == observed


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--runner', type=Path, default=DEFAULT_RUNNER)
    parser.add_argument('--case', action='append')
    parser.add_argument('--batch-size', type=int, default=12)
    parser.add_argument('--workers', type=int, default=3)
    parser.add_argument('--record', type=Path, default=HERE/'checked-verification.json')
    args = parser.parse_args()
    rows = vectors.cases()
    assert json.loads((HERE/'cases.json').read_text()) == rows
    if args.case:
        rows = [row for row in rows if row['name'] in args.case]
        assert len(rows) == len(args.case)
    assert rows and args.batch_size > 0 and 1 <= args.workers <= 4
    inputs, binary = snapshot(), sha(args.runner)
    batches, start = [], time.monotonic()
    build = build_text()
    def run_batch(selected):
        batch_start = time.monotonic()
        text, names = fixtures.render(selected)
        with tempfile.TemporaryDirectory(prefix='cathedral-field-protocol-checked-') as directory:
            work = Path(directory)
            (work/'main.omg').write_text(text)
            (work/'build.omg').write_text(build)
            run = subprocess.run([str(args.runner), str(work/'main.omg'), str(work/'build'), *names], capture_output=True, text=True, env=dict(os.environ, OMEGA_INTERP_STEP_BUDGET='10000000'))
        output = run.stdout + run.stderr
        assert run.returncode == 0, output
        validate(output, names)
        assert inputs == snapshot() and binary == sha(args.runner), 'inputs changed during execution'
        return dict(cases=[row['name'] for row in selected], source_sha256=hashlib.sha256(text.encode()).hexdigest(), output=output, elapsed_seconds=round(time.monotonic()-batch_start, 3))
    groups = [rows[offset:offset+args.batch_size] for offset in range(0, len(rows), args.batch_size)]
    with ThreadPoolExecutor(max_workers=args.workers) as pool:
        for batch in pool.map(run_batch, groups):
            batches.append(batch)
            print(batch['output'], flush=True)
            print('PASS', sum(len(item['cases']) for item in batches), '/', len(rows), 'checked pairs', flush=True)
    record = dict(stage='checked interpreter; detached planning only; no native/hardware execution', omega_revision=PIN, execution_root=str(ROOT), build_text=build, build_sha256=hashlib.sha256(build.encode()).hexdigest(), input_sha256=inputs, runner_sha256=binary, runner_path=str(args.runner.resolve()), scope='selected' if args.case else 'full', cases=[row['name'] for row in rows], positive_count=len(rows), control_count=len(rows), workers=args.workers, elapsed_seconds=round(time.monotonic()-start, 3), batches=batches)
    args.record.write_text(json.dumps(record, indent=2, sort_keys=True)+'\n')
    print('PASS', len(rows), 'checked protocol pairs')


if __name__ == '__main__':
    main()
