#!/usr/bin/env python3
"""Verify the retained source-bound scratch milestone; does not execute Omega."""
import argparse
import hashlib
import json
import re
from fixtures import reproduce
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
ROOT = HERE.parents[4]

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=ROOT)
    args = parser.parse_args()
    manifest = json.loads((HERE / 'manifest.json').read_text())
    assert manifest.get('current_receipts'), 'Current replay has no completed receipts'
    assert Path(manifest['execution_root']).resolve() == args.root.resolve()
    for group in ['evidence_verifier_sha256', 'owned_document_sha256']:
        for relative, expected in manifest[group].items():
            assert sha(args.root / relative) == expected, relative
    for relative, expected in manifest['production_source_sha256'].items():
        assert sha(args.root / relative) == expected, relative
    for item in manifest['current_receipts']:
        for relative, expected in item['fixture_inputs_sha256'].items():
            assert sha(args.root / relative) == expected, relative
        path = HERE / item['path']
        assert sha(path) == item['sha256'], path
        receipt = json.loads(path.read_text())
        assert Path(receipt['execution_root']).resolve() == args.root.resolve(), path.name
        build = receipt['build_source']
        assert hashlib.sha256(build.encode()).hexdigest() == receipt['build_sha256'], path.name
        assert str(args.root.resolve()) in build, path.name
        snapshot = receipt.get('source_sha256', receipt.get('source_snapshot_sha256', {}))
        assert snapshot
        for relative, expected in snapshot.items():
            assert sha(args.root / relative) == expected, (path.name, relative)
        if item['stage'] == 'constant evaluator':
            results = receipt['results']
            assert len(results) == 2 and results[0]['exit_code'] == 0 and results[1]['exit_code'] != 0
            assert 'require' in results[1]['output'].lower()
            sources, selections = reproduce(args.root, item['fixture_kind'], receipt)
            assert [hashlib.sha256(source.encode()).hexdigest() for source in sources] == [result['fixture_sha256'] for result in results]
        else:
            assert receipt.get('exit_code', 0) == 0
            assert receipt.get('source_unchanged', True)
            output = receipt['output']
            assert sum(line.startswith('PASS ') for line in output.splitlines()) == 2 * item['pairs']
            assert not any(line.startswith('FAIL ') for line in output.splitlines())
            sources, selections = reproduce(args.root, item['fixture_kind'], receipt)
            assert len(sources) == 1
            assert hashlib.sha256(sources[0].encode()).hexdigest() == receipt['fixture_sha256']
            assert receipt['command'][3:] == selections
            observed = re.findall(r'^PASS (\S+) expected=(\d+) observed=(\d+) error=None', output, re.M)
            assert [(name, expected) for name, expected, actual in observed] == [tuple(selection.rsplit('=', 1)) for selection in selections]
            assert all(expected == actual for name, expected, actual in observed)
        print('PASS retained evidence:', item['path'], item['pairs'], 'pair(s)')
    print('PASS source/receipt integrity only; no new compiler or runtime execution')

if __name__ == '__main__':
    main()
