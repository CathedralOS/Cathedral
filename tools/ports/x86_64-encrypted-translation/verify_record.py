#!/usr/bin/env python3
"""Verify recorded tested inputs; does not claim a fresh semantic execution."""
import hashlib,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
record=json.loads((HERE/'verification.json').read_text())
for name,digest in record['source_and_fixture_sha256'].items():
 assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest,name
assert record['omega_fixtures']==55 and record['body_controls']==9
print('Verified tested encrypted translation source/fixture hashes;330 observations,55 fixtures,9 controls.')
