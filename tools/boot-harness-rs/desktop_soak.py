"""One long desktop session: enter, edit, save and tear down a nested subtree
repeatedly, watching that memory returns, saves stay exact and providers live.

Run through desktop_test.py --soak CYCLES [--nested-depth D]. Boot one runs
every cycle on a fresh lab; boot two reopens the saved tree cold and runs two
more. Cycle times are host upper bounds; the heap samples come from the storage
provider's own allocator reports and exclude its stack and sealed reader pages.
"""
import json
import re
import time
from nested_desktop_test import Nested

# Serial markers that mean the session did not stay healthy on its own.
FAILURES = (
    'Cathedral desktop: storage replacement starting',
    'storage offline',
    'stalled storage cancelled',
    'matrix-service-exit',
    'STORAGE_TRACE collect-workspace-exhausted',
    'STORAGE_TRACE bulk-reply-refused',
)
# The provider must keep this much heap free at every report; closer than this
# and one copy or collection away from exhausting the allocator.
HEAP_HEADROOM = 32 * 1024
COLD_CYCLES = 2


def heap_reports(log):
    return [dict(phase=phase, live=int(live), peak=int(peak), failures=int(failures), capacity=int(capacity))
            for phase, live, peak, failures, capacity in re.findall(
                r'Cathedral objects heap: (\w+) live=(\d+) peak=(\d+) largest=\d+ failures=(\d+) capacity=(\d+)', log)]


def exercise(output, click, key, scan, width, height, reboot, cycles, pointer_stress=False, nested_depth=2, collection_pressure=False):
    n = Nested(click, key, scan, width, height, pointer_stress)
    serial = output / 'serial.log'
    path = output / f'soak-boot-{2 if reboot else 1}.json'
    records = []
    # The cold boot continues the fresh boot's numbering and expects its last save.
    done, saved = (cycles_before(output), last_token(output)) if reboot else (0, '')
    if reboot:
        cycles = COLD_CYCLES
    for cycle in range(1, cycles + 1):
        offset = serial.stat().st_size
        started = time.monotonic()
        phases = {}

        def phase(name, action):
            begin = time.monotonic()
            action()
            phases[name] = round(time.monotonic() - begin, 2)

        for depth in range(1, nested_depth + 1):
            phase(f'enter-{depth}', lambda depth=depth: n.enter(depth, cycle == 1 and not reboot))
        token = f'c{done + cycle}'

        def edit():
            n.launch(nested_depth, 0)
            n.note(nested_depth, saved)
            n.focus_note(nested_depth)
            for _ in saved:
                key('backspace')
            for ch in token:
                key(ch)
            n.note(nested_depth, token, True)

        def save():
            key('f1')
            n.note(nested_depth, token)

        def close_app():
            n.app_close(nested_depth)
            n.background(nested_depth)

        def teardown():
            n.task_close(0)
            n.background(0)

        phase('edit', edit)
        phase('save', save)
        saved = token
        phase('close-app', close_app)
        phase('teardown', teardown)
        elapsed = round(time.monotonic() - started, 2)
        with serial.open('rb') as stream:
            stream.seek(offset)
            log = stream.read().decode(errors='replace')
        heaps = heap_reports(log)
        failures = {marker: log.count(marker) for marker in FAILURES if marker in log}
        record = dict(cycle=cycle, token=token, seconds=elapsed, phases=phases,
                      windows=max((int(w) for w in re.findall(r'STORAGE_TRACE collect-pass \d+ \d+ \d+ \d+ (\d+) \d+', log)), default=0),
                      heap_live_end=heaps[-1]['live'] if heaps else None,
                      heap_peak=max((h['peak'] for h in heaps), default=None),
                      heap_capacity=heaps[-1]['capacity'] if heaps else None,
                      provider_requests=log.count('STORAGE_TRACE provider-begin'),
                      collection_passes=log.count('STORAGE_TRACE collect-pass'),
                      failures=failures)
        records.append(record)
        path.write_text(json.dumps(dict(nested_depth=nested_depth, reboot=reboot, cycles=records), indent=2))
        print(f"Soak cycle {cycle}: {elapsed}s; heap live {record['heap_live_end']} peak {record['heap_peak']} of {record['heap_capacity']}; "
              f"provider requests {record['provider_requests']}", flush=True)
        assert not failures, f'cycle {cycle} saw {failures}; logs: {output}'
        assert all(h['failures'] == 0 for h in heaps), f'provider allocation failed in cycle {cycle}: {heaps}'
        assert all(h['capacity'] - h['peak'] >= HEAP_HEADROOM for h in heaps), f'provider heap headroom lost in cycle {cycle}: {heaps}'
        if collection_pressure:
            # The pressure build collects before every write inside a tiny
            # workspace, so every cycle must show passes that split windows.
            assert record['collection_passes'] and record['windows'] > 4, f'cycle {cycle} ran no windowed collection under pressure: {record}'
    # The outer realm never sees the inner note, however many cycles ran.
    n.launch(0, 0)
    n.note(0, '')
    n.app_close(0)
    n.background(0)
    print(f'PASS: {cycles} soak cycles through {nested_depth} nested desktops; last saved {saved!r}')


def cycles_before(output):
    """Cycles the fresh boot completed, so the cold boot continues their numbering."""
    first = output / 'soak-boot-1.json'
    if not first.exists():
        return 0
    return len(json.loads(first.read_text())['cycles'])


def last_token(output):
    """The text the fresh boot saved last, which the cold boot must find."""
    first = output / 'soak-boot-1.json'
    if not first.exists():
        return ''
    cycles = json.loads(first.read_text())['cycles']
    return cycles[-1]['token'] if cycles else ''


def report(output):
    for path in sorted(output.glob('soak-boot-*.json')):
        data = json.loads(path.read_text())
        print(f"{path.name}: depth {data['nested_depth']}, {'cold' if data['reboot'] else 'fresh'} boot")
        for c in data['cycles']:
            phases = ' '.join(f"{name} {seconds}" for name, seconds in c.get('phases', {}).items())
            print(f"  cycle {c['cycle']:3} {c['seconds']:7.2f}s heap live {c['heap_live_end']} peak {c['heap_peak']} / {c['heap_capacity']} "
                  f"requests {c['provider_requests']} collections {c['collection_passes']} (windows up to {c.get('windows', 0)}) failures {c['failures'] or 'none'}; {phases}")
        times = [c['seconds'] for c in data['cycles']]
        if len(times) >= 4:
            first, last = sum(times[:2]) / 2, sum(times[-2:]) / 2
            print(f"  first two cycles {first:.2f}s, last two {last:.2f}s (host upper bounds, not a trend proof)")


if __name__ == '__main__':
    import argparse
    from pathlib import Path
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('artifact_directory', type=Path)
    report(parser.parse_args().artifact_directory)
