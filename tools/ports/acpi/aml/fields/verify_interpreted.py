#!/usr/bin/env python3
"""Check retained current field-regression hashes and original body controls."""
import hashlib
import json
from pathlib import Path
import check_interpreted as runner

record=json.loads((runner.HERE/'checked-verification.json').read_text())
expected=list(json.loads((runner.HERE/'cases.json').read_text()))
assert record['format']=='cathedral-acpi-fields-checked-v1'
assert record['runner_sha256']==runner.RUNNER_SHA
assert record['omega_revision']=='eaa7993a23623cd8fabf45350340479c5c9c7879'
assert record['cases']==expected
assert record['scenario_count']==record['control_count']==len(expected)
assert record['evaluator_step_limit']==10000000
assert record['source_sha256']==runner.snapshot(),'Changed field-regression source or fixture'
for name in expected:
    for suffix,value in [('positive',0),('control',1)]:
        marker=f"PASS FieldSuite::{name.replace('-','_')}_{suffix} expected={value} observed={value} error=None "
        assert record['output'].count(marker)==1,marker
assert '\nFAIL 'not in record['output']
print(f'Current field closure and {len(expected)} unchanged assertion/mutation pairs verified; native/hardware not run.')
