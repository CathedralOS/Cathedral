"""A Rust parent holds the real Omega matrix's sealed storage publication.

HEALTH and a fresh input poll must precede release. Exact text, a second save,
slot reuse and cold reopen check that the retained exchange settles normally.
"""
import argparse
from functools import partial

import omega_parent_poll_test
import omega_desktop_test
from nested_desktop_test import Nested


def exercise(output, click, key, scan, width, height, reboot, depth=0):
    n = Nested(click, key, scan, width, height)
    log = lambda: (output / 'serial.log').read_text(errors='replace')
    if depth:
        n.enter(depth, not reboot)
    n.launch(depth, 0)
    before = 'ab' if reboot else ''
    n.note(depth, before + ' ')
    n.focus_note(depth)
    for letter in ('cd' if reboot else 'ab'):
        key(letter)
        before += letter
        n.note(depth, before + ' ', dirty=True)
        start = len(log())
        key('f2')
        scan(lambda _w, _d: 'storage-delay-armed' in log()[start:], 'storage hold armed')
        key('f1')
        scan(lambda _w, _d: 'storage-delay-released' in log()[start:], 'storage released after independent work')
        events = log()[start:]
        for marker in ('storage-health-during-hold', 'storage-poll-during-hold'):
            assert events.index('storage-delay-held') < events.index(marker) < events.index('storage-delay-released')
        assert events.count('storage-delay-held') == 1
        n.note(depth, before + ' ')
    n.app_close(depth)
    n.background(depth)
    n.launch(depth, 0)
    n.note(depth, before + ' ')
    n.app_close(depth)
    n.background(depth)
    if depth:
        n.task_close(0)
        n.background(0)
    assert not omega_desktop_test.failed_children(log()), log()[-4000:]
    print('PASS: Omega HEALTH and input while storage holds its sealed input; repeat save, slot reuse and cold reopen')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--nested', action='store_true', help='write through an additional nested desktop and its scope checks')
    args = parser.parse_args()
    omega_parent_poll_test.main(user_check=partial(exercise, depth=1 if args.nested else 0), label='omega-storage')
