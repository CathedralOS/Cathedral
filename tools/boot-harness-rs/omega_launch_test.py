"""Hold launch storage work while the real Omega matrix services control and input.

The Rust fixture alone holds an executable read or its binding refresh. An invalid executable additionally
checks that a nested login restores its picker after failed admission.
"""
import argparse
import re
from functools import partial

import omega_desktop_test
import omega_parent_poll_test
from nested_desktop_test import Nested


def exercise(output, click, key, scan, width, height, reboot, invalid=False, regrant=False, binding=False):
    n = Nested(click, key, scan, width, height)
    log = lambda: (output / 'serial.log').read_text(encoding='utf-8', errors='replace')
    if invalid:
        # This is the Omega nested login, not the fixture parent's own picker.
        n.launch(0, 3)
        x, y, w, h = n.viewport(1)
        top = y + (h - 96) // 2
        if not reboot:
            left = x + (w - 128) // 2
            n.label(left + 16, top + 76, 'NEW USER')
            click(left + 64, top + 48)
        left = x + (w - 280) // 2
        n.label(left + 28, top + 76, 'USER 1')
    start = len(log())
    markers = ['storage-delay-armed', 'storage-read-armed', 'storage-invalid-armed',
               'storage-regrant-armed', 'storage-binding-armed', 'storage-binding-regrant-armed']
    count = (6 if regrant else 5) if binding else 4 if regrant else 3 if invalid else 2
    for marker in markers[:count]:
        key('f2')
        scan(lambda _w, _d, marker=marker: marker in log()[start:], marker)
    if invalid:
        click(left + 64, top + 48)
    else:
        n.launch(0, 0)
    scan(lambda _w, _d: 'storage-delay-released' in log()[start:], 'storage operation released after independent work')
    events = log()[start:]
    for marker in ('storage-health-during-hold', 'storage-poll-during-hold'):
        assert events.index('storage-delay-held') < events.index(marker) < events.index('storage-delay-released')
    assert events.count('storage-binding-held' if binding else 'storage-read-held') == 1
    if regrant:
        scan(lambda _w, _d: 'storage-grant-restored' in log()[start:], 'launcher grant restored')
        scan(lambda _w, _d: 'Cathedral matrix: launch refused' in log()[start:], 'old launch refused')
        events = log()[start:]
        assert not re.search(r'Cathedral matrix: launch [0-4] ', events), events
        assert events.index('storage-grant-revoked') < events.index('storage-delay-released')
        n.background(0)
        n.launch(0, 0)
    if invalid:
        scan(lambda _w, _d: 'storage-invalid-image' in log()[start:], 'invalid executable returned')
        # Wait for the original picker to retire and its replacement to stand.
        scan(lambda _w, _d: 'local picker restored' in log()[start:], 'replacement picker restored')
        n.label(left + 28, top + 76, 'USER 1')
        click(left + 64, top + 48)
        n.desktop(1)
        n.launch(1, 0)
        n.note(1, ' ')
        n.app_close(1)
        n.background(1)
        n.task_close(0)
        n.background(0)
    else:
        value = 'a' if reboot else ''
        n.note(0, value + ' ')
        n.focus_note(0)
        key('b' if reboot else 'a')
        value += 'b' if reboot else 'a'
        n.note(0, value + ' ', dirty=True)
        key('f1')
        n.note(0, value + ' ')
        n.app_close(0)
        n.background(0)
        n.launch(0, 0)
        n.note(0, value + ' ')
        n.app_close(0)
        n.background(0)
    assert not omega_desktop_test.failed_children(log()), log()[-4000:]
    print('PASS: ' + ('binding refresh' if binding else 'executable read') + ' yielded; ' + ('admission recovery' if invalid else
          'regrant refused old launch, new launch saved and cold reopened' if regrant else
          'save, slot reuse and cold reopen'))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group()
    group.add_argument('--invalid', action='store_true', help='return a malformed executable to a nested login')
    group.add_argument('--regrant', action='store_true', help='revoke the launch scope during its read, then grant it anew')
    parser.add_argument('--binding', action='store_true', help='hold the launch binding query instead of the executable read')
    args = parser.parse_args()
    if args.binding and args.invalid:
        parser.error('--binding does not supply an invalid executable')
    omega_parent_poll_test.main(user_check=partial(exercise, invalid=args.invalid, regrant=args.regrant, binding=args.binding), label='omega-launch')
