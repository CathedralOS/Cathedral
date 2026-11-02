"""Hold a real stock Omega matrix's forwarded image at its Rust parent.

The parent must receive HEALTH and a separate input poll before releasing the
image. Exact Notes text and a cold reopen check queued input and persistence.
"""
import omega_parent_poll_test
import omega_desktop_test
from nested_desktop_test import Nested


def exercise(output, click, key, scan, width, height, reboot):
    n = Nested(click, key, scan, width, height)
    log = lambda: (output / 'serial.log').read_text(errors='replace')
    n.launch(0, 0)
    before = 'ab' if reboot else ''
    n.note(0, before + ' ')
    n.focus_note(0)
    start = len(log())
    key('f5')
    scan(lambda _w, _d: 'interaction-delay-armed' in log()[start:], 'image gate armed')
    key('c' if reboot else 'a')
    scan(lambda _w, _d: 'interaction-delay-held' in log()[start:], 'parent holding Omega image')
    key('d' if reboot else 'b')
    for marker in ('image-health-during-hold', 'image-poll-during-hold'):
        scan(lambda _w, _d: marker in log()[start:], marker)
        events = log()[start:]
        if 'interaction-delay-released' in events:
            assert events.index(marker) < events.index('interaction-delay-released')
    scan(lambda _w, _d: 'interaction-delay-released' in log()[start:], 'image released')
    expected = before + ('cd' if reboot else 'ab') + ' '
    n.note(0, expected, dirty=True)
    # Hold the saved-state repaint as well. The taskbar can request a close
    # while Notes is still waiting for that image; it must eventually collect
    # the same app, without losing the completed save or reusing its scene early.
    start = len(log())
    key('f5')
    scan(lambda _w, _d: 'interaction-delay-armed' in log()[start:], 'saved image gate armed')
    key('f1')
    scan(lambda _w, _d: 'interaction-delay-held' in log()[start:], 'saved image held')
    n.task_close(0)
    scan(lambda _w, _d: 'image-health-during-hold' in log()[start:], 'HEALTH during saved image')
    scan(lambda _w, _d: 'interaction-delay-released' in log()[start:], 'saved image released')
    n.background(0)
    assert not omega_desktop_test.failed_children(log()), log()[-4000:]
    print('PASS: Omega HEALTH/input during held image; exact keys, close during held save and cold reopen')


if __name__ == '__main__':
    omega_parent_poll_test.main(user_check=exercise, label='omega-parent-image')
