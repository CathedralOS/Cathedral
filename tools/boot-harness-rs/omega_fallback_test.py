"""Closing the active app must not block the matrix on its fallback scene.

The existing Rust parent holds COMMIT and probes the ordinary Omega child.
The collected Colors slot stays unavailable until its fallback transaction
settles. Notes keeps its draft unless its owner explicitly stops it.
"""
import argparse
from functools import partial

import omega_desktop_test
import omega_parent_poll_test
from nested_desktop_test import Nested


def exercise(output, click, key, scan, width, height, reboot, mode='complete'):
    n = Nested(click, key, scan, width, height)
    log = lambda: (output / 'serial.log').read_text(encoding='utf-8', errors='replace')
    n.launch(0, 0)
    base = 'a' if reboot else ''
    n.note(0, base + ' ')
    n.focus_note(0)
    key('b' if reboot else 'a')
    base += 'b' if reboot else 'a'
    n.note(0, base + ' ', True)
    key('f1')
    n.note(0, base + ' ')
    key('x')
    n.note(0, base + 'x ', True)
    n.launch(0, 1)
    n.color(0, False)

    start = len(log())
    events = lambda: log()[start:]
    key('f3')
    scan(lambda _w, _d: 'interaction-delay-armed' in events(), 'fallback commit gate armed')
    n.task_close(0, 1)
    scan(lambda _w, _d: 'interaction-delay-held' in events(), 'fallback commit held')
    # Parent polling is notification-driven. Supply an actual input instead of
    # depending on incidental pointer motion to ring the matrix's input bell.
    key('left')
    for marker in ('commit-health-during-hold', 'commit-poll-during-hold'):
        scan(lambda _w, _d: marker in events(), marker)
        assert 'interaction-delay-released' not in events(), events()
    text = events()
    assert text.index('Cathedral matrix: child 2 ') < text.index('interaction-delay-held'), text
    assert 'Cathedral matrix: reclaimed 2 ' not in text, text

    if mode == 'refuse':
        key('f3')
        scan(lambda _w, _d: 'commit-refusal-armed' in events(), 'fallback refusal armed')
        scan(lambda _w, _d: 'commit-abort-received' in events(), 'refused fallback aborted')
        settled = 'commit-abort-received'
    elif mode == 'cancel':
        key('f6')  # The existing owner probe stops slot 1, our fallback Notes.
        scan(lambda _w, _d: 'selection-stop-requested' in events(), 'fallback target stop requested')
        assert 'interaction-delay-released' not in events(), events()
        scan(lambda _w, _d: 'selection-stop-acknowledged' in events(), 'fallback target stop settled')
        text = events()
        assert text.index('selection-loss-busy') < text.index('selection-stop-requested'), text
        assert 'selection cancelled' in text and 'selection restored' in text, text
        settled = 'interaction-delay-released'
    else:
        scan(lambda _w, _d: 'interaction-delay-released' in events(), 'fallback commit released')
        settled = 'interaction-delay-released'

    scan(lambda _w, _d: 'Cathedral matrix: reclaimed 2 ' in events(), 'closed slot settled')
    text = events()
    assert text.index(settled) < text.index('Cathedral matrix: reclaimed 2 '), text
    if mode == 'cancel':
        n.background(0)
        n.launch(0, 0)
        n.note(0, base + ' ')
    else:
        if mode == 'refuse':
            click(316, height - 48)  # Explicitly select the retained Notes instance.
        n.note(0, base + 'x ', True)
        assert log().count('note-loaded') == 1, log()[-4000:]
        n.focus_note(0)
        key('backspace')
        # Returning to the confirmed bytes makes Notes clean immediately.
        n.note(0, base + ' ')
    n.app_close(0)
    n.background(0)
    n.launch(0, 0)
    n.note(0, base + ' ')
    n.app_close(0)
    n.background(0)
    assert not omega_desktop_test.failed_children(log()), log()[-4000:]
    print(f'PASS: fallback {mode}; owner progress, retirement ordering, saved state and slot reuse', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--mode', choices=('complete', 'refuse', 'cancel'), default='complete')
    args = parser.parse_args()
    omega_parent_poll_test.main(user_check=partial(exercise, mode=args.mode),
                               label='omega-fallback-' + args.mode)
