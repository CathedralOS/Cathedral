#!/usr/bin/env python3
"""Partial zero-length Buffer conversion Store source mapping; aggregate anchors stay open."""
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
            entry['reason'] = 'Outside the zero-length direct named Buffer conversion Store component; aggregate behavior remains pending.'
    for path, key in [('src/aml/mod.rs','2406:do_store'), ('src/aml/object.rs','317:replace_with_implicit_casting')]:
        entry = value['files'][path]['symbols'][key]
        entry['targets'] = [dict(path='source/libraries/acpi/aml/named_value_store.omg', anchor=anchor)
                            for anchor in ['pub machine store_value(', 'pub machine store_integer(', 'machine prepare_integer(']]
        entry['targets'].append(dict(path='source/libraries/acpi/aml/buffer_target_values.omg', anchor='pub machine prepare_buffer_extent('))
        entry['note'] = ('Partial zero-length Buffer target conversion: Integer scalar/object and fully admitted String sources '
                         'publish an empty owned Buffer atomically without allocation; complete source/destination identity retained. '
                         'Positive-target empty String precedence and unequal same-type Buffer extent remain separate decisions.')
    value['primary_sources'] = [
        'https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#data-type-conversion-rules',
        'https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#buffer-declare-buffer-object',
    ]
    value['scope'] = {'target_extent': 0, 'sources': ['scalar Integer', 'object Integer', 'empty/nonempty String'],
                      'positive_target_empty_string': 'excluded pending conversion-special-case versus destination-extent precedence',
                      'unequal_buffers': 'existing separately documented compatibility decision unchanged'}
    target = ROOT/'source/libraries/acpi/aml/zero-buffer-store-inventory.json'
    text = json.dumps(value, indent=2, sort_keys=True)+'\n'
    if args.check:
        assert target.read_text() == text
    else:
        target.write_text(text)
    print(api.check(value, args.checkout, repository=ROOT))


if __name__ == '__main__':
    main()
