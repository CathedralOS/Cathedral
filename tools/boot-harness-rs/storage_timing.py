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
    'extent-end': ('extent-begin', 'extent growth'),
}


def summarize(output, serial, boot):
    pending, spans, io, retirements = {}, [], [], []
    text = serial.replace('Cathedral Rust lab: ', '')
    for label, numbers in re.findall(r'STORAGE_TRACE ([\w-]+) ([0-9 ]+)\r?\n', text):
        values = list(map(int, numbers.split()))
        if label in ('note-loaded', 'note-saved', 'note-budget', 'matrix-budget'): continue
        if label == 'storage-retire':
            assert len(values) == 5
            retirements.append(values)
            continue
        if label == 'storage-restart-failed':
            assert len(values) == 1
            continue
        if label == 'ata-io':
            assert len(values) == 7
            io.append(values)
            continue
        assert len(values) == 4
        scope, opcode, ticks, cpu = values
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
