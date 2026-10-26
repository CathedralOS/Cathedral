"""Cut real matrix owner control while keeping its descendants' input/storage live."""
from nested_desktop_test import Nested


def configure(value):
    if isinstance(value, dict):
        if value.get('package') == 'cathedral-stock-matrix':
            value.setdefault('features', []).append('control-failure-probe')
        for child in value.values():
            configure(child)
    elif isinstance(value, list):
        for child in value:
            configure(child)


def exercise(output, click, key, scan, width, height, reboot, mode):
    n = Nested(click, key, scan, width, height)
    log = lambda: (output / 'serial.log').read_text(errors='replace')
    marker = 'Cathedral control probe: owner link cut; child retained'
    before = len(log())
    if mode == 'matrix':
        n.enter(1, not reboot)
        scan(lambda _w, _d: marker in log()[before:], 'nested matrix owner link retired')
        n.launch(1, 0)
        n.note(1, 'x' if reboot else '')
        n.focus_note(1)
        key('y' if reboot else 'x')
        n.note(1, 'xy' if reboot else 'x', dirty=True)
        key('f1')
        n.note(1, 'xy' if reboot else 'x')
        n.launch(1, 1)
        n.color(1, False)
        # The outer owner cannot obtain LOSS through the cut connection, so it
        # must not interpret transport failure as death or consent to stop.
        before_close = len(log())
        n.task_close(0)
        scan(lambda _w, _d: 'matrix-close-unavailable' in log()[before_close:], 'uncertain matrix close refused')
        n.color(1, False)
        # Select the already-running Notes cell, then prove input still works.
        x, y, w, h = n.viewport(1)
        click(x+270, y+h-48)
        n.note(1, 'xy' if reboot else 'x')
        n.focus_note(1)
        key('z')
        n.note(1, 'xyz' if reboot else 'xz', dirty=True)
        key('backspace')
        key('f1')
        n.note(1, 'xy' if reboot else 'x')
    else:
        n.launch(0, 2)
        value = 'x' if reboot else ''
        def note(value, dirty=False):
            scan(lambda w,d: n.pixel(w,d,13,50)==bytes.fromhex('9b7028' if dirty else '27664a')
                 and n.text(w,d,18,60,value), f'Split live descendant {value!r}')
        note(value)
        scan(lambda _w, _d: marker in log()[before:], 'Split owner link retired')
        before_close = len(log())
        n.task_close(0)
        scan(lambda _w, _d: 'matrix-close-unavailable' in log()[before_close:], 'uncertain Split close refused')
        click(13, 150)
        key('y' if reboot else 'x')
        value += 'y' if reboot else 'x'
        note(value, True)
        key('f1')
        note(value)
        # Child-initiated close uses the independent interaction service.
        n.app_close(0)
        n.background(0)
    assert 'matrix-service-exit' not in log()[before:]
    print(f'PASS: {mode} retains descendants, input and durable storage after owner control loss')


if __name__ == '__main__':
    import argparse
    import desktop_test
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--mode', choices=['split', 'matrix'], required=True)
    args = parser.parse_args()
    desktop_test.run(180, trace_storage=True, control_failure=args.mode)
