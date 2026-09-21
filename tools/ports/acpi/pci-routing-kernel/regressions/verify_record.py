#!/usr/bin/env python3
"""Verify fresh query/parser runtime receipts after shared unsigned-guard fixes."""
import hashlib
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
sys.path.insert(0, str(ROOT / 'tools/ports/acpi/aml/object-queries'))
import check as query_check
import fixtures as query_fixtures

RUNNER = 'e6d0aee6b4dddbbf34a60cffbe8f158643cc5c4d100f9e20f0481f75f52890be'

def hashes(values):
    for path, digest in values.items():
        assert query_check.sha(ROOT / path) == digest, path

def pairs(output, names, prefix):
    assert '\nFAIL ' not in output
    for name in names:
        for control in [False, True]:
            suffix = '_control' if control else '_positive'
            expected = int(control)
            text = f'PASS {prefix}::{name}{suffix} expected={expected} observed={expected} error=None '
            assert output.count(text) == 1, text

def main():
    query = json.loads((HERE / 'object-queries.json').read_text())
    assert query['input_sha256'] == query_check.snapshot()
    assert query['omega_revision'] == query_check.PIN and query['runner_sha256'] == RUNNER
    rows = {row['name']: row for row in query_fixtures.cases()}
    assert query['cases'] == list(rows)
    assert query['positive_count'] == query['control_count'] == len(rows) == 96
    selected = []
    for batch in query['batches']:
        source, _ = query_check.source([rows[name] for name in batch['cases']])
        assert batch['source_sha256'] == hashlib.sha256(source.encode()).hexdigest()
        assert batch['runner_sha256'] == RUNNER
        pairs(batch['output'], batch['cases'], 'Suite')
        selected.extend(batch['cases'])
    assert selected == query['cases']
    parser = json.loads((HERE / 'parser.json').read_text())
    hashes(parser['source_sha256'])
    assert parser['omega_revision'] == query_check.PIN and parser['runner_sha256'] == RUNNER
    assert parser['scenario_count'] == parser['control_count'] == len(parser['cases']) == 27
    assert len(set(parser['cases'])) == 27
    pairs(parser['output'], parser['cases'], 'PipelineSuite')
    print('PASS 96 query pairs and 27 parser pairs after shared guard fixes; current source hashes')

if __name__ == '__main__':
    main()
