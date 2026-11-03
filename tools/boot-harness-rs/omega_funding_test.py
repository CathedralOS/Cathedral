"""A held parent FUND must leave the stock Omega desktop usable.
The launch retains its image; a refused or revoked launch must not admit a child.
"""
import argparse
import re
from functools import partial

import omega_desktop_test
import omega_parent_poll_test
from nested_desktop_test import Nested


def exercise(output, click, key, scan, width, height, reboot, refuse=False, regrant=False, nested=False):
    n = Nested(click, key, scan, width, height)
    log = lambda: (output / 'serial.log').read_text(encoding='utf-8', errors='replace')
    depth = 1 if nested else 0
    if nested:
        n.enter(1, not reboot)
    n.launch(depth, 0)
    value = 'a' if reboot else ''
    n.note(depth, value + ' ')
    start = len(log())
    count = 3 if regrant else 2 if refuse else 1
    for marker in ['funding-hold-armed', 'funding-refusal-armed', 'funding-regrant-armed'][:count]:
        key('f2')
        scan(lambda _w, _d, marker=marker: marker in log()[start:], marker)
    n.launch(depth, 2 if nested else 3)  # The larger nested OS needs a new resource grant.
    scan(lambda _w, _d: 'funding-held' in log()[start:], 'nested launch funding held')
    n.focus_note(depth)
    key('b' if reboot else 'a')
    value += 'b' if reboot else 'a'
    # Real app rendering must progress before FUND completes.
    n.note(depth, value + ' ', True)
    end = 'funding-refused' if refuse else 'funding-released'
    assert end not in log()[start:], 'rendering must finish during the hold'
    scan(lambda _w, _d: end in log()[start:], 'funding settled')
    events = log()[start:]
    for marker in ['funding-health-during-hold', 'funding-poll-during-hold', 'funding-render-during-hold']:
        assert events.index('funding-held') < events.index(marker) < events.index(end)
    if regrant:
        assert events.index('funding-held') < events.index('funding-grant-revoked') < events.index(end)
    if refuse or regrant:
        if regrant:
            scan(lambda _w, _d: 'funding-grant-restored' in log()[start:], 'launcher scope restored')
        scan(lambda _w, _d: 'Cathedral matrix: launch refused' in log()[start:], 'old launch refused')
        assert not re.search(r'Cathedral matrix: launch 2 ', log()[start:]), log()[start:]
        n.note(depth, value + ' ', True)
        n.launch(depth, 2 if nested else 3)
    x, y, w, h = n.viewport(depth + 1)
    n.label(x + (w - 128) // 2 + 16, y + (h - 96) // 2 + 76, 'NEW USER')
    n.task_close(depth, 1)
    n.note(depth, value + ' ', True)
    assert log().count('note-loaded') == 1
    n.focus_note(depth)
    key('f1')
    n.note(depth, value + ' ')
    n.app_close(depth)
    n.background(depth)
    if nested:
        n.task_close(0)
        n.background(0)
    assert not omega_desktop_test.failed_children(log()), log()[-4000:]
    print('PASS: FUND wait preserves HEALTH, input, rendering and the exact cold-saved draft')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group()
    group.add_argument('--refuse', action='store_true')
    group.add_argument('--regrant', action='store_true')
    group.add_argument('--nested', action='store_true', help='hold funding forwarded through intermediate Omega matrices')
    args = parser.parse_args()
    omega_parent_poll_test.main(user_check=partial(exercise, refuse=args.refuse, regrant=args.regrant, nested=args.nested),
                               label='omega-funding', fixture_features=['funding-delay'])
