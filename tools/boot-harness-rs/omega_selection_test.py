"""The stock Omega matrix switches existing apps while its parent holds COMMIT.
The old visible app must receive input, and a refusal must preserve both apps.
"""
import omega_parent_poll_test
import omega_desktop_test
from nested_desktop_test import Nested


def exercise(output, click, key, scan, width, height, reboot):
    n = Nested(click, key, scan, width, height)
    log = lambda: (output / 'serial.log').read_text(encoding='utf-8', errors='replace')

    def select(index):
        click(316 + 144 * index, height - 48)

    n.launch(0, 1)
    n.color(0, False)
    n.launch(0, 0)
    base = 'abc' if reboot else ''
    n.note(0, base + ' ')
    n.focus_note(0)
    key('d' if reboot else 'a')
    before = base + ('d' if reboot else 'a')
    n.note(0, before + ' ', True)

    def held(value, incoming, refuse=False):
        start = len(log())
        key('f3')
        scan(lambda _w, _d: 'interaction-delay-armed' in log()[start:], 'selection gate armed')
        select(0)
        scan(lambda _w, _d: 'interaction-delay-held' in log()[start:], 'selection commit held')
        n.note(0, value + ' ', True)
        n.focus_note(0)
        key(incoming)
        if refuse:
            key('f3')
            scan(lambda _w, _d: 'commit-refusal-armed' in log()[start:], 'selection refusal armed')
        for marker in ('commit-health-during-hold', 'commit-poll-during-hold'):
            scan(lambda _w, _d: marker in log()[start:], marker)
            events = log()[start:]
            end = 'commit-refused' if refuse else 'interaction-delay-released'
            if end in events:
                assert events.index(marker) < events.index(end)
        scan(lambda _w, _d: end in log()[start:], 'selection released')
        if refuse:
            scan(lambda _w, _d: 'commit-abort-received' in log()[start:], 'selection abort matched')
        else:
            n.color(0, False)
            select(1)
        # The key typed against the old visible app survives exactly once.
        n.note(0, value + incoming + ' ', True)
        assert 'matrix-reclaimed' not in log()[start:], 'selection must not replace either app'
        return value + incoming

    before = held(before, 'e' if reboot else 'b')
    before = held(before, 'f' if reboot else 'c', refuse=True)
    for _ in range(2):
        select(0)
        n.color(0, False)
        select(1)
        n.note(0, before + ' ', True)
    assert log().count('note-loaded') == 1
    n.focus_note(0)
    key('f1')
    n.note(0, before + ' ')
    n.app_close(0)
    n.color(0, False)
    n.app_close(0)
    n.background(0)
    assert not omega_desktop_test.failed_children(log()), log()[-4000:]
    print('PASS: held Omega selection serves HEALTH/input; old app gets exact keys; refusal preserves both apps; repeated switching and cold save')


if __name__ == '__main__':
    omega_parent_poll_test.main(user_check=exercise, label='omega-selection')
