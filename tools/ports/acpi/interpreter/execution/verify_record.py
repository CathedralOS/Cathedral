#!/usr/bin/env python3
"""Check retained whole-suite evidence against current source and fixture bytes."""
import hashlib
import json
from pathlib import Path
import fixtures

here = Path(__file__).resolve().parent
record = json.loads((here / 'verification.json').read_text())
expected = [row['name'] for row in fixtures.cases()]
assert record['format'] == 'cathedral-aml-execution-checked-v1'
assert record['omega_revision'] == 'eaa7993a23623cd8fabf45350340479c5c9c7879'
assert record['cases'] == expected, 'Record is not the complete current scenario set'
assert record['scenario_count'] == record['control_count'] == len(expected)
assert record['evaluator_step_limit'] == 10_000_000
assert record['cargo_lock_sha256'] == hashlib.sha256((here / 'runner.Cargo.lock').read_bytes()).hexdigest()
for relative, expected_hash in record['source_sha256'].items():
    path = (fixtures.ROOT / relative).resolve()
    assert path.is_relative_to(fixtures.ROOT), 'Evidence path leaves repository'
    assert hashlib.sha256(path.read_bytes()).hexdigest() == expected_hash, 'Changed evidence source: ' + relative
for name in expected:
    for suffix, value in [('positive', 0), ('control', 1)]:
        marker = f'PASS Suite::{name}_{suffix} expected={value} observed={value} error=None '
        assert record['output'].count(marker) == 1, 'Missing or duplicated observation: ' + marker
assert '\nFAIL ' not in record['output']
print(f'Current source hashes and {len(expected)} checked-interpreter pairs verified; native/hardware not run.')
