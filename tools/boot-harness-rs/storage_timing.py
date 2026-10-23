"""Summarize opt-in local storage spans; clocks are guest ticks/account CPU."""
import csv
import re

PAIRS = {
    'note-save-end': ('note-save-begin', 'note save'),
    'root-admit': ('root-receive', 'root dispatch'),
    'nested-admit': ('nested-receive', 'nested dispatch'),
    'provider-begin': ('root-send', 'provider queue'),
    'app-reply': ('app-send', 'app roundtrip'),
    'root-reply': ('root-admit', 'root service'),
    'nested-reply': ('nested-admit', 'nested service'),
    'provider-end': ('provider-begin', 'provider request'),
    'publish-begin': ('collect-begin', 'collection'),
    'committed': ('commit-begin', 'commit'),
}


def provider_samples(serial):
    """One serialized provider, including nested phases; never join matrix CPU clocks."""
    samples = []
    pending = {}
    begins = {'provider-begin': 'provider', 'collect-begin': 'collection', 'commit-begin': 'commit'}
    ends = {'provider-end': 'provider', 'publish-begin': 'collection', 'committed': 'commit'}
    for line in serial.replace('Cathedral Rust lab: ', '').splitlines():
        match = re.search(r'STORAGE_TRACE (\S+) ([\d ]+)$', line)
        if not match:
            continue
        label, numbers = match.groups()
        values = [int(value) for value in numbers.split()]
        # Stage lines carry scope, opcode, tick and account CPU, then the
        # microsecond clock; only the first four matter here.
        if label in begins and len(values) in (4, 5):
            pending[begins[label]] = (values[:4], {})
        elif label == 'ata-io' and len(values) == 7 and 'provider' in pending:
            start, extra = pending['provider']
            if values[:2] == start[:2]:
                extra.update(zip(('read_sectors', 'written_sectors', 'flushes', 'wait_ticks', 'polls'), values[2:]))
        elif label in ends and len(values) in (4, 5):
            values = values[:4]
            phase = ends[label]
            entry = pending.pop(phase, None)
            if entry is None:
                continue
            start, extra = entry
            if values[:2] != start[:2] or any(end < begin for begin, end in zip(start[2:], values[2:])):
                continue
            samples.append(dict(phase=phase, subject=start[0], operation=start[1],
                                guest_ticks=values[2]-start[2], cpu_us=values[3]-start[3], **extra))
    return samples



def summarize(output, serial, boot):
    pending, spans, io, retirements = {}, [], [], []
    text = serial.replace('Cathedral Rust lab: ', '')
    for label, numbers in re.findall(r'STORAGE_TRACE ([\w-]+) ([0-9 ]+)\r?\n', text):
        values = list(map(int, numbers.split()))
        if label in ('note-loaded', 'note-saved', 'note-budget', 'matrix-budget'): continue
        if label in ('funding-shortfall', 'funding-granted', 'funding-policy-refused', 'matrix-reclaimed'):
            assert len(values) == 5
            continue
        if label == 'funding-surface-refused':
            assert len(values) == 2
            continue
        if label in ('funding-transfer-refused', 'funding-local-slots-refused'):
            assert len(values) == 1
            continue
        if label in ('matrix-reserve-refused', 'matrix-start-refused', 'split-admission-refused', 'matrix-local-capacity'):
            assert len(values) == (5 if label in ('matrix-reserve-refused', 'matrix-start-refused') else 6)
            continue
        if label == 'storage-retire':
            assert len(values) == 5
            retirements.append(values)
            continue
        if label in ('storage-restart-failed', 'collect-workspace-exhausted'):
            assert len(values) == 1
            continue
        if label in ('collect-pass', 'bulk-reply-refused'):
            assert len(values) == 6
            continue
        if label in ('surface-create-refused', 'surface-replay-failed'):
            continue
        if label == 'ata-io':
            assert len(values) == 7
            io.append(values)
            continue
        assert len(values) in (4, 5), (label, values)
        scope, opcode, ticks, cpu = values[:4]
        if label in PAIRS:
            begin, stage = PAIRS[label]
            previous = pending.pop((begin, scope, opcode), None)
            cross_process = stage == 'provider queue'
            if previous and (cross_process or cpu >= previous[1]) and ticks >= previous[0]:
                spans.append([stage, scope, opcode, previous[0], ticks - previous[0], '' if cross_process else cpu - previous[1]])
        pending[label, scope, opcode] = ticks, cpu
    assert {'app roundtrip', 'root service', 'nested service', 'provider request'} <= {row[0] for row in spans}
    with (output / f'timing-{boot}.csv').open('w', newline='', encoding='utf-8') as file:
        writer = csv.writer(file)
        writer.writerow(['stage', 'local_scope', 'opcode', 'start_tick', 'elapsed_ticks', 'account_cpu_us'])
        writer.writerows(spans)
    with (output / f'ata-{boot}.csv').open('w', newline='', encoding='utf-8') as file:
        writer = csv.writer(file)
        writer.writerow(['root', 'opcode', 'reads', 'writes', 'flushes', 'wait_ticks', 'polls'])
        writer.writerows(io)
    with (output / f'retirements-{boot}.csv').open('w', newline='', encoding='utf-8') as file:
        writer = csv.writer(file)
        writer.writerow(['upstream', 'opcode', 'deadline', 'tick', 'error'])
        writer.writerows(retirements)
    assert io
    if boot == 1:
        assert any(row[4] > 0 for row in io)
        assert any(row[0] == 'commit' for row in spans)
    print(f'TIMING: {len(spans)} completed local spans, {len(io)} ATA samples; nominal tick=10ms; all completed spans included; see context for injected faults')
