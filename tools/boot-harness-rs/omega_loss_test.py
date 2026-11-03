"""Query LOSS through an ordinary image hold; drawing is not unsaved work.
Selection/lifecycle barriers have their own BUSY tests.
"""
import argparse
import re
from functools import partial
import omega_parent_poll_test
import omega_desktop_test
from nested_desktop_test import Nested


def exercise(output, click, key, scan, width, height, reboot, nested=False):
    n = Nested(click, key, scan, width, height)
    depth = 1 if nested else 0
    log = lambda: (output / 'serial.log').read_text(encoding='utf-8', errors='replace')
    if nested:
        n.enter(1, not reboot)
    n.launch(depth, 0)
    value = 'a' if reboot else ''
    n.note(depth, value + ' ')
    n.focus_note(depth)

    def held(action, loss):
        start = len(log())
        key('f5')
        scan(lambda _w, _d: 'interaction-delay-armed' in log()[start:], 'image gate armed')
        action()
        scan(lambda _w, _d: 'image-loss-during-hold' in log()[start:], 'LOSS answered during image hold')
        events = log()[start:]
        reply = re.search(r'image-loss-during-hold (\d+) (\d+)', events)
        assert reply and tuple(map(int, reply.groups())) == (0, loss), events
        if 'interaction-delay-released' in events:
            assert events.index('image-loss-during-hold') < events.index('interaction-delay-released')
        scan(lambda _w, _d: 'interaction-delay-released' in log()[start:], 'image released')

    # Establish the state before arming: a nested ancestor may still have an
    # older repaint queued, which must not race the edit this test measures.
    key('b' if reboot else 'a')
    value += 'b' if reboot else 'a'
    n.note(depth, value + ' ', True)
    held(lambda: key('left'), 1)
    key('right')
    n.note(depth, value + ' ', True)
    start = len(log())
    n.task_close(depth)
    scan(lambda _w, _d: re.search(r'Matrix control: [0-9a-f]+ 6 [0-4] ', log()[start:]),
         'dirty close refused')
    n.note(depth, value + ' ', True)
    n.focus_note(depth)
    key('f1')
    n.note(depth, value + ' ')
    held(lambda: key('left'), 0)
    n.note(depth, value + ' ')
    n.app_close(depth)
    n.background(depth)
    if nested:
        n.task_close(0)
        n.background(0)
    assert not omega_desktop_test.failed_children(log()), log()[-4000:]
    print('PASS: dirty and clean LOSS during ordinary image holds; protected close and exact cold save')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--nested', action='store_true')
    args = parser.parse_args()
    omega_parent_poll_test.main(user_check=partial(exercise, nested=args.nested),
                               label='omega-loss', fixture_features=['loss-query-probe'])
