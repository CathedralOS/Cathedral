#!/usr/bin/env python3
"""Const-evaluate current method admission, including MAX offset, with a body control."""
import argparse
import hashlib
from pathlib import Path
import subprocess
import tempfile
import fixtures

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--omega', type=Path, default=Path('/tmp/cathedral-omega-eaa7993/release/omega'))
a = p.parse_args()
compiler = a.omega.resolve()
print('Omega SHA-256:', hashlib.sha256(compiler.read_bytes()).hexdigest(), flush=True)
build = (fixtures.HERE / 'build.omg').read_text()
for relative in ['../../../../../source/libraries/acpi/interpreter/execution', '../../../../../source/libraries/acpi/interpreter', '../../../../../source/libraries/acpi/aml']:
    build = build.replace(relative, str((fixtures.HERE / relative).resolve()))
source = (fixtures.HERE / 'frame-const.omg').read_text()
for control in [False, True]:
    body = source.replace('invalid.outcome == ExecutionOutcome::Truncated', 'invalid.outcome == ExecutionOutcome::Success') if control else source
    with tempfile.TemporaryDirectory(prefix='cathedral-aml-frame-const-') as directory:
        work = Path(directory)
        (work / 'build.omg').write_text(build)
        (work / 'main.omg').write_text(body)
        result = subprocess.run([str(compiler), '--check', str(work / 'main.omg')], capture_output=True, text=True)
        output = result.stdout + result.stderr
        valid = result.returncode != 0 and 'cannot prove requires contract' in output and '1 == 0' in output if control else result.returncode == 0
        if not valid:
            raise SystemExit(('CONTROL' if control else 'POSITIVE') + ' failed:\n' + output)
        print(('CONTROL' if control else 'PASS') + ' valid method admission and MAX-offset rejection', flush=True)
