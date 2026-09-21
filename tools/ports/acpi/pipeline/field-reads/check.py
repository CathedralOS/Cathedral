#!/usr/bin/env python3
"""Check and interpret the authored resumable read fixtures; no native execution."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

import fixtures

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
PIN = 'eaa7993a23623cd8fabf45350340479c5c9c7879'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def snapshot():
    paths = list((ROOT / 'source/libraries/acpi').rglob('*.omg'))
    paths += [HERE / 'fixtures.py', HERE / 'check.py',
              ROOT / 'tools/ports/acpi/interpreter/execution/checked_runner.rs',
              ROOT / 'tools/ports/acpi/interpreter/execution/runner.Cargo.lock']
    return {str(p.relative_to(ROOT)): digest(p.read_bytes()) for p in sorted(paths)}


def build_source():
    base = ROOT / 'source/libraries/acpi'
    packages = {'pipeline': base / 'pipeline', 'aml': base / 'aml',
                'execution': base / 'interpreter/execution',
                'integer_helpers': base / 'interpreter'}
    return ('machine build(builder:&mut Build){\n'
            'builder.application("cathedral-field-read-tests");builder.freestanding=true;\n'
            + ''.join(f'builder.depend_as("{name}",Source::Path {{location:"{path}"}});\n'
                      for name, path in packages.items()) + '}\n')


def render(rows):
    bodies = [fixtures.IMPORTS, fixtures.HELPERS, 'data FieldReadSuite {}\n']
    entries = []
    for row in rows:
        for control in (False, True):
            entry = 'FieldReadSuite::' + row['name'] + ('_control' if control else '_positive')
            bodies.append(fixtures.render(row, control, entry))
            entries.append(entry + '=' + str(int(control)))
    return ''.join(bodies), entries


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--runner', type=Path)
    parser.add_argument('--omega-source', type=Path)
    parser.add_argument('--match', default='')
    parser.add_argument('--record', type=Path)
    parser.add_argument('--verify', type=Path)
    args = parser.parse_args()
    all_rows = fixtures.cases()
    if args.verify:
        record = json.loads(args.verify.read_text())
        rows = [row for row in all_rows if row['name'] in record['cases']]
        source, entries = render(rows)
        assert record['format'] == 'cathedral-field-read-checked-v1'
        assert [row['name'] for row in rows] == record['cases']
        assert record['source_sha256'] == snapshot()
        assert record['fixture_sha256'] == digest(source.encode())
        assert record['build_source'] == build_source()
        assert record['build_sha256'] == digest(build_source().encode())
        assert record['selections'] == entries
        assert record['scenario_count'] == record['control_count'] == len(rows)
        assert record['source_unchanged'] and record['omega_revision'] == PIN
        output = record['output']
        assert 'CHECKED authored package and dependency bodies' in output
        assert sum(line.startswith('PASS ') for line in output.splitlines()) == len(entries)
        for entry in entries:
            name, expected = entry.split('=')
            assert f'PASS {name} expected={expected} observed={expected} error=None ' in output
        print(f'Verified {len(rows)} retained read-session pairs and current inputs')
        return
    if not args.runner or not args.omega_source:
        parser.error('--runner and --omega-source are required for execution')
    runner = args.runner.resolve()
    omega = args.omega_source.resolve()
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=omega, text=True).strip()
    assert revision == PIN
    assert not subprocess.check_output(['git', 'status', '--porcelain'], cwd=omega, text=True).strip()
    rows = [row for row in all_rows if any(part in row['name'] for part in args.match.split(','))]
    assert rows, 'No selected fixtures'
    before = snapshot()
    runner_hash = digest(runner.read_bytes())
    source, entries = render(rows)
    build = build_source()
    with tempfile.TemporaryDirectory(prefix='cathedral-field-reads-') as directory:
        work = Path(directory)
        (work / 'main.omg').write_text(source)
        (work / 'build.omg').write_text(build)
        command = [str(runner), str(work / 'main.omg'), str(work / 'build'), *entries]
        started = time.monotonic()
        process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                   text=True, env=dict(os.environ, OMEGA_INTERP_STEP_BUDGET='10000000'))
        lines = []
        for line in process.stdout:
            print(line, end='', flush=True)
            lines.append(line)
        assert process.wait() == 0
        elapsed = time.monotonic() - started
    assert before == snapshot(), 'Inputs changed during execution'
    assert runner_hash == digest(runner.read_bytes())
    assert revision == subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=omega, text=True).strip()
    assert not subprocess.check_output(['git', 'status', '--porcelain'], cwd=omega, text=True).strip()
    output = ''.join(lines)
    assert sum(line.startswith('PASS ') for line in lines) == len(entries)
    if args.record:
        record = dict(format='cathedral-field-read-checked-v1',
                      stage='checked interpretation; synthetic caller completions; no native/hardware execution',
                      execution_root=str(ROOT), omega_revision=revision, omega_source=str(omega),
                      runner=str(runner), runner_sha256=runner_hash, source_sha256=before,
                      source_unchanged=True, fixture_sha256=digest(source.encode()),
                      build_source=build, build_sha256=digest(build.encode()),
                      selections=entries, command=command, cases=[row['name'] for row in rows],
                      scenario_count=len(rows), control_count=len(rows),
                      evaluator_step_limit=10000000, elapsed_seconds=round(elapsed, 3), output=output)
        args.record.write_text(json.dumps(record, indent=2, sort_keys=True) + '\n')
    print(f'PASS {len(rows)} read-session behavior/control pairs; hardware/native not run')


if __name__ == '__main__':
    main()
