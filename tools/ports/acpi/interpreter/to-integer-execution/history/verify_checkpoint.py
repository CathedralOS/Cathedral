#!/usr/bin/env python3
"""Reproduce source-bound ToInteger evidence from an explicit Git checkpoint.

This verifies archived inputs and observations; it does not execute Omega.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path, PurePosixPath
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[6]
DIRECTORY = 'tools/ports/acpi/interpreter/to-integer-execution/'
RUNNER = '19e3f01dff2e6d7fbc9a1ee04843c4075b63a2e3c83b23f15203b6a58e9fec3a'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-ref', required=True)
    parser.add_argument('--group', choices=['bridge', 'execution'], default='bridge')
    args = parser.parse_args()
    revision = subprocess.check_output(
        ['git', '-C', str(ROOT), 'rev-parse', '--verify', '--end-of-options',
         args.source_ref + '^{commit}'], text=True).strip()

    def blob(path):
        relative = PurePosixPath(path)
        assert not relative.is_absolute() and '..' not in relative.parts
        return subprocess.check_output(['git', '-C', str(ROOT), 'show', revision + ':' + path])

    record = json.loads(blob(DIRECTORY + args.group + '-verification.json'))
    assert record['omega_revision'] == 'eaa7993a23623cd8fabf45350340479c5c9c7879'
    assert record['runner_sha256'] == RUNNER
    assert record['source_unchanged'] and record['exit_code'] == 0
    assert record['stage'] == 'checked interpreter' and not record['native_execution']
    assert record['group'] == args.group and record['match'] == ''
    sys.dont_write_bytecode = True
    with tempfile.TemporaryDirectory(prefix='cathedral-to-integer-history-') as folder:
        archive = Path(folder)
        for path, expected in record['source_sha256'].items():
            data = blob(path)
            assert digest(data) == expected, path
            destination = archive / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(data)
        source = archive / DIRECTORY / 'check.py'
        sys.path.insert(0, str(source.parent))
        spec = importlib.util.spec_from_file_location('archived_to_integer_check', source)
        check = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(check)
        assert check.snapshot() == record['source_sha256']
        rows, generated, selections = check.fixtures.render(args.group, '')
        assert [row['name'] for row in rows] == record['cases']
        assert len(set(record['cases'])) == len(rows)
        assert len(selections) == 2 * len(rows) == len(set(selections))
        assert [item.rsplit('=', 1)[1] for item in selections] == ['0', '1'] * len(rows)
        assert record['selections'] == record['command'][3:] == selections
        assert digest(generated.encode()) == record['fixture_sha256']
        build = check.build(Path(record['production_root']))
        assert build == record['build_source'] and digest(build.encode()) == record['build_sha256']
        check.validate(record['output'], selections)
        print('PASS historical', args.group, len(rows), 'pairs;',
              len(record['source_sha256']), 'Git inputs at', revision)
    print('Retained evidence verification only; no fresh execution.')


if __name__ == '__main__':
    main()
