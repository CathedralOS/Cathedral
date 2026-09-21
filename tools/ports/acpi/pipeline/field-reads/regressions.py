#!/usr/bin/env python3
"""Separate current-source execution regressions for the synthetic Field read draft."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time

PIN = 'eaa7993a23623cd8fabf45350340479c5c9c7879'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def load(name, path):
    sys.path.insert(0, str(path.parent))
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def paths(root):
    base = root / 'tools/ports/acpi'
    return sorted(set((root / 'source/libraries/acpi').rglob('*.omg')) | {
        base / 'interpreter/execution/fixtures.py',
        base / 'interpreter/execution/checked_runner.rs',
        base / 'interpreter/execution/runner.Cargo.lock',
        base / 'interpreter/generic-execution/fixtures.py',
        base / 'interpreter/generic-execution/decoder_fixtures.py',
        base / 'interpreter/generic-execution/focused/bridge-atomicity/main.omg',
        base / 'interpreter/named-store-execution/fixtures.py',
        base / 'pipeline/fixtures.py',
    })


def snapshot(root):
    return {str(path.relative_to(root)): digest(path.read_bytes()) for path in paths(root)}


def generate(root):
    base = root / 'tools/ports/acpi'
    integer = load('read_regression_integer', base / 'interpreter/execution/fixtures.py')
    generic = load('read_regression_generic', base / 'interpreter/generic-execution/fixtures.py')
    pipeline = load('read_regression_pipeline', base / 'pipeline/fixtures.py')
    named = load('read_regression_named', base / 'interpreter/named-store-execution/fixtures.py')
    groups = []

    rows = integer.cases()
    first = integer.render(rows[0], False)
    source = first[:first.index('machine test_result()')] + 'data IntegerSuite {}\n'
    selections = []
    for row in rows:
        rendered = integer.render(row, False)
        body = rendered[rendered.index('machine test_result()'):rendered.index('data Main {}')]
        for control in (False, True):
            value = body
            if control:
                number = row['expected'] is not None and row['error'] == 'Success'
                marker = f'result.value.number == {row["expected"]}' if number else f'result.outcome == ExecutionOutcome::{row["error"]}'
                replacement = f'result.value.number == {(row["expected"] + 1) & ((1 << row["bits"]) - 1)}' if number else f'result.outcome == ExecutionOutcome::{"BadEncoding" if row["error"] == "Success" else "Success"}'
                assert value.count(marker) == 1
                value = value.replace(marker, replacement)
            entry = 'IntegerSuite::' + row['name'] + ('_control' if control else '_positive')
            source += value.replace('machine test_result()', f'machine {entry}(&mut self)', 1)
            selections.append(entry + '=' + str(int(control)))
    groups.append(('integer_regression', source, selections, [row['name'] for row in rows]))

    rows = generic.cases()
    source, selections = generic.render(rows)
    source = re.sub(r'\bSuite\b', 'GenericSuite', source)
    selections = [re.sub(r'\bSuite\b', 'GenericSuite', name) for name in selections]
    groups.append(('generic_regression', source, selections, [row['name'] for row in rows]))

    rows = pipeline.cases()
    source = pipeline.IMPORTS + 'data PipelineSuite {}\n'
    selections = []
    for row in rows:
        for control in (False, True):
            entry = 'PipelineSuite::' + row['name'] + ('_control' if control else '_positive')
            source += pipeline.render(row, control, entry)
            selections.append(entry + '=' + str(int(control)))
    groups.append(('pipeline_regression', source, selections, [row['name'] for row in rows]))

    # Seed active read metadata before the existing expected-frame copies. These
    # are the old bridge assertions with additional initialized Frame state.
    seed = 'frame.field_reads=true;frame.deferred_read=DeferredRead::Field {object:2,at:17,next:23};'
    def seeded(source):
        marker = 'let mut expected_store:ObjectStore=ObjectStore {space:store.space,bytes:store.bytes};let mut expected_frame:Frame=frame;'
        assert source.count(marker) > 0
        return source.replace(marker, seed + marker)

    source = (base / 'interpreter/generic-execution/focused/bridge-atomicity/main.omg').read_text()
    source = seeded(source)
    names = re.findall(r'machine Suite::(\w+)\(&mut self\)->i32', source)
    selections = ['BridgeSuite::' + name + '=' + name.rsplit('_', 1)[1] for name in names]
    assert all(name.rsplit('_', 1)[1] in ('0', '1') for name in names)
    source = re.sub(r'\bSuite\b', 'BridgeSuite', source)
    groups.append(('bridge_regression', source, selections, names[::2]))

    rows, source, selections = named.render('bridge')
    source = re.sub(r'\bSuite\b', 'NamedBridgeSuite', seeded(source))
    selections = [re.sub(r'\bSuite\b', 'NamedBridgeSuite', name) for name in selections]
    groups.append(('named_bridge_regression', source, selections, [row['name'] for row in rows]))

    # Independently show that the new comparator observes every continuation
    # member, while the actual null Store bridge preserves the seeded frame.
    original = (base / 'interpreter/generic-execution/focused/bridge-atomicity/main.omg').read_text()
    prefix = original[:original.index('machine Suite::atomic_uninitialized_0')]
    match = re.search(r'machine Suite::atomic_null_store_0\(&mut self\)->i32 \{.*?\n\}', original, re.S)
    assert match
    body = seeded(match.group())
    anchor = 'let result:i32=ba_check('
    assert body.count(anchor) == 1
    changes = {
        'mode': 'expected_frame.field_reads=false;',
        'variant': 'expected_frame.deferred_read=DeferredRead::None;',
        'object': 'expected_frame.deferred_read=DeferredRead::Field {object:3,at:17,next:23};',
        'at': 'expected_frame.deferred_read=DeferredRead::Field {object:2,at:18,next:23};',
        'next': 'expected_frame.deferred_read=DeferredRead::Field {object:2,at:17,next:24};',
    }
    source, selections = prefix, []
    for label, change in changes.items():
        for control in (False, True):
            entry = 'DeferredBridgeSuite::' + label + ('_control' if control else '_positive')
            value = body.replace('Suite::atomic_null_store_0', entry)
            if control:
                value = value.replace(anchor, change + anchor)
            source += value + '\n'
            selections.append(entry + '=' + str(int(control)))
    source = re.sub(r'\bSuite\b', 'DeferredBridgeSuite', source)
    groups.append(('deferred_bridge_regression', source, selections, list(changes)))

    sources = {name + '.omg': 'module ' + name + ';\n' + source for name, source, _, _ in groups}
    sources['main.omg'] = ''.join('use ' + name + ';\n' for name, _, _, _ in groups)
    build = 'machine build(builder:&mut Build){builder.application("cathedral-field-read-regressions");builder.freestanding=true;'
    for alias, relative in [('aml', 'aml'), ('execution', 'interpreter/execution'), ('integer_helpers', 'interpreter'), ('pipeline', 'pipeline')]:
        build += f'builder.depend_as("{alias}",Source::Path {{location:"{root / "source/libraries/acpi" / relative}"}});'
    sources['build.omg'] = build + '}\n'
    selections = [entry for _, _, entries, _ in groups for entry in entries]
    assert len(selections) == len(set(selections))
    counts = {name: len(entries) // 2 for name, _, entries, _ in groups}
    cases = {name: names for name, _, _, names in groups}
    return sources, selections, counts, cases


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path('/tmp/cathedral-field-reads-current'))
    parser.add_argument('--runner', type=Path, default=Path('/tmp/cathedral-field-protocol-target/release/cathedral-acpi-checked-runner'))
    parser.add_argument('--omega-source', type=Path, default=Path('/tmp/cathedral-field-protocol-omega'))
    parser.add_argument('--record', type=Path)
    parser.add_argument('--verify', type=Path)
    parser.add_argument('--prepare-only', action='store_true')
    args = parser.parse_args()
    root = args.root.resolve()
    sources, selections, counts, cases = generate(root)
    source_hashes = {name: digest(source.encode()) for name, source in sources.items()}
    if args.prepare_only:
        print(json.dumps(dict(counts=counts, scenarios=sum(counts.values()), selections=len(selections), source_sha256=source_hashes), indent=2))
        return
    if args.verify:
        record = json.loads(args.verify.read_text())
        assert record['format'] == 'cathedral-field-read-regressions-v1'
        assert record['source_sha256'] == snapshot(root)
        assert record['harness_sha256'] == digest(Path(__file__).read_bytes())
        assert record['generated_source_sha256'] == source_hashes
        assert record['generated_sources'] == sources
        assert record['selections'] == selections and record['counts'] == counts and record['cases'] == cases
        assert record['source_unchanged'] and record['omega_revision'] == PIN and record['exit_code'] == 0
        assert 'CHECKED authored package and dependency bodies' in record['output']
        assert sum(line.startswith('PASS ') for line in record['output'].splitlines()) == len(selections)
        for entry in selections:
            name, expected = entry.split('=')
            assert f'PASS {name} expected={expected} observed={expected} error=None ' in record['output']
        print('Verified', sum(counts.values()), 'current-source regression pairs', counts)
        return
    assert args.record, '--record is required for execution'
    runner, omega = args.runner.resolve(), args.omega_source.resolve()
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=omega, text=True).strip()
    assert revision == PIN
    assert not subprocess.check_output(['git', 'status', '--porcelain'], cwd=omega, text=True).strip()
    before = snapshot(root)
    harness_hash = digest(Path(__file__).read_bytes())
    runner_hash = digest(runner.read_bytes())
    print('Counts:', counts, 'total pairs:', sum(counts.values()), flush=True)
    with tempfile.TemporaryDirectory(prefix='cathedral-read-regression-') as directory:
        work = Path(directory)
        for name, source in sources.items():
            (work / name).write_text(source)
        command = [str(runner), str(work / 'main.omg'), str(work / 'build'), *selections]
        start = time.monotonic()
        process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                   text=True, env=dict(os.environ, OMEGA_INTERP_STEP_BUDGET='10000000'))
        print('Runner PID:', process.pid, 'package:', work, flush=True)
        lines = []
        for line in process.stdout:
            print(line, end='', flush=True)
            lines.append(line)
        code = process.wait()
        elapsed = time.monotonic() - start
    output = ''.join(lines)
    unchanged = before == snapshot(root) and harness_hash == digest(Path(__file__).read_bytes()) and runner_hash == digest(runner.read_bytes())
    assert revision == subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=omega, text=True).strip()
    assert not subprocess.check_output(['git', 'status', '--porcelain'], cwd=omega, text=True).strip()
    record = dict(format='cathedral-field-read-regressions-v1', execution_root=str(root),
                  stage='checked interpretation; no native/hardware execution',
                  omega_revision=revision, omega_source=str(omega), runner=str(runner),
                  runner_sha256=runner_hash, harness=str(Path(__file__).resolve()), harness_sha256=harness_hash,
                  source_sha256=before, source_unchanged=unchanged, generated_source_sha256=source_hashes,
                  generated_sources=sources, selections=selections, counts=counts, cases=cases,
                  command=command, exit_code=code, elapsed_seconds=round(elapsed, 3), output=output,
                  scenario_count=sum(counts.values()), control_count=sum(counts.values()), evaluator_step_limit=10000000,
                  bridge_note='Existing bridge assertions seed enabled mode and a pending read; five added controls independently cover every new Frame member.')
    args.record.write_text(json.dumps(record, indent=2, sort_keys=True) + '\n')
    assert unchanged, 'Source/harness/runner changed during verification'
    assert code == 0, 'Checked regression process failed'
    assert 'CHECKED authored package and dependency bodies' in output
    assert sum(line.startswith('PASS ') for line in lines) == len(selections)
    print('PASS', sum(counts.values()), 'current-source behavior/control pairs')


if __name__ == '__main__':
    main()
