#!/usr/bin/env python3
"""Format Cargo modules and the named bodies included by Rust module plumbing."""

import argparse
from pathlib import Path
import re
import subprocess

WORKSPACE = Path(__file__).resolve().parents[2] / 'source-rs'
LAYERS = ('kernel', 'platform', 'distribution', 'contracts', 'foundation')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', help='report changes without writing')
    args = parser.parse_args()
    cargo = ['cargo', 'fmt', '--all']
    if args.check:
        cargo += ['--', '--check']
    result = subprocess.run(cargo, cwd=WORKSPACE).returncode

    # cargo fmt follows module declarations, but does not open include! bodies.
    # Resolve these from their wiring files, so newly added entrances are covered.
    bodies = set()
    for layer in LAYERS:
        for module in (WORKSPACE / layer).rglob('mod.rs'):
            for relative in re.findall(
                r'^include!\("([^"]+\.rs)"\);$', module.read_text(encoding='utf-8'), re.MULTILINE
            ):
                bodies.add((module.parent / relative).relative_to(WORKSPACE).as_posix())
    files = sorted(bodies)
    # Bound command size for Windows; use the workspace's pinned Rust toolchain.
    for offset in range(0, len(files), 32):
        command = ['rustfmt', '--edition', '2024']
        if args.check:
            command.append('--check')
        code = subprocess.run(command + files[offset:offset + 32], cwd=WORKSPACE).returncode
        result = result or code
    return result


if __name__ == '__main__':
    raise SystemExit(main())
