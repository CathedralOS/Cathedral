#!/usr/bin/env python3
"""Verify historical named Store receipts against one explicit Git checkpoint.

This reconstructs authored inputs and checks retained observations. It does not
invoke Omega, execute AML, or claim that the current checkout was verified.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[6]
DIRECTORY = 'tools/ports/acpi/interpreter/named-store-execution/'
GENERIC = 'tools/ports/acpi/interpreter/generic-execution/'
INTEGER = 'tools/ports/acpi/interpreter/execution/'
PIPELINE = 'tools/ports/acpi/pipeline/'
PIN = 'eaa7993a23623cd8fabf45350340479c5c9c7879'
RUNNER_SHA256 = '19e3f01dff2e6d7fbc9a1ee04843c4075b63a2e3c83b23f15203b6a58e9fec3a'
RECEIPTS = (
    ('execution-verification.json', 'named-execution', 44),
    ('bridge-verification.json', 'named-bridge', 30),
    ('target-actions-regression.json', 'focused', 25),
    ('generic-regression.json', 'generic', 55),
    ('integer-regression.json', 'integer', 79),
    ('pipeline-regression.json', 'pipeline', 22),
)


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    previous = sys.path[:]
    try:
        sys.path.insert(0, str(path.parent))
        spec.loader.exec_module(module)
    finally:
        sys.path[:] = previous
    return module


def reproduce(archive, kind, receipt):
    """Use generator bodies from the checkpoint, never the live checkout."""
    if kind.startswith('named-'):
        fixtures = load(archive / DIRECTORY / 'fixtures.py', 'archived_named_fixtures')
        group = kind.removeprefix('named-')
        require(receipt['group'] == group and receipt['match'] == '', 'named suite selection changed')
        rows, source, selections = fixtures.render(group, '')
        require([row['name'] for row in rows] == receipt['cases'], 'named cases changed')
        return source, selections
    helper = load(archive / GENERIC / 'evidence/fixtures.py', 'archived_evidence_fixtures')
    if kind == 'focused':
        require(receipt['group'] == 'target-actions', 'focused suite selection changed')
    sources, selections = helper.reproduce(archive, kind, receipt)
    require(len(sources) == 1, 'expected one checked fixture')
    return sources[0], selections


def reproduce_build(archive, kind, execution_root):
    root = Path(execution_root)
    require(root.is_absolute(), 'recorded execution root must be absolute')
    if kind in ('integer', 'pipeline'):
        directory = INTEGER if kind == 'integer' else PIPELINE
        source = (archive / directory / 'build.omg').read_text()
        relative_prefix = '../../../../../' if kind == 'integer' else '../../../../'
        packages = ['source/libraries/acpi/interpreter/execution',
                    'source/libraries/acpi/interpreter', 'source/libraries/acpi/aml']
        if kind == 'pipeline':
            packages = ['source/libraries/acpi/pipeline', 'source/libraries/acpi/aml',
                        'source/libraries/acpi/interpreter/execution',
                        'source/libraries/acpi/interpreter']
        for package in packages:
            source = source.replace(relative_prefix + package, str(root / package))
        return source
    package = {'focused': 'generic-focused', 'generic': 'cathedral-acpi-generic-fixtures'}.get(kind, 'named-store-execution')
    source = f'machine build(builder:&mut Build){{builder.package("{package}");builder.freestanding=true;'
    for alias, relative in [('aml', 'source/libraries/acpi/aml'),
                            ('execution', 'source/libraries/acpi/interpreter/execution'),
                            ('integer_helpers', 'source/libraries/acpi/interpreter'),
                            ('pipeline', 'source/libraries/acpi/pipeline')]:
        source += f'builder.depend_as("{alias}",Source::Path {{location:"{root / relative}"}});'
    return source + '}\n'


def verify_observations(receipt, selections, pairs):
    require(receipt.get('exit_code', 0) == 0, 'receipt records execution failure')
    require(receipt.get('source_unchanged') is True, 'receipt does not attest unchanged inputs')
    require(receipt.get('native_execution', False) is False, 'unexpected native execution claim')
    require(receipt['stage'] in ('checked interpreter', 'checked-interpreter execution; native/hardware not run'), 'unexpected execution stage')
    require(len(selections) == pairs * 2 and len(set(selections)) == pairs * 2, 'selection count/uniqueness')
    require([item.rsplit('=', 1)[1] for item in selections] == ['0', '1'] * pairs, 'behavior/control pairing')
    require(receipt['command'][3:] == selections, 'command selections differ from authored fixtures')
    if 'selections' in receipt:
        require(receipt['selections'] == selections, 'recorded selections differ')
    for key in ('scenarios', 'controls', 'scenario_count', 'control_count'):
        if key in receipt:
            require(receipt[key] == pairs, key + ' count differs')
    if 'cases' in receipt:
        require(len(receipt['cases']) == pairs and len(set(receipt['cases'])) == pairs, 'case count/uniqueness')
    output = receipt['output']
    require(output.count('CHECKED authored package and dependency bodies;') == 1, 'checked-admission observation missing/duplicated')
    lines = [line for line in output.splitlines() if line.startswith('PASS ')]
    require(len(lines) == len(selections), 'PASS observation count differs')
    require(not any(line.startswith(('FAIL ', 'SKIP ')) for line in output.splitlines()), 'failed/skipped observation')
    for line, selection in zip(lines, selections):
        found = re.fullmatch(r'PASS (\S+) expected=(\d+) observed=(\d+) error=None usage=EvaluationUsage \{ (.+) \}', line)
        require(found is not None, 'malformed or errored observation: ' + selection)
        name, expected, actual, usage = found.groups()
        require(name + '=' + expected == selection and actual == expected, 'observation mismatch: ' + selection)
        require('schema: EvaluationUsageSchemaIdentity(7)' in usage and
                'schedule: EvaluationStepScheduleIdentity(1)' in usage and
                'fuel_ceiling: 10000000' in usage, 'unexpected evaluator identity/budget')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-ref', required=True, help='Git commit containing the completed receipts and their source checkpoint')
    parser.add_argument('--integrated', action='store_true', help='Verify the later 58/38 combined-source milestone')
    args = parser.parse_args()
    suites = ((
        ('integrated-execution-verification.json', 'named-execution', 58),
        ('integrated-bridge-verification.json', 'named-bridge', 38),
    ) if args.integrated else RECEIPTS)
    revision = subprocess.check_output(
        ['git', '-C', str(ROOT), 'rev-parse', '--verify', '--end-of-options', args.source_ref + '^{commit}'],
        text=True).strip()
    cache = {}

    def blob(path):
        relative = PurePosixPath(path)
        require(not relative.is_absolute() and '..' not in relative.parts, 'unsafe recorded path')
        if path not in cache:
            cache[path] = subprocess.check_output(['git', '-C', str(ROOT), 'show', revision + ':' + path])
        return cache[path]

    records = [(name, kind, pairs, json.loads(blob(DIRECTORY + name))) for name, kind, pairs in suites]
    toolchain = json.loads(blob(DIRECTORY + 'toolchain.json'))
    require(toolchain['source_revision'] == PIN and toolchain['source_clean'] is True, 'toolchain source pin/cleanliness')
    require(toolchain['runner_sha256'] == RUNNER_SHA256 and toolchain['smoke_passed'] is True, 'audited runner identity')
    require(digest(blob(INTEGER + 'checked_runner.rs')) == toolchain['runner_source_sha256'], 'runner source identity')
    require(digest(blob(INTEGER + 'runner.Cargo.lock')) == toolchain['runner_lock_sha256'], 'runner dependency identity')
    roots = set()
    for name, kind, pairs, receipt in records:
        root = receipt.get('execution_root', receipt.get('production_root'))
        require(root is not None, name + ': missing execution root')
        roots.add(root)
        require(receipt.get('omega_revision', PIN) == PIN, name + ': Omega pin differs')
        require(receipt.get('runner_sha256', receipt.get('binary_sha256')) == RUNNER_SHA256, name + ': runner differs')
        require(Path(receipt['command'][0]).name == Path(toolchain['runner']).name, name + ': runner command differs')
        if 'cargo_lock_sha256' in receipt:
            require(receipt['cargo_lock_sha256'] == toolchain['runner_lock_sha256'], name + ': dependency lock differs')
        if 'rustc' in receipt:
            require(receipt['rustc'] == toolchain['rustc'], name + ': rustc differs')
        mapping = receipt['source_sha256']
        require(bool(mapping), name + ': empty source snapshot')
        for path, expected in mapping.items():
            require(digest(blob(path)) == expected, name + ': source hash differs: ' + path)
    require(len(roots) == 1, 'receipts describe different execution roots')
    execution_root = roots.pop()
    # The historical generic reproducer also checks its checked-in fixture.
    for path in [GENERIC + 'evidence/fixtures.py', GENERIC + 'main.omg']:
        blob(path)
    sys.dont_write_bytecode = True
    with tempfile.TemporaryDirectory(prefix='cathedral-named-store-history-') as directory:
        archive = Path(directory)
        for path, data in cache.items():
            target = archive / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        for name, kind, pairs, receipt in records:
            source, selections = reproduce(archive, kind, receipt)
            require(digest(source.encode()) == receipt['fixture_sha256'], name + ': generated fixture differs')
            build = reproduce_build(archive, kind, execution_root)
            require(build == receipt['build_source'] and digest(build.encode()) == receipt['build_sha256'], name + ': generated build differs')
            verify_observations(receipt, selections, pairs)
            print(f'PASS historical receipt: {name}; {pairs} behavior/control pairs')
    total = sum(pairs for _, _, pairs in suites)
    require(total == (96 if args.integrated else 255), 'milestone pair count changed')
    print(f'PASS checkpoint {revision}: {len(suites)} receipts, {total} behavior/control pairs; {len(cache)} Git blobs verified')
    print('Historical input/observation integrity only; no current-source or new compiler/runtime execution claim')


if __name__ == '__main__':
    main()
