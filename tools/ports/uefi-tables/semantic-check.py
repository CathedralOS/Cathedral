#!/usr/bin/env python3
"""Execute the table Omega fixtures by semantic evaluation, with a negative control."""
import argparse
import hashlib
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]
FIXTURE = Path(__file__).resolve().parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--omega', type=Path, default=ROOT.parent / 'Omega/target/release/omega')
    args = parser.parse_args()
    compiler = args.omega.resolve()
    print('Omega binary SHA-256:', hashlib.sha256(compiler.read_bytes()).hexdigest(), flush=True)
    positive = subprocess.run([str(compiler), '--check', str(FIXTURE / 'main.omg')], cwd=ROOT)
    if positive.returncode:
        raise SystemExit(positive.returncode)
    source = (FIXTURE / 'main.omg').read_text()
    marker = 'require_success(TEST_RESULT);'
    if source.count(marker) != 1:
        raise SystemExit('negative control requires one exact success assertion')
    build = (FIXTURE / 'build.omg').read_text()
    for relative in ('../../../source/contracts/uefi/raw',):
        if relative not in build:
            raise SystemExit(f'negative control cannot locate fixture dependency {relative}')
        build = build.replace(relative, str((FIXTURE / relative).resolve()))
    with tempfile.TemporaryDirectory(prefix='cathedral-uefi-tables-negative-') as directory:
        root = Path(directory)
        (root / 'build.omg').write_text(build)
        (root / 'main.omg').write_text(source.replace(marker, 'require_success(TEST_RESULT + 1);'))
        negative = subprocess.run([str(compiler), '--check', str(root / 'main.omg')],
                                  cwd=ROOT, capture_output=True, text=True)
        output = negative.stdout + negative.stderr
        if not negative.returncode or 'cannot prove requires contract' not in output or '0 + 1 == 0' not in output:
            raise SystemExit('negative control did not reject the computed assertion as expected:\n' + output)
    print('Table constant tests passed in Omega semantic evaluation; negative assertion rejected.')
    print('Native execution, firmware behavior, and Omega foreign-layout comparison NOT RUN.')


if __name__ == '__main__':
    main()
