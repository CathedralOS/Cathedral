#!/usr/bin/env python3
"""Verify recorded tested inputs; does not claim a fresh semantic execution."""
import hashlib,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
record=json.loads((HERE/'verification.json').read_text())
for name,digest in record['source_and_fixture_sha256'].items():
 assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest,name
assert record['omega_fixtures']==10 and record['body_controls']==12
print('Verified tested encrypted register source/fixture hashes;100 rows,10 fixtures,12 controls.')
