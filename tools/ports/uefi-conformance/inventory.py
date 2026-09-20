#!/usr/bin/env python3
"""Union reviewed slice inventories; require every pinned raw-crate source file."""
import argparse
import copy
import collections
import hashlib
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
# This script shares a filename with the common module, so load by explicit path.
spec = importlib.util.spec_from_file_location('port_inventory', ROOT / 'tools/ports/inventory.py')
common = importlib.util.module_from_spec(spec)
spec.loader.exec_module(common)
RAW = ROOT / 'source/contracts/uefi/raw'
UPSTREAM = ROOT / 'reference_code/rust-osdev/uefi-rs'
PIN = 'c0facddf9ba42b74906a37fca2869e6cdbc8da6a'
SLICES = ['scalar', 'tables', 'console', 'storage', 'machine', 'network', 'hii', 'tcg', 'shell']
SCAFFOLD = ['uefi-raw/src/enums.rs', 'uefi-raw/src/protocol/mod.rs', 'uefi-raw/src/table/mod.rs']
RANK = {'pending': 0, 'omitted': 1, 'blocked': 2, 'translated': 3}


def merge(left, right):
    if left['anchor'] != right['anchor']:
        raise ValueError('slice source anchors disagree')
    if RANK[right['disposition']] > RANK[left['disposition']]:
        return copy.deepcopy(right)
    if left['disposition'] == right['disposition'] == 'translated':
        for target in right['targets']:
            if target not in left['targets']:
                left['targets'].append(copy.deepcopy(target))
    return left


def collect():
    result = common.snapshot(UPSTREAM, PIN, ['uefi-raw/src'], 'https://github.com/rust-osdev/uefi-rs')
    mapped = {}
    for name in SLICES:
        path = RAW / (name + '-inventory.json')
        if not path.is_file():
            raise ValueError('required reviewed slice absent: ' + str(path))
        part = common.read_json(path)
        common.check(part, UPSTREAM)
        for source, entry in part['files'].items():
            if source not in result['files']:
                continue  # separately audited safe-crate additions are outside this raw-crate claim
            if source not in mapped:
                mapped[source] = copy.deepcopy(entry)
            else:
                for key, row in entry['symbols'].items():
                    mapped[source]['symbols'][key] = merge(mapped[source]['symbols'][key], row)
                if RANK[entry['disposition']] > RANK[mapped[source]['disposition']]:
                    for key in ['disposition', 'targets', 'reason']:
                        if key in entry:
                            mapped[source][key] = copy.deepcopy(entry[key])
                        else:
                            mapped[source].pop(key, None)
    for source in SCAFFOLD:
        entry = copy.deepcopy(result['files'][source])
        reason = ('Rust newtype_enum macro and formatting machinery deliberately omitted; every invocation is expanded into open raw carriers and named constants in its owning slice.'
                  if source.endswith('enums.rs') else 'Rust module/reexport topology deliberately flattened into the reviewed Omega slice modules; all referenced source files are independently covered by this complete inventory.')
        entry.update(disposition='omitted', reason=reason)
        for row in entry['symbols'].values():
            row.update(disposition='omitted', reason=reason)
        mapped[source] = entry
    if set(mapped) != set(result['files']):
        raise ValueError('unreviewed raw source files: ' + repr(sorted(set(result['files']) - set(mapped))))
    result['files'] = {source: mapped[source] for source in sorted(mapped)}
    if any(row['disposition'] == 'pending' for entry in result['files'].values() for row in entry['symbols'].values()):
        raise ValueError('pending source anchors remain')
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write', action='store_true', help='write reviewed union after validating every source slice')
    args = parser.parse_args()
    expected = collect()
    parts = {name: common.read_json(RAW / (name + '-inventory.json')) for name in SLICES}
    index = {'format': 'cathedral-uefi-conformance-index-v1', 'upstream': expected['upstream'],
             'slices': {name: {'path': name + '-inventory.json', 'sha256': hashlib.sha256((RAW / (name + '-inventory.json')).read_bytes()).hexdigest()} for name in SLICES},
             'files': {}}
    for source, entry in expected['files'].items():
        row = {'sha256': entry['sha256'], 'review_slices': [name for name in SLICES if source in parts[name]['files']],
               'symbols': dict(sorted(collections.Counter(symbol['disposition'] for symbol in entry['symbols'].values()).items()))}
        if source in SCAFFOLD:
            row['scaffold_omission'] = entry['reason']
        index['files'][source] = row
    path = RAW / 'conformance-index.json'
    if args.write:
        path.write_text(json.dumps(index, indent=2) + '\n')
    elif common.read_json(path) != index:
        raise ValueError('whole-crate index differs from reviewed source slices; review and regenerate')
    print(json.dumps(common.check(expected, UPSTREAM), indent=2))


if __name__ == '__main__':
    main()
