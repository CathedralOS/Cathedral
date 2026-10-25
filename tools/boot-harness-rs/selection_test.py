"""Select existing apps through F5; hold visual commit without stopping input."""
from nested_desktop_test import Nested


def configure(composition, mode):
    def enable(value):
        if isinstance(value, dict):
            if value.get('package') == 'cathedral-stock-matrix':
                value.setdefault('features', []).extend(['selection-delay', 'trace-storage'])
                if mode == 'failure': value['features'].append('scene-failure')
                if mode == 'late-stop': value['features'].append('scene-commit-hold')
            for child in value.values(): enable(child)
        elif isinstance(value, list):
            for child in value: enable(child)
    enable(composition)
    if mode in ('stop', 'late-stop'):
        composition['startup']['initial']['features'].extend(['selection-stop-probe', 'launch-pending-loss-probe'])


def exercise(output, click, key, scan, width, height, reboot, mode):
    n = Nested(click, key, scan, width, height)
    def log(): return (output / 'serial.log').read_text(encoding='utf-8', errors='replace')
    def select(index):
        x,y,w,h = n.viewport(1)
        click(x+260+144*index, y+h-48)
    n.enter(1, not reboot)
    n.launch(1, 1); n.color(1, False)  # Colors owns slot 1.
    n.launch(1, 0)
    base = 'ab' if reboot else ''
    n.note(1, base); n.focus_note(1); key('a'); n.note(1, base+'a', True)
    before = len(log())
    select(0)
    scan(lambda sw,d: 'launch-scene-held' in log()[before:], 'existing-app switch held')
    n.focus_note(1); key('b')
    scan(lambda sw,d: any(line.endswith(' 16') for line in log()[before:].splitlines() if 'launch-scene-input' in line), 'B routed during switch')
    assert 'launch-scene-released' not in log()[before:]
    scan(lambda sw,d: 'selection-complete' in log()[before:], 'switch completed')
    transcript = log()[before:]
    if mode == 'complete':
        assert any(line.endswith(' 0') for line in transcript.splitlines() if 'selection-complete' in line)
        n.color(1, False); select(1)
    else:
        status = (1 << 64) - (5 if mode == 'failure' else 13)
        assert any(line.endswith(f' {status}') for line in transcript.splitlines() if 'selection-complete' in line)
        if mode in ('stop', 'late-stop'):
            scan(lambda sw,d: 'read cancellation acknowledged' in log()[before:], 'owner stop acknowledged')
            transcript = log()[before:]
            assert 'pending launch loss busy' in transcript
            assert transcript.index('read cancellation requested') < transcript.index('launch-scene-released')
            assert 'matrix-reclaimed 1 ' in transcript
            assert ('selection-restored' in transcript) == (mode == 'late-stop')
        else:
            assert 'matrix-reclaimed' not in transcript, 'failed selection must not kill either app'
    n.note(1, base+'ab', True)
    if mode != 'complete':
        # Same surviving task ticket, or a new task in the explicitly stopped slot.
        if mode == 'failure': select(0)
        else: n.launch(1, 1)
        n.color(1, False); select(1); n.note(1, base+'ab', True)
    # Repeated switches retain cached image backing and independent app state.
    select(0); n.color(1, False); select(1); n.note(1, base+'ab', True)
    assert log().count('note-loaded') == 1
    n.app_close(1)
    x,y,w,h = n.viewport(1)
    n.label(x+18,y+28,'UNSAVED NOTE'); click(x+38,y+84)
    n.note(1, base+'ab', True); n.focus_note(1); key('f1'); n.note(1, base+'ab')
    n.app_close(1); n.color(1, False); n.app_close(1)
    n.task_close(0); n.background(0)
    print(f'PASS: existing-app selection {mode}, routed input, repeated switches, protected draft and reclamation', flush=True)
