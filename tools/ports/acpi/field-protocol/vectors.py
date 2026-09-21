"""Original protocol cases and independent mathematical recipe expectations.

No imported production planner, geometry body, or pinned private implementation.
All arithmetic uses unbounded Python integers before applying explicit limits.
"""
from copy import deepcopy

MAX = (1 << 64) - 1


def field(offset=3, length=11, flags=2, kind='Field', patch=''):
    return dict(offset=offset, length=length, flags=flags, kind=kind, patch=patch)


def geometry(item, region, size):
    patch = item.get('patch', '')
    if item.get('kind', 'Field') != 'Field':
        return dict(error='UnsupportedKind')
    if patch == 'connection':
        return dict(error='UnsupportedConnection')
    if patch in ['attribute', 'attribute_mode', 'extended', 'access_length']:
        return dict(error='UnsupportedMetadata')
    flags = item['flags']; code = flags & 15; update = (flags >> 5) & 3
    if flags >= 128 or code > 5 or update == 3:
        return dict(error='InvalidFlags')
    if patch in ['kind_mismatch', 'update_mismatch', 'lock_mismatch']:
        return dict(error='InconsistentAccess')
    assert patch == '', patch
    if code == 5:
        return dict(error='UnsupportedBuffer')
    offset = item['offset']; length = item['length']
    if not length:
        return dict(error='UnsupportedZeroWidth')
    if length > 2048:
        return dict(error='Capacity')
    if offset + length > MAX:
        return dict(error='Overflow')
    width = [1, 1, 2, 4, 8][code]; bits = width * 8
    first = (offset // bits) * width
    end = ((offset + length + bits - 1) // bits) * width
    if end > region:
        return dict(error='OutsideRegion')
    chunks = []
    for byte in range(first, end, width):
        left = max(offset, byte * 8); right = min(offset + length, (byte + width) * 8)
        native = left - byte * 8; count = right - left
        chunks.append(dict(offset=byte, width=width, field_bit=left-offset,
                           native_bit=native, bit_count=count,
                           mask=((1 << count) - 1) << native, partial=count != bits))
    return dict(start=first, end=end, width=width, count=len(chunks),
                preserve_reads=sum(c['partial'] for c in chunks) if update == 0 else 0,
                locked=bool(flags & 16), shape='Integer' if length <= size * 8 else 'Buffer',
                bytes=(length + 7) // 8, chunks=chunks)


def protocol(row):
    if row['kind'] not in ['Bank', 'Index']:
        return dict(error=dict(component='Field', geometry='UnsupportedKind'))
    plans = {}
    for component, key, region in [('Field', 'field', row['region'] if row['kind'] == 'Bank' else MAX),
                                   ('Selector', 'selector', row['selector_region']),
                                   ('Data', 'data', row['data_region'])]:
        if component == 'Data' and row['kind'] == 'Bank':
            continue
        plan = geometry(row[key], region, row['size'])
        if 'error' in plan:
            return dict(error=dict(component=component, geometry=plan['error']))
        plans[key] = plan
    value = (row['bank_value'] & ((1 << (row['size'] * 8)) - 1)) if row['kind'] == 'Bank' else 0
    outer = plans['field']; selector_length = row['selector']['length']
    values = [value] if row['kind'] == 'Bank' else [c['offset'] for c in outer['chunks']]
    if selector_length < 64 and any(v >= 1 << selector_length for v in values):
        return dict(protocol='SelectorOverflow')
    actions = []
    for index, chunk in enumerate(outer['chunks']):
        selector = value if row['kind'] == 'Bank' else chunk['offset']
        if row['direction'] == 'Read' or (chunk['partial'] and ((row['field']['flags'] >> 5) & 3) == 0):
            actions += [dict(kind='Select', value=selector), dict(kind='Read', index=index)]
        if row['direction'] == 'Write':
            actions += [dict(kind='Select', value=selector), dict(kind='Write', index=index)]
    return dict(outer=outer, selector=plans['selector'], data=plans.get('data'),
                bank_value=value, lock=any(p['locked'] for p in plans.values()), actions=actions)


def case(name, kind='Index', direction='Read', size=8, **overrides):
    row = dict(name=name, kind=kind, direction=direction, size=size,
               field=field(), region=512, selector=field(256, 16, 2), selector_region=512,
               data=field(512, 16, 2), data_region=512, bank_value=5)
    row.update(overrides)
    row['expected'] = protocol(row)
    return row


def cases():
    rows = []
    for kind in ['Bank', 'Index']:
        for direction in ['Read', 'Write']:
            for code, width in enumerate([1, 1, 2, 4, 8]):
                for update in range(3):
                    rows.append(case(f'{kind.lower()}_{direction.lower()}_{code}_{update}', kind, direction,
                                     field=field(width*8+3, width*16+1, code | (update << 5)),
                                     data=field(515, width*8, 1)))
    for direction in ['Read', 'Write']:
        for length in [1, 3, 8, 16, 64, 65, 2048]:
            rows.append(case(f'data_width_{direction.lower()}_{length}', direction=direction,
                             field=field(35, 42, 3), data=field(519, length, 2)))
        for length in [1, 3, 8, 64, 65, 2048]:
            rows.append(case(f'selector_width_{direction.lower()}_{length}', direction=direction,
                             selector=field(263, length, 2)))
        for part in ['field', 'selector', 'data']:
            row = case(f'lock_{direction.lower()}_{part}', direction=direction)
            row[part]['flags'] |= 16; row['expected'] = protocol(row); rows.append(row)
        rows.append(case(f'all_locks_{direction.lower()}', direction=direction,
                         field=field(3, 19, 18), selector=field(263, 17, 18), data=field(515, 33, 18)))
        for kind in ['Bank', 'Index']:
            rows.append(case(f'max_chunks_{kind.lower()}_{direction.lower()}', kind, direction,
                             field=field(1, 2048, 1), selector=field(256, 16, 1), data=field(512, 8, 1)))
            rows.append(case(f'full_write_{kind.lower()}_{direction.lower()}', kind, direction,
                             field=field(64, 64, 4)))
    for size in [4, 8]:
        for bank in [0, (1 << 32) - 1, 1 << 32, MAX]:
            rows.append(case(f'bank_integer_{size}_{bank}', 'Bank', size=size,
                             bank_value=bank, selector=field(256, 64, 4)))
        for length in [32, 33, 64, 65]:
            rows.append(case(f'shape_{size}_{length}', size=size, field=field(1, length, 4)))
    for kind in ['Bank', 'Index']:
        for fits in [True, False]:
            rows.append(case(f'selector_fit_{kind.lower()}_{fits}', kind, bank_value=7 if fits else 8,
                             field=field(56 if fits else 64, 8, 1), selector=field(256, 3, 1)))
    rows.append(case('index_last_selector_overflow', field=field(56, 16, 1), selector=field(256, 3, 1)))
    rows.append(case('index_last_selector_exact', field=field(48, 16, 1), selector=field(256, 3, 1)))
    rows.append(case('index_virtual_extent', field=field(4096, 8, 1), region=0))
    for offset, length in [(1 << 63, 64), (MAX-7, 7), (MAX, 1), (MAX-6, 7)]:
        rows.append(case(f'index_offset_{offset}_{length}', field=field(offset, length, 1), selector=field(256, 64, 4)))
    for component in ['field', 'selector', 'data']:
        variants = [('zero', field(length=0)), ('capacity', field(length=2049)),
                    ('length_high', field(length=1 << 63)), ('length_max', field(length=MAX)),
                    ('interval', field(offset=MAX, length=1)), ('kind_bank', field(kind='Bank')),
                    ('kind_index', field(kind='Index')), ('flags_access', field(flags=6)),
                    ('flags_reserved', field(flags=129)), ('flags_update', field(flags=97)),
                    ('buffer', field(flags=5))]
        variants += [(patch, field(patch=patch)) for patch in
                     ['connection', 'attribute', 'attribute_mode', 'extended', 'access_length',
                      'kind_mismatch', 'update_mismatch', 'lock_mismatch']]
        for suffix, item in variants:
            if component == 'field' and suffix in ['kind_bank', 'kind_index']:
                continue
            rows.append(case(f'invalid_{component}_{suffix}', **{component: item}))
    for component in ['selector', 'data']:
        rows.append(case(f'outside_{component}', **{component: field(16, 1, 2), component+'_region': 3}))
    rows.append(case('outside_bank_field', 'Bank', field=field(16, 1, 2), region=3))
    rows.append(case('bank_ignores_data', 'Bank', data=field(length=0, kind='Bank', patch='connection'), data_region=0))
    rows.append(case('precedence_outer_selector', field=field(flags=6), selector=field(length=0)))
    rows.append(case('precedence_selector_data', selector=field(length=0), data=field(flags=6)))
    rows.append(case('precedence_data_fit', field=field(64, 8, 1), selector=field(256, 3, 1), data=field(length=0)))
    rows.append(case('precedence_metadata_flags', field=field(flags=255, patch='attribute')))
    rows.append(case('request_kind_field', kind='Field', field=field(flags=255, patch='connection')))
    rows.append(case('index_ignores_bank_value', bank_value=MAX))
    rows.append(case('maximum_regions', region=MAX, selector_region=MAX, data_region=MAX))
    rows.append(case('selector_high_offset', selector=field(MAX-63, 63, 4), selector_region=MAX))
    rows.append(case('data_high_offset', data=field(MAX-63, 63, 4), data_region=MAX))
    for size in [4, 8]:
        rows.append(case(f'bank_normalize_before_fit_{size}', 'Bank', size=size,
                         bank_value=(1 << 32)+7, selector=field(256, 3, 1)))
    for fits in [True, False]:
        rows.append(case(f'bank_selector_63bit_{fits}', 'Bank',
                         bank_value=(1 << 63)-1 if fits else 1 << 63,
                         selector=field(256, 63, 4)))
    rows.append(case('bank_ignores_data_lock', 'Bank', data=field(515, 17, 18)))
    for selector_rule, data_rule in [(1, 2), (2, 1)]:
        rows.append(case(f'component_updates_{selector_rule}_{data_rule}', direction='Write',
                         selector=field(259, 9, 2 | (selector_rule << 5)),
                         data=field(515, 17, 2 | (data_rule << 5))))
    assert len({r['name'] for r in rows}) == len(rows)
    return deepcopy(rows)


if __name__ == '__main__':
    import json
    print(json.dumps(cases(), indent=2, sort_keys=True))
