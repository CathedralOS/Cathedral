"""Hold an Omega login handoff after picker collection, before serving the user.

Owner HEALTH must complete during the hold. STOP and policy revocation must
still cancel the selected desktop and restore a usable picker. The Rust parent
routes those ordinary control requests through the outer Omega desktop.
"""
import argparse
from functools import partial

import omega_desktop_test
import omega_parent_poll_test
from nested_desktop_test import Nested


def exercise(output, click, key, scan, width, height, reboot, mode='health'):
    n = Nested(click, key, scan, width, height)
    log = lambda: (output / 'serial.log').read_text(encoding='utf-8', errors='replace')
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
    events = lambda: log()[start:]
    click(left + 64, top + 48)
    held = 'Matrix handoff probe: held'
    released = 'Matrix handoff probe: released'
    scan(lambda _w, _d: held in events(), 'picker collected; selected desktop withheld')
    key({'health': 'f8', 'stop': 'f7', 'regrant': 'f6'}[mode])
    scan(lambda _w, _d: 'admission-health' in events(), 'nested login answers HEALTH during handoff')
    # These two polls prove progress of the outer desktop's parent connection.
    # The routed HEALTH above proves progress of the nested login itself.
    key('left')
    scan(lambda _w, _d: 'admission-input-first-poll' in events(), 'first outer input poll')
    key('right')
    scan(lambda _w, _d: 'admission-input-polled' in events(), 'second outer input poll')
    scan(lambda _w, _d: 'admission-probe-complete' in events(), 'owner probe completes')
    text = events()
    assert text.index(held) < text.index('admission-health'), text
    assert released not in text or text.index('admission-health') < text.index(released), text
    if mode == 'health':
        scan(lambda _w, _d: released in events(), 'handoff released')
        text = events()
        assert 'configured from realm paths' not in text[:text.index(released)], text
    else:
        scan(lambda _w, _d: 'Cathedral matrix: launch refused' in events(), 'selected desktop refused')
        scan(lambda _w, _d: 'local picker restored' in events(), 'picker restored after cancelled selection')
        text = events()
        assert released not in text, text
        assert 'configured from realm paths' not in text, text
        assert 'Cathedral matrix: launch refused fffffffffffffff3' in text, text
        if mode == 'regrant':
            assert text.index('admission-policy-revoked') < text.index('admission-policy-restored'), text
        else:
            assert 'admission-stopped' in text, text
        n.label(left + 28, top + 76, 'USER 1')
        click(left + 64, top + 48)
    n.desktop(1)
    value = 'a' if reboot else ''
    n.launch(1, 0)
    n.note(1, value + ' ')
    n.focus_note(1)
    key('b' if reboot else 'a')
    value += 'b' if reboot else 'a'
    n.note(1, value + ' ', dirty=True)
    key('f1')
    n.note(1, value + ' ')
    n.app_close(1)
    n.background(1)
    n.launch(1, 0)
    n.note(1, value + ' ')
    n.app_close(1)
    n.background(1)
    n.task_close(0)
    n.background(0)
    assert not omega_desktop_test.failed_children(log()), log()[-4000:]
    print(f'PASS: login handoff {mode}; owner progress, withheld desktop, save and cold reopen', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--mode', choices=('health', 'stop', 'regrant'), default='health')
    args = parser.parse_args()
    omega_parent_poll_test.main(user_check=partial(exercise, mode=args.mode),
                               label='omega-handoff', fixture_features=['handoff-probe'],
                               matrix_entry='distribution/lab/matrix-handoff/native/main.omg')
