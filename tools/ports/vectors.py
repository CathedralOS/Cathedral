#!/usr/bin/env python3
"""Validate port layout/value vectors or compare independently observed Omega output."""
import argparse
import json
import re
from pathlib import Path
from inventory import read_json

KINDS = {'size', 'alignment', 'offset', 'value', 'bytes'}


def validate(document):
    if document['format'] != 'cathedral-port-vectors-v1':
        raise ValueError('unsupported vector format')
    target = document['target']
    if target['pointer_bits'] not in (32, 64) or target['endian'] not in ('little', 'big') or not target['abi']:
        raise ValueError('target must specify ABI, pointer_bits (32/64), and endian')
    provenance = document['provenance']
    if not provenance['description'].strip() or not provenance['sources']:
        raise ValueError('vectors need a provenance description and sources')
    if provenance['kind'] not in {'upstream', 'specification', 'omega-inspection', 'test-fixture'}:
        raise ValueError('unsupported provenance kind')
    if not document['measurements']:
        raise ValueError('empty vector set')
    for name, row in document['measurements'].items():
        if not name or row['kind'] not in KINDS:
            raise ValueError(f'{name}: invalid measurement kind')
        value = row['value']
        if row['kind'] == 'bytes':
            if not isinstance(value, str) or not re.fullmatch(r'(?:[0-9a-f]{2})*', value):
                raise ValueError(f'{name}: bytes must be lowercase hex pairs')
        elif type(value) is not int:
            raise ValueError(f'{name}: expected an integer, not a float or boolean')
        elif row['kind'] != 'value' and value < 0:
            raise ValueError(f'{name}: negative geometry')
        if row['kind'] == 'alignment' and (value <= 0 or value & (value - 1)):
            raise ValueError(f'{name}: alignment must be a positive power of two')
    return document


def compare(expected, observed):
    validate(expected)
    validate(observed)
    if observed['provenance']['kind'] != 'omega-inspection':
        raise ValueError('comparison requires an independently produced Omega observation')
    for field in ('compiler_revision', 'artifact_sha256'):
        width = 40 if field == 'compiler_revision' else 64
        if not re.fullmatch(f'[0-9a-f]{{{width}}}', observed['provenance'].get(field, '')):
            raise ValueError(f'observation requires {field}')
    if expected['target'] != observed['target']:
        raise ValueError('target/ABI mismatch')
    missing = set(expected['measurements']) - set(observed['measurements'])
    extra = set(observed['measurements']) - set(expected['measurements'])
    if missing or extra:
        raise ValueError(f'vector coverage differs: missing={sorted(missing)}, extra={sorted(extra)}')
    mismatches = [name for name, row in expected['measurements'].items()
                  if (row['kind'], row['value']) != (observed['measurements'][name]['kind'], observed['measurements'][name]['value'])]
    if mismatches:
        raise ValueError(f'vector values differ: {sorted(mismatches)}')
    return len(expected['measurements'])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('expected', type=Path)
    parser.add_argument('--observed', type=Path)
    arguments = parser.parse_args()
    try:
        expected = validate(read_json(arguments.expected))
        if arguments.observed:
            count = compare(expected, read_json(arguments.observed))
            print(f'{count} measurements match supplied Omega observation; provenance must be reviewed independently')
        else:
            print(f'{len(expected["measurements"])} expected vectors valid; Omega comparison NOT RUN')
    except (KeyError, TypeError, ValueError, OSError) as error:
        parser.exit(1, f'error: {error}\n')


if __name__ == '__main__':
    main()
