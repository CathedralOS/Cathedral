"""Launch into a workspace while its parent holds the activation COMMIT.
The old app keeps input; refusal/cancellation reclaims only the new child.
"""
import argparse
from functools import partial

import omega_desktop_test
import omega_parent_poll_test
from nested_desktop_test import Nested


def exercise(output, click, key, scan, width, height, reboot, cancel=False):
    n = Nested(click, key, scan, width, height)
    log = lambda: (output / 'serial.log').read_text(encoding='utf-8', errors='replace')
    select = lambda index: click(316 + 144 * index, height - 48)
    # Keep Notes in slot 2 so the existing owner STOP probe targets the newly
    # admitted Colors in slot 1, never the old app or its unsaved draft.
    n.launch(0, 1)
    n.color(0, False)
    n.launch(0, 0)
    value = 'ab' if reboot else ''
    n.note(0, value + ' ')
    select(0)
    n.color(0, False)
    n.app_close(0)
    n.note(0, value + ' ')

    def launch_held(value, incoming, reject):
        start = len(log())
        key('f3')
        scan(lambda _w, _d: 'interaction-delay-armed' in log()[start:], 'launch scene gate armed')
        n.launch(0, 1)
        scan(lambda _w, _d: 'interaction-delay-held' in log()[start:], 'launch activation held')
        n.note(0, value + ' ', value != ('ab' if reboot else ''))
        n.focus_note(0)
        key(incoming)
        if reject:
            key('f6' if cancel else 'f3')
            armed = 'selection-stop-requested' if cancel else 'commit-refusal-armed'
            scan(lambda _w, _d: armed in log()[start:], armed)
        end = 'commit-refused' if reject and not cancel else 'interaction-delay-released'
        scan(lambda _w, _d: end in log()[start:], 'activation released')
        events = log()[start:]
        for marker in ('commit-health-during-hold', 'commit-poll-during-hold'):
            assert events.index('interaction-delay-held') < events.index(marker) < events.index(end)
        if reject:
            settled = 'selection-stop-acknowledged' if cancel else 'commit-abort-received'
            scan(lambda _w, _d: settled in log()[start:], settled)
            scan(lambda _w, _d: 'Cathedral matrix: launch refused' in log()[start:], 'new launch refused')
            events = log()[start:]
            assert 'matrix: reclaimed 1 ' in events
            assert events.index('matrix: reclaimed 1 ') < events.index('Cathedral matrix: launch refused'), events
            assert 'matrix: reclaimed 2 ' not in events
            if cancel:
                assert events.index('selection-stop-requested') < events.index(end)
                assert 'selection restored' in events
        else:
            n.color(0, False)
            select(1)
        n.note(0, value + incoming + ' ', True)
        return value + incoming

    value = launch_held(value, 'a', True)
    value = launch_held(value, 'b', False)
    assert log().count('note-loaded') == 1
    n.focus_note(0)
    key('f1')
    n.note(0, value + ' ')
    n.app_close(0)
    n.color(0, False)
    n.app_close(0)
    n.background(0)
    assert not omega_desktop_test.failed_children(log()), log()[-4000:]
    print('PASS: held launch activation serves HEALTH/input; old draft survives; '
          + ('STOP rolls back' if cancel else 'refusal reclaims')
          + ' new child; slot reuse, successful activation and cold save')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cancel', action='store_true', help='STOP the new child after COMMIT was sent')
    args = parser.parse_args()
    omega_parent_poll_test.main(user_check=partial(exercise, cancel=args.cancel), label='omega-launch-scene')
