#!/usr/bin/env python3
"""Partial equal-extent Buffer Store source mapping; aggregate anchors stay open."""
import argparse
import importlib.util
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
spec = importlib.util.spec_from_file_location('inventory_api', ROOT/'tools/ports/inventory.py')
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--checkout', type=Path, default=ROOT/'reference_code/rust-osdev/acpi')
    args = parser.parse_args()
    value = api.snapshot(args.checkout, '257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5',
                         ['src/aml/mod.rs', 'src/aml/object.rs'], 'https://github.com/rust-osdev/acpi')
    for item in value['files'].values():
        for entry in item['symbols'].values():
            entry['reason'] = 'Outside the equal-extent direct named Buffer Store component; aggregate behavior remains pending.'
    for path, key in [('src/aml/mod.rs','2406:do_store'), ('src/aml/object.rs','317:replace_with_implicit_casting')]:
        entry = value['files'][path]['symbols'][key]
        entry['targets'] = [dict(path='source/libraries/acpi/aml/named_value_store.omg', anchor=anchor)
                            for anchor in ['pub machine store_value(', 'state buffer_copy(', 'state same_extent(']]
        entry['note'] = ('Partial Buffer-source branch only: both direct objects fully admitted; equal logical extents 0..256 copied atomically '
                         'through existing conversion and publisher. Unequal extents are a separate compatibility decision: '
                         'the Rust pin resizes while ACPICA retains nonzero dynamic extents. No target/frame/field execution claims.')
    value['primary_sources'] = [
        'https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#named-objects',
        'https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#buffer-declare-buffer-object',
    ]
    value['scope'] = {'equal_extents': [0,256], 'source_identity': 'retained', 'destination_identity': 'retained',
                      'ordinary_conversions': 'existing Integer/String target-extent rules unchanged',
                      'unequal_extents': 'explicitly unsupported pending same-type extent compatibility decision'}
    target = ROOT/'source/libraries/acpi/aml/named-buffer-store-inventory.json'
    text = json.dumps(value, indent=2, sort_keys=True)+'\n'
    if args.check:
        assert target.read_text() == text
    else:
        target.write_text(text)
    print(api.check(value, args.checkout, repository=ROOT))


if __name__ == '__main__':
    main()
