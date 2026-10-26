"""Real stock owner control: old clean LOSS must not authorize a later dirty close."""
from nested_desktop_test import Nested


def configure(value):
    if isinstance(value, dict):
        if value.get('package') == 'cathedral-notes':
            value.setdefault('features', []).append('control-delay-probe')
        for child in value.values():
            configure(child)
    elif isinstance(value, list):
        for child in value:
            configure(child)


def exercise(output, click, key, scan, width, height, reboot):
    n = Nested(click, key, scan, width, height)
    log = lambda: (output / 'serial.log').read_text(errors='replace')
    def observed(marker, offset=0):
        scan(lambda _w, _d: marker in log()[offset:], marker)

    n.launch(0, 0)
    value = 'x' if reboot else ''
    n.note(0, value)
    scan(lambda _w, _d: log().count('Cathedral matrix: shared child control ready') >= 5,
         'shared Notes owner control')
    start = len(log())
    n.task_close(0)
    observed('Cathedral control probe: holding original loss answer', start)
    observed('matrix-close-refused', start)  # Caller expired; child remains alive.
    n.focus_note(0)
    key('y' if reboot else 'x')
    value += 'y' if reboot else 'x'
    n.note(0, value, dirty=True)
    assert 'released original loss answer' not in log()[start:], 'fixture gate released before input proof'
    second = len(log())
    n.task_close(0)  # An uncertain outstanding request cannot be replaced/replayed.
    n.note(0, value, dirty=True)
    assert 'released original loss answer' not in log()[start:], 'second close missed the outstanding-request window'
    observed('Cathedral control probe: released original loss answer', second)
    observed('matrix-control-late-drained 0 0', second)
    latest = len(log())
    n.task_close(0)  # Fresh LOSS must observe the new dirty state.
    observed('matrix-close-refused 1 0 1', latest)
    n.note(0, value, dirty=True)
    n.focus_note(0)
    key('f1')
    n.note(0, value)
    n.task_close(0)
    n.background(0)
    # The separate UI presenter is also an ordinary shared-control receiver.
    presenter = len(log())
    n.launch(0, 4)
    observed('Omega UI surface ready', presenter)
    observed('Cathedral matrix: shared child control ready', presenter)
    n.task_close(0)
    n.background(0)
    print('PASS: shared control expiry, live editing, old clean reply drained, fresh dirty refusal and clean close')


if __name__ == '__main__':
    import desktop_test
    desktop_test.run(180, trace_storage=True, control_late=True)
