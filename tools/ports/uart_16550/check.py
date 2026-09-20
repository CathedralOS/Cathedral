#!/usr/bin/env python3
"""Execute the UART Omega fixtures by semantic evaluation, with a negative control."""
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
    marker = 'calc_baud_rate(1843200, 1, false, 0).value == 115200'
    if source.count(marker) != 1:
        raise SystemExit('negative control requires one exact success assertion')
    build = (FIXTURE / 'build.omg').read_text()
    for relative in ('../../../source/drivers/uart_16550', '../../../source/drivers/facts'):
        if relative not in build:
            raise SystemExit(f'negative control cannot locate fixture dependency {relative}')
        build = build.replace(relative, str((FIXTURE / relative).resolve()))
    with tempfile.TemporaryDirectory(prefix='cathedral-uart-negative-') as directory:
        root = Path(directory)
        (root / 'build.omg').write_text(build)
        (root / 'main.omg').write_text(source.replace(marker, 'calc_baud_rate(1843200, 1, false, 0).value == 115201'))
        negative = subprocess.run([str(compiler), '--check', str(root / 'main.omg')],
                                  cwd=ROOT, capture_output=True, text=True)
        output = negative.stdout + negative.stderr
        if not negative.returncode or 'cannot prove requires contract' not in output or '1 == 0' not in output:
            raise SystemExit('negative control did not reject the computed assertion as expected:\n' + output)
    print('UART tests passed in Omega semantic evaluation; mutated baud expectation evaluated false and was rejected.')
    print('Native execution and UART hardware access NOT RUN.')


if __name__ == '__main__':
    main()
