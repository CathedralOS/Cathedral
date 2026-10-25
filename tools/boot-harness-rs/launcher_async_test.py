"""Normal nested desktop with a held recovery read; only disposable builds add probes."""
import time
from nested_desktop_test import Nested


def configure(composition, mode):
    def enable(value):
        if isinstance(value, dict):
            if value.get('package') == 'cathedral-stock-matrix':
                value.setdefault('features', []).append('recovery-delay')
            if value.get('path') == 'Nested/Users/Template/Desktop':
                value.setdefault('features', []).append('recovery-probes')
            for child in value.values(): enable(child)
        elif isinstance(value, list):
            for child in value: enable(child)
    enable(composition)
    if mode != 'recover':
        composition['startup']['initial']['features'].append('launcher-read-' + mode + '-probe')


def exercise(output, click, key, scan, width, height, reboot, mode):
    n = Nested(click, key, scan, width, height)
    def log(): return (output / 'serial.log').read_text(encoding='utf-8', errors='replace')
    n.enter(1, not reboot)
    n.launch(1, 0)
    base = 'ab' if reboot else ''
    n.note(1, base)
    n.focus_note(1); key('a'); n.note(1, base + 'a', True)
    x,y,w,h = n.viewport(1)
    click(x+232, y+h-48); key('f8')
    scan(lambda sw,d: 'launcher-read-held' in log(), 'recovery read started')
    n.focus_note(1); key('b'); n.note(1, base + 'ab', True)
    assert 'launcher-read-released' not in log(), 'input did not progress during the held read'
    print('PASS: sibling input consumed while the nested recovery read is pending', flush=True)
    if mode == 'recover':
        scan(lambda sw,d: log().count('launcher recovered; retry consumed') == 1, 'ticketed completion admitted once')
        n.desktop(1)
    else:
        scan(lambda sw,d: 'read cancellation acknowledged' in log(), 'owner cancellation acknowledged')
        transcript = log()
        assert transcript.index('read cancellation requested') < transcript.index('launcher-read-released')
        assert 'launcher recovered; retry consumed' not in transcript
        assert 'launcher fallback admitted' not in transcript
    n.note(1, base + 'ab', True)
    assert log().count('note-loaded') == 1, 'sibling restarted during recovery'
    n.app_close(1)
    n.label(x+18, y+28, 'UNSAVED NOTE')
    click(x+38,y+84)  # Cancel the app-owned close dialog.
    n.note(1, base+'ab', True)
    n.focus_note(1); key('f1'); n.note(1, base+'ab')
    time.sleep(1)
    assert log().count('launcher-read-held') == 1
    assert log().count('launcher-read-released') == 1
    assert log().count('launcher recovered; retry consumed') == (1 if mode == 'recover' else 0)
    n.app_close(1)
    # Clean subtree close reclaims the recovered or cancelled launcher's resources.
    n.task_close(0); n.background(0)
    print(f'PASS: nested recovery {mode}, protected sibling draft, save and subtree reclamation', flush=True)
