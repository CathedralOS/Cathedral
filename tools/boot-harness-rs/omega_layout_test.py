"""Apply a live layout while its parent holds COMMIT: input and HEALTH continue."""
import argparse

import omega_desktop_test
import omega_parent_poll_test
from nested_desktop_test import Nested


def exercise(output, click, key, scan, width, height, reboot, mode="complete"):
    n = Nested(click, key, scan, width, height)
    log = lambda: (output / 'serial.log').read_text(encoding='utf-8', errors='replace')
    margin = 8
    left = (width - 512) // 2

    def note(value, dirty=False):
        scan(lambda w, d: n.pixel(w, d, margin + 5, margin + 10)
             == bytes.fromhex('9b7028' if dirty else '27664a')
             and n.text(w, d, margin + 10, margin + 20, value + ' '),
             f'Notes at margin {margin}: {value!r}')

    def settings():
        click(width - margin - 100, height - margin - 16)
        n.label(left + 4, height - margin - 70, 'BAR HEIGHT')

    def action(row, right=False):
        click(left + row * 176 + (120 if right else 40), height - margin - 20)

    def status(value):
        n.label(width - margin - 184, height - margin - 48, value)

    if mode == 'cancel':
        # The existing owner-control probe stops slot 1. Keep Notes in slot 2
        # so cancelling another captured placement must preserve its dirty draft.
        n.launch(0, 1)
        n.color(0, False)
    n.launch(0, 0)
    value = 'ac' if reboot else ''
    note(value)
    n.focus_note(0)
    key('b' if reboot else 'a')
    value += 'b' if reboot else 'a'
    note(value, True)
    settings()
    action(1, True)
    status('UNSAVED')

    start = len(log())
    events = lambda: log()[start:]
    key('f3')
    scan(lambda _w, _d: 'interaction-delay-armed' in events(), 'layout commit gate armed')
    action(2)  # Save settings, then request the live arrangement.
    scan(lambda _w, _d: 'interaction-delay-held' in events(), 'layout commit held')
    note(value, True)
    # This location belongs to Notes in the currently displayed arrangement,
    # but falls outside the proposed 24px placement. Keep routing to what is shown.
    n.focus_note(0)
    key('d' if reboot else 'c')
    value += 'd' if reboot else 'c'
    if mode == 'refuse':
        key('f3')
        scan(lambda _w, _d: 'commit-refusal-armed' in events(), 'layout refusal armed')
    elif mode == 'cancel':
        scan(lambda _w, _d: 'commit-health-during-hold' in events(), 'owner ready for cancellation probe')
        key('f6')
        scan(lambda _w, _d: 'selection-stop-requested' in events(), 'captured child stop requested')
        assert 'interaction-delay-released' not in events()
    end = 'commit-refused' if mode == 'refuse' else 'interaction-delay-released'
    for marker in ('commit-health-during-hold', 'commit-poll-during-hold'):
        scan(lambda _w, _d: marker in events(), marker)
        if end in events():
            assert events().index(marker) < events().index(end)
    scan(lambda _w, _d: end in events(), 'layout hold finished')
    if mode != 'complete':
        settled = 'commit-abort-received' if mode == 'refuse' else 'selection-stop-acknowledged'
        scan(lambda _w, _d: settled in events(), 'refused or cancelled layout settled')
        note(value, True)
        assert log().count('note-loaded') == 1, 'rollback must preserve the same draft'
        if mode == 'cancel':
            assert 'selection-loss-busy' in events()
            assert events().index('selection-stop-requested') < events().index(end)
        action(2)  # Saved configuration can be explicitly applied again.

    margin = 24
    status('APPLIED')
    note(value, True)
    assert log().count('note-loaded') == 1, 'layout must retain the app and draft'

    action(2, True)  # Close settings, then save and reopen at the new placement.
    click(margin + 5, margin + 130)
    key('f1')
    note(value)
    click(width - margin - 14, margin + 10)
    scan(lambda w, d: n.pixel(w, d, margin + 5, margin + 10)
         == bytes.fromhex('101827'), 'resized Notes closed')
    click(margin + 32, height - margin - 40)
    note(value)
    assert log().count('note-loaded') == 2, 'later launch must use the new standing placement'

    # Restore default geometry for the shared desktop harness's cold login.
    # The separate settings suite covers cold boot with a changed arrangement.
    settings()
    action(1)
    status('UNSAVED')
    action(2)
    margin = 8
    status('APPLIED')
    note(value)
    action(2, True)
    n.app_close(0)
    n.background(0)
    assert not omega_desktop_test.failed_children(log()), log()[-4000:]
    print(f'PASS: held layout {mode} serves owner/input, preserves draft and uses accepted placement for later launches', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--mode', choices=('complete', 'refuse', 'cancel'), default='complete')
    args = parser.parse_args()
    omega_parent_poll_test.main(user_check=lambda *values: exercise(*values, mode=args.mode),
                               label='omega-layout-' + args.mode)
