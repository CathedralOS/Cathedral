"""Real nested ordinary launches: delayed read, owner cancellation and partial rollback."""
from nested_desktop_test import Nested


def configure(composition, mode):
    def enable(value):
        if isinstance(value, dict):
            if value.get('package') == 'cathedral-stock-matrix':
                value.setdefault('features', []).extend(['launch-delay', 'trace-storage'])
                if mode in ('bind', 'place'): value['features'].append('launch-' + mode + '-failure')
            for child in value.values(): enable(child)
        elif isinstance(value, list):
            for child in value: enable(child)
    enable(composition)
    if mode in ('stop', 'revoke'):
        composition['startup']['initial']['features'].append('launch-read-' + mode + '-probe')


def exercise(output, click, key, scan, width, height, reboot, mode):
    n = Nested(click, key, scan, width, height)
    def log(): return (output / 'serial.log').read_text(encoding='utf-8', errors='replace')
    n.enter(1, not reboot); n.launch(1, 0)
    base = 'ab' if reboot else ''
    n.note(1, base); n.focus_note(1); key('a'); n.note(1, base+'a', True)
    before = len(log())
    n.launch(1, 1)
    scan(lambda sw,d: 'launch-read-held' in log()[before:], 'ordinary launch read held')
    n.focus_note(1); key('b'); n.note(1, base+'ab', True)
    assert 'launch-read-released' not in log()[before:], 'input waited for launch read'
    print('PASS: sibling input during ordinary launch read', flush=True)
    scan(lambda sw,d: 'launch-read-released' in log()[before:], 'ordinary launch read released')
    if mode == 'complete':
        n.color(1, False); n.app_close(1)
    else:
        scan(lambda sw,d: 'stored-launch-refused' in log()[before:], 'cancelled or rolled back launch')
        if mode in ('stop', 'revoke'):
            scan(lambda sw,d: 'read cancellation acknowledged' in log()[before:], 'routed owner cancellation')
            transcript=log()[before:]
            assert transcript.index('read cancellation requested') < transcript.index('launch-read-released')
            assert 'stored-launch-admitted 2 ' not in transcript
            assert 'stored-launch-published 2 ' not in transcript
        else:
            assert 'launch-stage-failure' in log()[before:]
            assert 'stored-launch-admitted 2 ' in log()[before:]
            assert 'matrix-reclaimed 2 ' in log()[before:], 'admitted task was not reclaimed'
            assert 'stored-launch-published 2 ' not in log()[before:]
        n.note(1, base+'ab', True)
        if mode != 'stop':
            # A fresh operation must succeed in the same slot after refusal/rollback.
            n.launch(1, 1); n.color(1, False); n.app_close(1)
    n.note(1, base+'ab', True)
    assert log().count('note-loaded') == 1
    x,y,w,h = n.viewport(1)
    n.app_close(1); n.label(x+18,y+28,'UNSAVED NOTE')
    click(x+38,y+84); n.note(1, base+'ab', True)
    n.focus_note(1); key('f1'); n.note(1, base+'ab')
    n.app_close(1)
    n.task_close(0); n.background(0)
    print(f'PASS: ordinary launch {mode}, protected draft, save and subtree reclamation', flush=True)
