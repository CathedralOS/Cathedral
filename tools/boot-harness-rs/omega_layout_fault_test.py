"""Invalidate a held layout through controller STOP or an ancestor view regrant."""
import argparse
import omega_parent_poll_test
import omega_desktop_test
from nested_desktop_test import Nested


def exercise(output, click, key, scan, width, height, reboot, mode="controller"):
    n = Nested(click, key, scan, width, height)
    log = lambda: (output / 'serial.log').read_text(encoding='utf-8', errors='replace')
    margin = 24 if reboot else 8
    standing_margin = margin
    chrome = margin
    left = (width - 512) // 2

    def note(value, dirty=True):
        scan(lambda w, d: n.pixel(w, d, margin + 5, margin + 10)
             == bytes.fromhex('9b7028' if dirty else '27664a')
             and n.text(w, d, margin + 10, margin + 20, value + ' '),
             f'Notes at margin {margin}: {value!r}, dirty={dirty}')

    def settings():
        click(width - chrome - 100, height - chrome - 16)
        n.label(left + 4, height - chrome - 70, 'BAR HEIGHT')

    def action(row, right=False):
        click(left + row * 176 + (120 if right else 40), height - chrome - 20)

    def focus():
        click(margin + 5, margin + 130)

    click(margin + 32, height - margin - 40)
    value = 'ac' if reboot else ''
    note(value, False)
    focus()
    key('b' if reboot else 'a')
    value += 'b' if reboot else 'a'
    note(value)
    settings()
    action(1, not reboot)  # Save a different placement, but cancel its live application.
    start = len(log())
    events = lambda: log()[start:]
    key('f3')
    scan(lambda _w, _d: 'interaction-delay-armed' in events(), 'layout gate armed')
    action(2)
    scan(lambda _w, _d: 'interaction-delay-held' in events(), 'layout held')
    note(value)
    focus()
    if mode == 'controller':
        key('d' if reboot else 'c')
        value += 'd' if reboot else 'c'
    for marker in ('commit-health-during-hold', 'commit-poll-during-hold'):
        scan(lambda _w, _d: marker in events(), marker)
    key('f1' if mode == 'controller' else 'f8')
    marker = 'selection-stop-requested' if mode == 'controller' else 'layout-view-regranted'
    scan(lambda _w, _d: marker in events(), marker)
    assert 'interaction-delay-released' not in events()
    settled = 'layout-controller-replaced' if mode == 'controller' else 'layout-view-restored'
    scan(lambda _w, _d: settled in events(), settled)
    if mode == 'controller':
        assert events().index('interaction-delay-released') < events().index('selection-stop-acknowledged')
        assert events().index('matrix: reclaimed 0 ') < events().index(settled)
    else:
        assert events().index('layout-view-regranted') < events().index('interaction-delay-released')
        margin = 8  # Explicit owner placements use the standard geometry.
        n.label(width - 8 - 184, height - 8 - 48, 'SAVED; RESTART')
        note(value)
        focus()
        key('d' if reboot else 'c')
        value += 'd' if reboot else 'c'
    note(value)
    assert log().count('note-loaded') == 1

    # Explicit placement gives an 8px/80px bar, with no new layout grant.
    # Only the controller case replaces the program; ancestor keeps its editor.
    chrome = 8
    if mode == 'controller':
        scan(lambda w, d: n.pixel(w, d, 32, height - 56) == bytes.fromhex('d6e5ef'), 'replacement bar')
        settings()
    action(2)
    n.label(width - chrome - 184, height - chrome - 48, 'SAVED; RESTART')
    note(value)
    action(2, True)
    click(88, height - 48)
    scan(lambda w, d: n.pixel(w, d, standing_margin + 5, standing_margin + 10) == bytes.fromhex('315ca8'), 'Colors uses unchanged standing launch placement')
    click(316, height - 48)
    note(value)
    assert log().count('note-loaded') == 1, 'grant invalidation restarted Notes'
    focus()
    key('f1')
    note(value, False)
    click(width - margin - 14, margin + 10)
    scan(lambda w, d: n.pixel(w, d, standing_margin + 5, standing_margin + 10) == bytes.fromhex('315ca8'), 'Notes closes to Colors')
    click(width - standing_margin - 14, standing_margin + 10)
    n.background(0)
    assert not omega_desktop_test.failed_children(log()), log()[-4000:]
    print(f'PASS: held layout {mode}; no inherited layout authority; draft, new requests and saved settings survive', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--mode', choices=('controller', 'ancestor'), default='controller')
    args = parser.parse_args()
    omega_parent_poll_test.main(user_check=lambda *values: exercise(*values, mode=args.mode),
                               label='omega-layout-' + args.mode,
                               user_layouts=((8, 80), (24, 80)))
