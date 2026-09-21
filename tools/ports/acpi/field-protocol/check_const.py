#!/usr/bin/env python3
"""Representative actual constant evaluation; distinct from checked execution."""
import argparse
import hashlib
import json
import subprocess
import tempfile
import time
from pathlib import Path

import check
import fixtures
import vectors

HERE = fixtures.HERE


def selected():
    names = ['bank_read_0_0', 'index_write_3_0', 'index_last_selector_overflow']
    rows = {row['name']: row for row in vectors.cases()}
    return [rows[name] for name in names]


def source(row, control):
    return fixtures.HEAD + 'machine test_result()->i32 {\n' + fixtures.body(row, control) + '}\nconst TEST_RESULT:i32=test_result();\nmachine require_ok(value:i32) requires value==0;{}\ndata Main{}\nmachine Main::main(&mut self){require_ok(TEST_RESULT);}\n'


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--compiler', type=Path, default=Path('/tmp/cathedral-field-protocol-target/release/omega'))
    parser.add_argument('--record', type=Path, default=HERE/'const-verification.json')
    args = parser.parse_args()
    inputs, binary = check.snapshot(), check.sha(args.compiler)
    proofs, start = [], time.monotonic()
    build = check.build_text()
    for row in selected():
        for control in (False, True):
            text = source(row, control)
            with tempfile.TemporaryDirectory(prefix='cathedral-field-protocol-const-') as directory:
                work = Path(directory)
                (work/'build.omg').write_text(build)
                (work/'main.omg').write_text(text)
                run = subprocess.run([str(args.compiler), '--check', str(work/'main.omg')], capture_output=True, text=True)
            output = run.stdout + run.stderr
            if control:
                assert run.returncode != 0 and 'cannot prove requires contract' in output and '1 == 0' in output, output
            else:
                assert run.returncode == 0, output
            proofs.append(dict(case=row['name'], control=control, source_sha256=hashlib.sha256(text.encode()).hexdigest(), output=output))
            assert inputs == check.snapshot() and binary == check.sha(args.compiler), 'inputs changed'
            print('PASS constant', row['name'], 'control', control, flush=True)
    args.record.write_text(json.dumps(dict(stage='constant evaluation and requires proof; no native/hardware', omega_revision=check.PIN, execution_root=str(fixtures.ROOT), build_text=build, build_sha256=hashlib.sha256(build.encode()).hexdigest(), input_sha256=inputs, compiler_path=str(args.compiler.resolve()), compiler_sha256=binary, positive_count=len(selected()), control_count=len(selected()), elapsed_seconds=round(time.monotonic()-start, 3), proofs=proofs), indent=2, sort_keys=True)+'\n')


if __name__ == '__main__':
    main()
