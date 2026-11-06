"""Real matrix admission observation paused; ordinary owner IPC must still work.

This is an observation hold, not a kernel pre-publication gate. The worker may
commit while held. The generic admission witness separately covers both sides
of that race. Here we check the matrix's authority, cleanup and reuse boundary.
"""
import argparse
from functools import partial
import re

import omega_desktop_test
import omega_parent_poll_test
from nested_desktop_test import Nested


def exercise(output, click, key, scan, width, height, reboot, mode='health', rounds=1):
    value = 'a' * rounds if reboot else ''
    baseline = None
    for round_index in range(rounds):
        value, samples = cycle(output, click, key, scan, width, height, reboot, mode, value)
        if baseline is None:
            baseline = samples[0]
        assert all(sample == baseline for sample in samples), (baseline, samples, round_index)
        print(f'PASS: construction {mode} round {round_index + 1}/{rounds}; '
              f'reclaimed account {baseline}', flush=True)


def cycle(output, click, key, scan, width, height, reboot, mode, value):
    n = Nested(click, key, scan, width, height)
    log = lambda: (output / 'serial.log').read_text(encoding='utf-8', errors='replace')
    def drive_input(offset):
        # Parent polling is demand-driven. Generate work after the probe starts;
        # do not mistake a quiet matrix for a blocked matrix.
        key('left')
        scan(lambda _w, _d: 'admission-input-first-poll' in log()[offset:], 'first fresh input poll')
        key('right')
        scan(lambda _w, _d: 'admission-input-polled' in log()[offset:], 'second fresh input poll')
    if mode == 'regrant':
        # Establish an explicit ordinary-import policy first. Restoring it during the
        # held launch must restore exactly that policy, not a different default.
        primed = len(log())
        key('f6')
        drive_input(primed)
        scan(lambda _w, _d: 'admission-probe-complete' in log()[primed:], 'explicit policy established')
    start = len(log())
    events = lambda: log()[start:]
    n.launch(0, 0)
    held = 'Matrix admission probe: held'
    released = 'Matrix admission probe: released'
    scan(lambda _w, _d: held in events(), 'construction observation held')
    key({'health': 'f8', 'stop': 'f7', 'regrant': 'f6', 'requester': 'f5'}[mode])
    scan(lambda _w, _d: 'admission-health' in events(), 'owner HEALTH during construction')
    drive_input(start)
    for marker in ('admission-health', 'admission-input-polled'):
        text = events()
        assert text.index(held) < text.index(marker), text
        assert released not in text or text.index(marker) < text.index(released), text
    assert events().count(held) == 1, events()
    if mode == 'health':
        scan(lambda _w, _d: released in events(), 'construction observation released')
    if mode != 'health':
        marker = {'stop': 'admission-stopped', 'regrant': 'admission-policy-restored',
                  'requester': 'admission-requester-stopped'}[mode]
        scan(lambda _w, _d: marker in events(), marker)
        scan(lambda _w, _d: 'Cathedral matrix: launch refused' in events(), 'old launch refused')
        text = events()
        assert 'Cathedral matrix: launch refused fffffffffffffff3' in text, text
        assert released not in text, text
        assert not re.search(r'Cathedral matrix: launch 1 ', text), text
        if mode == 'regrant':
            assert text.index('admission-policy-revoked') < text.index('admission-policy-restored'), text
        if mode == 'requester':
            scan(lambda _w, _d: 'admission-requester-replaced' in events(), 'requester replaced')
            assert events().count('admission-requester-refused') == 1, events()
            failed = re.search(r'Cathedral matrix: launch 0 0000000000000005 ffffffffffffffea', events())
            assert failed, events()
            reclaimed = events().index('Matrix unused reservation released: 0', failed.end())
            assert reclaimed < events().index('admission-requester-refused'), events()
            scan(lambda w, d: n.pixel(w, d, 32, height - 56) == bytes.fromhex('d6e5ef'),
                 'replacement launcher painted')
            assert events().index('Cathedral matrix: launch refused') < events().index('admission-requester-replaced'), events()
            assert re.search(r'Cathedral matrix: child 0 0000000000000003 0000000000000000', events()), events()
        # The same target must work after cancellation, including for a fresh requester.
        n.background(0)
        n.launch(0, 0)
    n.note(0, value + ' ')
    n.focus_note(0)
    key('b' if reboot else 'a')
    value += 'b' if reboot else 'a'
    n.note(0, value + ' ', dirty=True)
    key('f1')
    n.note(0, value + ' ')
    closing = len(log())
    n.app_close(0)
    n.background(0)
    n.launch(0, 0)
    n.note(0, value + ' ')
    n.app_close(0)
    n.background(0)
    assert not omega_desktop_test.failed_children(log()), log()[-4000:]
    # Sample only the two completed Notes closes, excluding cancellation and
    # requester teardown. Compare identical points across repeated cycles; this
    # does not claim all asynchronous reservation returns have settled.
    def accounts():
        return [tuple(int(word, 16) for word in match) for match in re.findall(
            r'Cathedral matrix: reclaimed 1 ([0-9a-f]{4}) ([0-9a-f]{4}) ([0-9a-f]{4}) ([0-9a-f]{4})',
            log()[closing:])]
    scan(lambda _w, _d: len(accounts()) >= 2, 'both Notes accounts reclaimed')
    samples = accounts()
    assert len(samples) == 2, samples
    return value, samples


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--mode', choices=('health', 'stop', 'regrant', 'requester'), default='health')
    parser.add_argument('--rounds', type=int, choices=range(1, 9), default=1,
                        help='repeat construction and two Notes closes per boot (default: 1)')
    args = parser.parse_args()
    omega_parent_poll_test.main(user_check=partial(exercise, mode=args.mode, rounds=args.rounds),
                               label='omega-construction', fixture_features=['construction-probe'],
                               matrix_entry='distribution/lab/matrix-construction/native/main.omg')
