#!/usr/bin/env python3
"""Verify complete pipeline/parser checked-execution evidence and current hashes."""
import hashlib
import json
from pathlib import Path
import fixtures
import check

for filename, expected, kind in [
    ('verification.json', [row['name'] for row in fixtures.cases()], 'cathedral-acpi-pipeline-checked-v1'),
    ('parser-verification.json', [row['name'] for row in check.parser_suite()[0]], 'cathedral-acpi-parser-regression-checked-v1'),
]:
    record = json.loads((fixtures.HERE / filename).read_text())
    assert record['format'] == kind
    assert record['omega_revision'] == 'eaa7993a23623cd8fabf45350340479c5c9c7879'
    assert record['cases'] == expected
    assert record['scenario_count'] == record['control_count'] == len(expected)
    assert record['evaluator_step_limit'] == 10_000_000
    assert record['cargo_lock_sha256'] == hashlib.sha256((check.SHARED / 'runner.Cargo.lock').read_bytes()).hexdigest()
    for relative, digest in record['source_sha256'].items():
        source = (fixtures.ROOT / relative).resolve()
        assert source.is_relative_to(fixtures.ROOT)
        assert hashlib.sha256(source.read_bytes()).hexdigest() == digest, 'Changed source: ' + relative
    for name in expected:
        for suffix, value in [('positive', 0), ('control', 1)]:
            marker = f'PASS PipelineSuite::{name}_{suffix} expected={value} observed={value} error=None '
            assert record['output'].count(marker) == 1, 'Missing observation: ' + marker
    assert '\nFAIL ' not in record['output']
    print(f'{filename}: current source hashes and {len(expected)} checked-interpreter pairs verified.')
print('Native/hardware not run.')
