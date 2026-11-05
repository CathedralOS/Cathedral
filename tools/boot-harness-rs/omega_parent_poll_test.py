"""The real Omega stock matrix must answer its owner while a Rust parent holds
its input poll. No Omega fixture implementation or compile-time flag substitutes
for the stock child. The parent asks HEALTH on the child's established owner
connection and fails before releasing the poll if it cannot get an answer.
"""
import json
from datetime import datetime, timezone

import desktop_test
import omega_desktop_test
from nested_desktop_test import Nested


def exercise(output, click, key, scan, width, height, reboot):
    n = Nested(click, key, scan, width, height)
    log = lambda: (output / 'serial.log').read_text(errors='replace')
    n.launch(0, 0)
    value = 'a' if reboot else ''
    n.note(0, value + ' ')  # Check the next blank cell too, not merely a prefix.
    n.focus_note(0)
    start = len(log())
    key('f8')
    scan(lambda _w, _d: 'interaction-delay-held' in log()[start:], 'parent holding Omega poll')
    key('b' if reboot else 'a')  # Queue a real input while its recipient's poll waits.
    scan(lambda _w, _d: 'poll-health-during-hold' in log()[start:],
         'Omega owner HEALTH answered while parent poll is held')
    events = log()[start:]
    if 'interaction-delay-released' in events:
        assert events.index('poll-health-during-hold') < events.index('interaction-delay-released')
    scan(lambda _w, _d: 'interaction-delay-released' in log()[start:], 'poll released')
    n.note(0, value + ('b' if reboot else 'a') + ' ', dirty=True)
    key('f1')
    n.note(0, value + ('b' if reboot else 'a') + ' ')
    n.app_close(0)
    n.background(0)
    assert not omega_desktop_test.failed_children(log()), log()[-4000:]
    print('PASS: stock Omega matrix answered HEALTH during held parent poll; queued key retained once')


def main(user_check=exercise, label='omega-parent-poll', fixture_features=(), matrix_entry=None):
    output = omega_desktop_test.harness.BUILD / label / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    profile = omega_desktop_test.compose(output)
    composition = json.loads(profile.read_text(encoding='utf-8'))
    stock = json.loads((omega_desktop_test.ROOT / 'source-rs/distribution/profile.json').read_text(encoding='utf-8'))
    root = next(p for p in stock['startup']['launches'] if p.get('package') == 'cathedral-stock-matrix')
    root.setdefault('features', []).append('interaction-delay')
    root['features'].extend(fixture_features)
    for i, launch in enumerate(composition['startup']['launches']):
        if launch.get('entry') == 'distribution/matrix/native/main.omg':
            composition['startup']['launches'][i] = root
    if matrix_entry is not None:
        for program in composition['startup']['programs'] + composition['startup']['realm_programs']:
            if program.get('entry') == 'distribution/matrix/native/main.omg':
                program['entry'] = matrix_entry
    profile.write_bytes((json.dumps(composition, indent=2) + '\n').encode())
    desktop_test.run(240, trace_storage=True, source_profile=profile, user_check=user_check)


if __name__ == '__main__':
    main()
