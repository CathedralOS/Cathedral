#!/usr/bin/env python3
"""Verify committed milestone inputs; no compiler or runtime execution."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[6]
DIRECTORY = 'tools/ports/acpi/interpreter/generic-execution/'
CHECKPOINT = 'c4a8b03e2ff50b8e0eb69231f80e31554511291f'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--revision', default=CHECKPOINT)
    args = parser.parse_args()
    revision = subprocess.check_output(
        ['git', '-C', str(ROOT), 'rev-parse', '--verify', args.revision + '^{commit}'],
        text=True,
    ).strip()
    cache = {}

    def blob(path):
        assert not Path(path).is_absolute() and '..' not in Path(path).parts
        if path not in cache:
            cache[path] = subprocess.check_output(
                ['git', '-C', str(ROOT), 'show', revision + ':' + path]
            )
        return cache[path]

    def verify(mapping):
        for path, expected in mapping.items():
            assert hashlib.sha256(blob(path)).hexdigest() == expected, path

    manifest = json.loads(blob(DIRECTORY + 'manifest.json'))
    for key in ['production_source_sha256', 'evidence_verifier_sha256', 'owned_document_sha256']:
        verify(manifest[key])
    for archive in manifest['historical_archives']:
        verify({DIRECTORY + archive['path']: archive['sha256']})
    pairs = 0
    constants = 0
    for item in manifest['current_receipts']:
        path = DIRECTORY + item['path']
        verify({path: item['sha256']})
        verify(item['fixture_inputs_sha256'])
        receipt = json.loads(blob(path))
        assert receipt['execution_root'] == manifest['execution_root']
        assert hashlib.sha256(receipt['build_source'].encode()).hexdigest() == receipt['build_sha256']
        assert manifest['execution_root'] in receipt['build_source']
        snapshot = receipt.get('source_sha256', receipt.get('source_snapshot_sha256'))
        assert snapshot
        verify(snapshot)
        if item['stage'] == 'constant evaluator':
            positive, control = receipt['results']
            assert positive['exit_code'] == 0 and control['exit_code'] != 0
            assert 'require' in control['output'].lower()
            constants += item['pairs']
        else:
            assert receipt.get('exit_code', 0) == 0 and receipt.get('source_unchanged', True)
            lines = receipt['output'].splitlines()
            assert sum(line.startswith('PASS ') for line in lines) == 2 * item['pairs']
            assert not any(line.startswith('FAIL ') for line in lines)
            pairs += item['pairs']
    print(f'PASS committed inputs: {revision}; {len(cache)} files; {pairs} checked pairs, {constants} const pair(s)')
    print('Historical receipt integrity only; no current-source or new execution claim')


if __name__ == '__main__':
    main()
