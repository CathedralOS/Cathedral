"""Stall the real compositor beneath the unmodified stock Omega root/user.
The dedicated test owner asks HEALTH; the physical input adapter records replies.
"""
from datetime import datetime, timezone
import json
import desktop_test
import omega_desktop_test
from nested_desktop_test import Nested


def exercise(output, click, key, scan, width, height, reboot):
    n = Nested(click, key, scan, width, height)
    log = lambda: (output / 'serial.log').read_text(encoding='utf-8', errors='replace')

    def arm(x):
        start = len(log())
        click(x, 3)
        scan(lambda _w, _d: 'display-armed' in log()[start:], 'compositor armed')
        return start

    def responsive(start, refused=False):
        scan(lambda _w, _d: 'display-held' in log()[start:], 'compositor held')
        start += log()[start:].index('display-held')
        # Fresh motion and a key need acknowledgements while drawing is stalled.
        click(13, 144)
        key('b' if not reboot else 'd')
        # Physical B/D press, not an acknowledgement left over from arming.
        fresh_key = f'input-ack 104 {18 if reboot else 16} 1'
        for marker in ('root-health', fresh_key):
            scan(lambda _w, _d: marker in log()[start:], marker + ' during display hold')
            events = log()[start:]
            end = 'display-refused' if refused else 'display-released'
            if end in events:
                assert events.index(marker) < events.index(end)
        scan(lambda _w, _d: end in log()[start:], end)
        if refused:
            scan(lambda _w, _d: 'display-aborted' in log()[start:], 'matching compositor abort')

    n.launch(0, 0)
    before = 'ab' if reboot else ''
    n.note(0, before + ' ')
    start = arm(3)
    n.focus_note(0)
    key('c' if reboot else 'a')
    responsive(start)
    expected = before + ('cd' if reboot else 'ab') + ' '
    n.note(0, expected, dirty=True)
    key('f1')
    n.note(0, expected)
    n.app_close(0)
    n.background(0)

    n.launch(0, 1)
    saved = reboot
    n.color(0, saved)
    for position in (4, 5, 6):
        start = arm(position)
        click(13, 144)
        key('ret')
        scan(lambda _w, _d: 'display-held' in log()[start:], 'color publication held')
        n.color(0, saved)
        responsive(start, position == 6)
        saved = not saved
        if position == 6:
            n.background(0)
            n.launch(0, 1)
        n.color(0, saved)
    n.app_close(0)
    n.background(0)
    start = arm(7)
    click(13, 110)
    responsive(start)
    scan(lambda w, d: n.pixel(w, d, 13, 144) == bytes.fromhex('ffffff')
         and n.pixel(w, d, 13, 110) == bytes.fromhex('101827'),
         'cursor catches up to latest position after held pointer render')
    assert not omega_desktop_test.failed_children(log()), log()[-4000:]
    print('PASS: compositor image/commit/rectangle/pointer holds serve root HEALTH and hardware input; refusal aborts; exact Notes and Colors reopen')


def main():
    output = omega_desktop_test.harness.BUILD / 'omega-display-wait' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    profile = omega_desktop_test.compose(output)
    composition = json.loads(profile.read_text(encoding='utf-8'))
    composition['startup']['initial']['features'].append('display-wait-probe')
    for launch in composition['startup']['launches']:
        if launch.get('package') == 'cathedral-display-service':
            launch.setdefault('features', []).append('presentation-delay')
        if launch.get('package') == 'cathedral-input-service':
            launch.setdefault('features', []).append('display-wait-probe')
    profile.write_bytes((json.dumps(composition, indent=2) + '\n').encode())
    desktop_test.run(240, trace_storage=True, source_profile=profile, user_check=exercise)


if __name__ == '__main__':
    main()
