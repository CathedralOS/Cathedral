"""Report Omega matrix control delays from paired value/clock diagnostics.

CPU is the reporting account's work during an interval, including other requests.
Elapsed minus CPU is off-CPU time, not proof of a particular IPC/scheduler cause.
Startup stamps correlate instances within a boot; they are not process identities.
"""
import argparse
from collections import defaultdict, deque
from dataclasses import dataclass
from pathlib import Path
import re
import statistics


@dataclass(frozen=True)
class Event:
    origin: int
    kind: int
    slot: int
    value: int
    micros: int
    cpu: int


STAMP = re.compile(r'Matrix t: ([0-9a-f]{16}) ([0-9])([0-7]) ([0-9a-f]{16}) ([0-9a-f]{16})')
VALUE = re.compile(r'Matrix control: ([0-9a-f]{16}) ([0-9]) ([0-7]) ([0-9a-f]{16})')
BACKGROUND = re.compile(r'Bg ctl: ([0-9a-f]{16}) ([0-3]) ([0-9a-f]{16}) ([0-9a-f]{16})')


def events(text):
    pending = defaultdict(deque)
    previous = {}
    found = []
    for line in text.splitlines():
        if match := STAMP.search(line):
            origin, kind, slot, micros, cpu = (int(v, 16) for v in match.groups())
            key = origin, kind, slot
            if not origin or not micros or not cpu:
                raise ValueError('clock stamp missing origin/time/CPU: ' + line)
            if origin in previous:
                old_time, old_cpu = previous[origin]
                if micros < old_time or cpu < old_cpu:
                    raise ValueError('clock/CPU regressed within one instance: ' + line)
            previous[origin] = micros, cpu
            pending[key].append((micros, cpu))
        elif match := VALUE.search(line):
            origin, kind, slot, value = (int(v, 16) for v in match.groups())
            key = origin, kind, slot
            if not pending[key]:
                raise ValueError('value without matching timing sample: ' + line)
            micros, cpu = pending[key].popleft()
            found.append(Event(origin, kind, slot, value, micros, cpu))
    if not found:
        raise ValueError('no instrumented matrix control events')
    if any(pending.values()):
        raise ValueError('incomplete final timing/value pair')
    return found


def spans(records):
    questions, queued, rows = {}, {}, []
    for event in records:
        key = event.origin, event.slot
        start, label = None, ''
        if event.kind == 1 and event.value == 0:
            questions[key] = event
        elif event.kind == 2:
            start = questions.pop(key, None)
            label = f'question status={event.value:016x}'
        elif event.kind == 9:
            queued[key + (event.value,)] = event
        elif event.kind == 8:
            start = queued.pop(key + (event.value,), None)
            label = f'queued opcode={event.value}'
        if start:
            wall, cpu = event.micros - start.micros, event.cpu - start.cpu
            if wall < 0 or cpu < 0:
                raise ValueError('control interval reversed')
            rows.append((event.origin, event.slot, wall / 1000, cpu / 1000, max(0, wall - cpu) / 1000, label))
    pending = [(e.origin, e.slot, e.micros, 'question') for e in questions.values()]
    pending += [(e.origin, e.slot, e.micros, f'queued opcode={e.value}') for e in queued.values()]
    return rows, pending


def background_spans(text):
    """Receive-to-reply only; startup tags do not establish matrix membership."""
    previous, pending, rows = {}, {}, []
    for match in BACKGROUND.finditer(text):
        origin, phase, micros, cpu = (int(value, 16) for value in match.groups())
        before = previous.get(origin, (0, 0))
        if not origin or not micros or not cpu or micros < before[0] or cpu < before[1]:
            raise ValueError(f'invalid/nonmonotonic Background stamp for {origin:016x}')
        previous[origin] = (micros, cpu)
        if phase == 0:
            continue
        if phase == 1:
            if origin in pending:
                raise ValueError(f'Background query overlaps for {origin:016x}')
            pending[origin] = (micros, cpu)
            continue
        start = pending.pop(origin, None)
        if start is None:
            raise ValueError(f'Background reply lacks receipt for {origin:016x}')
        wall, charged = micros - start[0], cpu - start[1]
        rows.append((origin, wall / 1000, charged / 1000,
                     max(0, wall - charged) / 1000, 'published' if phase == 2 else 'failed'))
    return rows, pending


def report(text):
    records = events(text)
    print('instance-start-us slot elapsed-ms account-CPU-ms off-CPU-ms operation')
    rows, pending = spans(records)
    for origin, slot, wall, cpu, other, label in rows:
        print(f'{origin:17d} {slot:4d} {wall:10.3f} {cpu:14.3f} {other:10.3f} {label}')
    for origin, slot, micros, label in pending:
        print(f'PENDING instance={origin} slot={slot} since-us={micros} {label}')
    # Status and loss are consecutive diagnostics from the same callback. Their
    # gap estimates the clock/format/write overhead; do not subtract it as though
    # every nested operation were serialized on one account.
    prior, costs = {}, []
    for event in records:
        last = prior.get(event.origin)
        if last and last.kind == 2 and event.kind == 3 and last.slot == event.slot:
            costs.append(event.cpu - last.cpu)
        prior[event.origin] = event
    if costs:
        print(f'Adjacent diagnostic pair CPU gap: median {statistics.median(costs)/1000:.3f} ms; max {max(costs)/1000:.3f} ms')
    print(f'{len(records)} complete samples across {len({e.origin for e in records})} instance tags; tracing perturbs execution.')
    leaf_rows, leaf_pending = background_spans(text)
    if leaf_rows:
        print('Background owner handling: received -> reply (not parent send -> reply)')
        print('instance-start-us elapsed-ms account-CPU-ms off-CPU-ms result')
        for origin, wall, cpu, other, label in leaf_rows:
            print(f'{origin:17d} {wall:10.3f} {cpu:14.3f} {other:10.3f} {label}')
    for origin, (micros, cpu) in leaf_pending.items():
        print(f'PENDING Background instance={origin} since-us={micros}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('serial', type=Path)
    args = parser.parse_args()
    report(args.serial.read_text(encoding='utf-8', errors='replace'))
