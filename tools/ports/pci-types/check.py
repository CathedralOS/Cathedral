#!/usr/bin/env python3
"""Audit the exact pci_types pin and execute upstream witnesses without hardware."""
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
import inventory


def main():
    package = ROOT/'source/libraries/pci'
    checkout = ROOT/'reference_code/rust-osdev/pci_types'
    manifest = inventory.read_json(package/'inventory.json')
    result = inventory.check(manifest, checkout, ROOT, require_transcribed=True)
    print(json.dumps(result, indent=2), flush=True)
    # The lexical scanner does not index bare enum variants. Audit every named
    # DeviceType value separately so the long taxonomy cannot silently shrink.
    source = (checkout/'src/device_type.rs').read_text()
    body = source.split('pub enum DeviceType {', 1)[1].split('\n}', 1)[0]
    names = set(re.findall(r'^    ([A-Za-z][A-Za-z0-9]*),$', body, re.M))
    translated = set(re.findall(r'pub const (\w+): DeviceType', (package/'device_type.omg').read_text()))
    expected = (names - {'Unknown'}) | {'DEVICE_UNKNOWN'}
    if translated != expected:
        raise SystemExit(f'DeviceType taxonomy mismatch: {translated ^ expected}')
    print(f'All {len(names)} upstream DeviceType names retained.', flush=True)
    subprocess.run(['cargo', 'run', '--locked', '--manifest-path', str(HERE/'Cargo.toml')], cwd=ROOT, check=True)
    print('Inventory/taxonomy and actual pinned Rust witnesses passed. Run semantic-check.py for Omega behavior.')

if __name__ == '__main__':
    main()
