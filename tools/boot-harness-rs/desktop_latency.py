"""The F5 desktop at depths 0..3: launch phases and visible input/save completion.

Run through desktop_test.py --latency. Each boot uses the same workload; no
latency threshold depends on host load. Screenshots and QMP add measurement cost.
"""
from pathlib import Path
import shutil
import capture


def exercise(stream, output, click, scan, width, height, reboot, timings):
    font = bytes.fromhex((Path(__file__).resolve().parents[2] / 'source-rs/platform/libraries/bitmap-font/font.hex').read_text())

    def pixel(w, data, x, y):
        return data[(y*w+x)*3:(y*w+x+1)*3]

    def text(w, data, x, y, value):
        return all((pixel(w, data, x+i*12+c*2, y+r*2) == b'\xff\xff\xff') == bool(bits & (1 << (4-c)))
                   for i, ch in enumerate(value)
                   for r, bits in enumerate(font[(ord(ch)-32)*7:(ord(ch)-31)*7]) for c in range(5))

    def key(code):
        # No artificial key dwell: observation still includes QMP and capture.
        capture.command(stream, 'input-send-event', {'events': [
            {'type': 'key', 'data': {'down': down, 'key': {'type': 'qcode', 'data': code}}}
            for down in (True, False)]})

    for depth in range(4):
        # Every nested user matrix has the same 8px inset and 80px taskbar.
        x = y = 8*depth
        w, h = width-16*depth, height-104*depth
        if depth:
            parent = depth-1
            top = y+(h-96)//2
            left = x+(w-(280 if reboot else 128))//2
            with timings.step(f'depth-{depth}-launch-picker'):
                click(8*parent+40+56*(3 if depth == 1 else 2), 8*parent+height-104*parent-48)
                scan(lambda sw, d: text(sw, d, left+(28 if reboot else 16), top+76,
                                       'USER 1' if reboot else 'NEW USER'), 'nested picker')
            if not reboot:
                with timings.step(f'depth-{depth}-create-user'):
                    click(left+64, top+48)
                    left = x+(w-280)//2
                    scan(lambda sw, d: text(sw, d, left+28, top+76, 'USER 1'), 'nested user')
            with timings.step(f'depth-{depth}-enter-user'):
                click(left+64, top+48)
                scan(lambda sw, d: pixel(sw, d, x+148, y+h-48) == bytes.fromhex('c7a2ed')
                     and text(sw, d, x+w-150, y+h-32, 'SETTINGS'), 'nested taskbar')

        def note(value, dirty=False):
            scan(lambda sw, d: pixel(sw, d, x+13, y+18) == bytes.fromhex('9b7028' if dirty else '27664a')
                 and text(sw, d, x+18, y+28, value+' '), f'depth {depth} note {value!r}, dirty={dirty}')

        with timings.step(f'depth-{depth}-launch-notes'):
            click(x+40, y+h-48)
            note('xyz' if reboot else '')
        click(x+13, y+120)
        for offset in (240, 280):
            px, py = x+offset, y+160
            with timings.step(f'depth-{depth}-pointer-{offset}'):
                capture.command(stream, 'input-send-event', {'events': [
                    {'type': 'abs', 'data': {'axis': 'x', 'value': round(px*32767/(width-1))}},
                    {'type': 'abs', 'data': {'axis': 'y', 'value': round(py*32767/(height-1))}}]})
                scan(lambda sw, d: pixel(sw, d, px, py) == b'\xff\xff\xff', 'root pointer tip')
        # Reproduce the same edits after a cold reopen, including a durable save.
        if reboot:
            value = 'xyz'
            for _ in range(3):
                key('backspace')
                value = value[:-1]
                note(value, True)
        value = ''
        for char in ('uvw' if reboot else 'xyz'):
            with timings.step(f'depth-{depth}-type-{char}'):
                key(char)
                value += char
                note(value, True)
        with timings.step(f'depth-{depth}-save'):
            key('f1')
            note(value)
        shutil.copyfile(output/'display.png', output/f'boot-{2 if reboot else 1}-depth-{depth}.png')
        with timings.step(f'depth-{depth}-close-notes'):
            click(x+w-22, y+18)
            scan(lambda sw, d: pixel(sw, d, x+13, y+18) == bytes.fromhex('101827')
                 and pixel(sw, d, x+249, y+h-60) == bytes.fromhex('101827'), 'closed and collected Notes')
