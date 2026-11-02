"""The installed Omega matrix stays responsive through COMMIT and aborts a
refused transaction before a subsequent app can publish on the same link."""
import omega_parent_poll_test
import omega_desktop_test
from nested_desktop_test import Nested


def exercise(output, click, key, scan, width, height, reboot):
    n = Nested(click, key, scan, width, height)
    log = lambda: (output / 'serial.log').read_text(errors='replace')
    n.launch(0, 1)
    n.color(0, False)
    click(13, 80)  # Select Colors without changing its saved color.

    def held(refuse=False):
        start = len(log())
        key('f3')
        scan(lambda _w, _d: 'interaction-delay-armed' in log()[start:], 'commit gate armed')
        key('ret')  # Save the other color, then publish its two rectangles.
        scan(lambda _w, _d: 'interaction-delay-held' in log()[start:], 'commit held')
        n.color(0, refuse)  # The previous complete scene remains visible.
        key('a')  # Fresh input must cause another parent poll during the wait.
        if refuse:
            key('f3')
            scan(lambda _w, _d: 'commit-refusal-armed' in log()[start:], 'commit refusal armed')
        for marker in ('commit-health-during-hold', 'commit-poll-during-hold'):
            scan(lambda _w, _d: marker in log()[start:], marker)
            events = log()[start:]
            end = 'commit-refused' if refuse else 'interaction-delay-released'
            if end in events:
                assert events.index(marker) < events.index(end)
        scan(lambda _w, _d: end in log()[start:], end)
        if refuse:
            scan(lambda _w, _d: 'commit-abort-received' in log()[start:], 'matching transaction aborted')

    held()
    n.color(0, True)
    held(refuse=True)
    # Colors intentionally exits on a rendering refusal in an unchanged view.
    # Its completed storage write survives; relaunch must use a fresh batch.
    n.background(0)
    n.launch(0, 1)
    n.color(0, False)
    n.app_close(0)
    n.background(0)
    assert not omega_desktop_test.failed_children(log()), log()[-4000:]
    print('PASS: held COMMIT serves HEALTH/input; refusal aborts, replacement publishes and saved state reopens')


if __name__ == '__main__':
    omega_parent_poll_test.main(user_check=exercise, label='omega-parent-batch')
