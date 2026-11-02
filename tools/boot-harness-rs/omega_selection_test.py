"""The stock Omega matrix switches existing apps while its parent holds COMMIT.
The old visible app must receive input, and a refusal must preserve both apps.
"""
import omega_parent_poll_test
import omega_desktop_test
from nested_desktop_test import Nested
import argparse


def exercise(output, click, key, scan, width, height, reboot):
    n = Nested(click, key, scan, width, height)
    log = lambda: (output / 'serial.log').read_text(encoding='utf-8', errors='replace')

    def select(index):
        click(316 + 144 * index, height - 48)

    n.launch(0, 1)
    n.color(0, False)
    n.launch(0, 0)
    base = 'abc' if reboot else ''
    n.note(0, base + ' ')
    n.focus_note(0)
    key('d' if reboot else 'a')
    before = base + ('d' if reboot else 'a')
    n.note(0, before + ' ', True)

    def held(value, incoming, refuse=False):
        start = len(log())
        key('f3')
        scan(lambda _w, _d: 'interaction-delay-armed' in log()[start:], 'selection gate armed')
        select(0)
        scan(lambda _w, _d: 'interaction-delay-held' in log()[start:], 'selection commit held')
        n.note(0, value + ' ', True)
        n.focus_note(0)
        key(incoming)
        if refuse:
            key('f3')
            scan(lambda _w, _d: 'commit-refusal-armed' in log()[start:], 'selection refusal armed')
        for marker in ('commit-health-during-hold', 'commit-poll-during-hold'):
            scan(lambda _w, _d: marker in log()[start:], marker)
            events = log()[start:]
            end = 'commit-refused' if refuse else 'interaction-delay-released'
            if end in events:
                assert events.index(marker) < events.index(end)
        scan(lambda _w, _d: end in log()[start:], 'selection released')
        if refuse:
            scan(lambda _w, _d: 'commit-abort-received' in log()[start:], 'selection abort matched')
        else:
            n.color(0, False)
            select(1)
        # The key typed against the old visible app survives exactly once.
        n.note(0, value + incoming + ' ', True)
        assert 'matrix-reclaimed' not in log()[start:], 'selection must not replace either app'
        assert 'matrix: reclaimed' not in log()[start:], 'Omega selection must retain both apps'
        return value + incoming

    before = held(before, 'e' if reboot else 'b')
    before = held(before, 'f' if reboot else 'c', refuse=True)
    for _ in range(2):
        select(0)
        n.color(0, False)
        select(1)
        n.note(0, before + ' ', True)
    assert log().count('note-loaded') == 1
    n.focus_note(0)
    key('f1')
    n.note(0, before + ' ')
    n.app_close(0)
    n.color(0, False)
    n.app_close(0)
    n.background(0)
    assert not omega_desktop_test.failed_children(log()), log()[-4000:]
    print('PASS: held Omega selection serves HEALTH/input; old app gets exact keys; refusal preserves both apps; repeated switching and cold save')


def cancellation(output, click, key, scan, width, height, reboot, late):
    n = Nested(click, key, scan, width, height)
    log = lambda: (output / 'serial.log').read_text(encoding='utf-8', errors='replace')
    select = lambda index: click(316 + 144 * index, height - 48)
    n.launch(0, 1)
    n.color(0, False)
    n.launch(0, 0)
    base = 'ab' if reboot else ''
    n.note(0, base + ' ')
    n.focus_note(0)
    key('a')
    n.note(0, base + 'a ', True)
    start = len(log())
    key('f3' if late else 'f5')
    scan(lambda _w, _d: 'interaction-delay-armed' in log()[start:], 'selection gate armed')
    select(0)
    scan(lambda _w, _d: 'interaction-delay-held' in log()[start:], 'selection held')
    marker = 'commit-health-during-hold' if late else 'image-health-during-hold'
    scan(lambda _w, _d: marker in log()[start:], 'owner responds during selection')
    n.note(0, base + 'a ', True)
    n.focus_note(0)
    key('b')
    key('f6')  # The test parent uses its real owner link for LOSS then STOP.
    scan(lambda _w, _d: 'selection-stop-requested' in log()[start:], 'stop pending')
    assert 'interaction-delay-released' not in log()[start:]
    scan(lambda _w, _d: 'selection-stop-acknowledged' in log()[start:], 'stop settled')
    events = log()[start:]
    assert events.index('selection-loss-busy') < events.index('selection-stop-requested')
    assert events.index('selection-stop-requested') < events.index('interaction-delay-released')
    assert 'selection cancelled' in events
    assert ('selection restored' in events) == late
    assert events.index('selection cancelled') < events.index('selection-stop-acknowledged')
    assert events.index('matrix: reclaimed 1 ') < events.index('selection-stop-acknowledged')
    n.note(0, base + 'ab ', True)
    # Reuse the stopped slot, then switch back to the same Notes and its draft.
    n.launch(0, 1)
    n.color(0, False)
    select(1)  # Colors reused slot 1; the same Notes still occupies slot 2.
    n.note(0, base + 'ab ', True)
    assert log().count('note-loaded') == 1
    n.focus_note(0)
    key('f1')
    n.note(0, base + 'ab ')
    n.app_close(0)
    n.color(0, False)
    n.app_close(0)
    n.background(0)
    assert not omega_desktop_test.failed_children(log()), log()[-4000:]
    print(f'PASS: Omega selection cancellation {"late" if late else "before commit"}; LOSS busy, STOP settled, draft and slot reuse survive cold reopen')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cancel', choices=['before', 'late'])
    args = parser.parse_args()
    check = exercise if args.cancel is None else lambda *values: cancellation(*values, late=args.cancel == 'late')
    omega_parent_poll_test.main(user_check=check, label='omega-selection-' + (args.cancel or 'ordinary'))
