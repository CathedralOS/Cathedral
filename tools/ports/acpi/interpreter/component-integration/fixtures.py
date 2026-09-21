"""Focused interaction witnesses for installed Fields, ObjectType and ToInteger."""
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
GENERIC = HERE.parent / 'generic-execution'
sys.path.insert(0, str(GENERIC))
spec = importlib.util.spec_from_file_location('generic_component_fixtures', GENERIC / 'fixtures.py')
base = importlib.util.module_from_spec(spec)
spec.loader.exec_module(base)
integer, name, pkg, op, ret = base.integer, base.name, base.pkg, base.op, base.ret


def cases():
    rows = []
    string = lambda value: b'\x0d' + value + b'\0'
    named = lambda key, value: b'\x08' + name(key) + value
    method = lambda key, body, arguments=0: pkg(0x14, name(key) + bytes([arguments]) + body)
    # One independently encoded 16-byte region and one eight-bit normal Field.
    field = b'\x5b\x80REG0\x00\x00\x0a\x10\x5b\x81\x0bREG0\x01FLD0\x08'

    def add(label, table, bits, number, count, after=None, error='Success', setup=None):
        rows.append(dict(name=label, table=table.hex(), kind=1, number=number,
                         bytes='', bits=bits, after=after or {}, object_count=count,
                         error=error, note='', setup=setup or [], mutate_source=False))

    for bits in (32, 64):
        for label, target in [('string', string(b'old')), ('buffer', pkg(0x11, b'\0'))]:
            body = op(0x99, name('SRC'), name('DST')) + ret(op(0x8e, name('DST')))
            table = named('SRC', string(b'42')) + named('DST', target) + method('MAIN', body)
            add(f'converted_{label}_type_{bits}', table, bits, 1, 3, {'DST': 42})
        add(f'installed_field_type_{bits}', field + method('MAIN', ret(op(0x8e, name('FLD0')))), bits, 5, 3)
        table = named('SRC', string(b'42')) + field + method('MAIN', ret(op(0x99, name('SRC'), name('FLD0'))))
        add(f'field_conversion_target_{bits}', table, bits, 42, 4, error='UnresolvedRegion')
        body = op(0x9d, name('SRC'), b'\x60') + ret(op(0x99, b'\x60', b'\0'))
        table = named('SRC', string(b'42')) + named('DST', integer(7)) + method('MAIN', body)
        add(f'owned_local_source_{bits}', table, bits, 42, 4, {'DST': 7})
        for kind in ('RefOf', 'Index'):
            table = (named('SRC', string(b'42')) + named('DST', string(b'old')) + named('REF0', integer(0))
                     + method('CAL', ret(op(0x99, name('SRC'), b'\x68')), 1)
                     + method('MAIN', ret(name('CAL') + name('REF0'))))
            add(f'argument_{kind.lower()}_target_{bits}', table, bits, 42, 5, {'DST': 42},
                setup=[f'program.store.space.objects[2].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:1}};'])
    return rows


def render(match=''):
    rows = [row for row in cases() if not match or row['name'] in match.split(',')]
    assert rows
    source, selections = base.render(rows)
    return rows, source, selections


if __name__ == '__main__':
    rows, source, selections = render()
    (HERE / 'cases.json').write_text(json.dumps(rows, indent=2) + '\n')
    print(len(rows), 'component interaction behavior/control pairs')
