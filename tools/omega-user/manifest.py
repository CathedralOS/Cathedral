"""Encode a process's declared imports in the existing platform ELF record.

This is launch metadata, not a grant. The owning matrix resolves the imports
against its offer. Wire owner: source-rs/platform/contracts/imports/imports.rs.
"""
import struct


def bounded(value, maximum, name, minimum=0):
    if type(value) is not int or not minimum <= value <= maximum:
        raise ValueError(f'{name} must be an integer in {minimum}..{maximum}')
    return value


def encode(value):
    if not isinstance(value, dict) or set(value) != {'imports', 'channels', 'descendants'}:
        raise ValueError('process manifest requires imports, channels and descendants')
    imports = value['imports']
    if not isinstance(imports, list) or len(imports) > 4:
        raise ValueError('process manifest supports at most four imports')
    channels = bounded(value['channels'], 65535, 'channels')
    descendants = bounded(value['descendants'], 65535, 'descendants')
    words = [int.from_bytes(b'CTHIMP03', 'little'), len(imports) | channels << 16 | descendants << 32]
    seen = set()
    for entry in imports:
        if not isinstance(entry, dict) or set(entry) != {'contract', 'minimum', 'maximum', 'required'}:
            raise ValueError('an import requires contract, minimum, maximum and required')
        contract = bounded(entry['contract'], 2**64 - 1, 'contract', 1)
        minimum = bounded(entry['minimum'], 2**64 - 1, 'minimum', 1)
        maximum = bounded(entry['maximum'], 2**64 - 1, 'maximum', minimum)
        if type(entry['required']) is not bool or contract in seen:
            raise ValueError('import required must be boolean and contracts must be unique')
        seen.add(contract)
        words.extend([contract, minimum, maximum, int(entry['required'])])
    words.extend([0] * (18 - len(words)))
    return struct.pack('<18Q', *words)
