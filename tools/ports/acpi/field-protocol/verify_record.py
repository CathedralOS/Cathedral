#!/usr/bin/env python3
"""Reproduce fixtures and bind retained results to exact current inputs."""
import argparse
import hashlib
import json
from collections import Counter
from pathlib import Path

import check
import check_const
import fixtures
import reference
import vectors

HERE = fixtures.HERE


def common(record):
    assert record['omega_revision'] == check.PIN
    assert record['input_sha256'] == check.snapshot()
    # Root may move after landing. Replay the exact recorded build rooted at
    # the original execution checkout; never reinterpret its absolute paths.
    build = check.build_text().replace(str(fixtures.ROOT), record['execution_root'])
    assert record['build_text'] == build
    assert record['build_sha256'] == hashlib.sha256(build.encode()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--require-binaries', action='store_true')
    args = parser.parse_args()
    record = json.loads((HERE/'checked-verification.json').read_text())
    common(record)
    rows = vectors.cases()
    assert json.loads((HERE/'cases.json').read_text()) == rows
    assert record['scope'] == 'full'
    assert record['cases'] == [row['name'] for row in rows]
    assert record['positive_count'] == record['control_count'] == len(rows)
    consumed = []
    for batch in record['batches']:
        selected = [row for row in rows if row['name'] in batch['cases']]
        assert batch['cases'] == [row['name'] for row in selected]
        text, names = fixtures.render(selected)
        assert batch['source_sha256'] == hashlib.sha256(text.encode()).hexdigest()
        check.validate(batch['output'], names)
        consumed += selected
    assert consumed == rows
    const = json.loads((HERE/'const-verification.json').read_text())
    common(const)
    selected = check_const.selected()
    assert const['positive_count'] == const['control_count'] == len(selected)
    assert len(const['proofs']) == len(selected)*2
    for row, pair in zip(selected, zip(const['proofs'][::2], const['proofs'][1::2])):
        for control, proof in zip((False, True), pair):
            assert proof['case'] == row['name'] and proof['control'] == control
            assert proof['source_sha256'] == hashlib.sha256(check_const.source(row, control).encode()).hexdigest()
            assert ('cannot prove requires contract' in proof['output'] and '1 == 0' in proof['output']) if control else 'compiled ' in proof['output']
    provenance = json.loads((HERE/'toolchain.json').read_text())
    assert provenance['omega_revision'] == check.PIN and provenance['omega_clean']
    artifact_hashes = provenance.get('sha256') or {item['path']: item['sha256'] for item in provenance['artifacts'].values()}
    canonical_hashes = {str(Path(path).resolve()): digest for path, digest in artifact_hashes.items()}
    assert canonical_hashes[str(Path(record['runner_path']).resolve())] == record['runner_sha256']
    assert canonical_hashes[str(Path(const['compiler_path']).resolve())] == const['compiler_sha256']
    if args.require_binaries:
        assert check.sha(Path(record['runner_path'])) == record['runner_sha256']
        assert check.sha(Path(const['compiler_path'])) == const['compiler_sha256']
    # Public source inputs, generated AML, intended arithmetic and observed
    # differences are independently retained; no hardware/Omega execution claim.
    public = json.loads((HERE/'reference-verification.json').read_text())
    assert public['pin'] == reference.PIN
    assert public['source_sha256'] == {str(path.relative_to(fixtures.ROOT)): check.sha(path) for path in reference.source_inputs()}
    assert len(public['rows']) == len(reference.cases())
    comparisons = []
    for expected, actual in zip(reference.cases(), public['rows']):
        for key, value in expected.items():
            assert actual[key] == value
        assert actual['aml_hex'] == reference.table(expected).hex()
        assert actual['observed']['forbidden_calls'] == '0'
        comparisons.append(reference.compare(expected, actual['observed']))
    expected_comparison = dict(stage='independent recipe arithmetic expanded through normal register geometry; differences retained without compatibility mode', public_count=len(comparisons), counts=dict(sorted(Counter(row['category'] for row in comparisons).items())), rows=comparisons)
    assert json.loads((HERE/'comparison.json').read_text()) == expected_comparison
    print('PASS', len(rows), 'checked pairs,', len(selected), 'constant pairs,', len(public['rows']), 'public observations; exact retained inputs')


if __name__ == '__main__':
    main()
