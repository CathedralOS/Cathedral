"""Physical typing workload; every expected glyph is checked across cold reboot."""
from pathlib import Path
import time
import capture
import pointer_test

FULL = 'hello cathedral notes abcdefghijklmnopqrstuvwxyz 0123456789abc'[:60]
FINAL = FULL[:35]
SIBLING = 'private sibling note 2'


def desktop(stream, output, serial, reboot, recovery=False):
    position = [40, 40]
    font = bytes.fromhex((Path(__file__).resolve().parents[2] / 'source-rs/platform/libraries/bitmap-font/font.hex').read_text())

    def send(events):
        capture.command(stream, 'input-send-event', {'events': events})

    def key(code, count=1):
        for _ in range(count):
            for down in (True, False):
                send([{'type': 'key', 'data': {'down': down, 'key': {'type': 'qcode', 'data': code}}}])
                time.sleep(0.06)

    def type_text(text):
        for char in text:
            key('spc' if char == ' ' else char)

    def click(x, y):
        while position != [x, y]:
            dx = max(-70, min(70, x - position[0]))
            dy = max(-70, min(70, y - position[1]))
            send([{'type': 'rel', 'data': {'axis': 'x', 'value': dx}},
                  {'type': 'rel', 'data': {'axis': 'y', 'value': dy}}])
            position[0] += dx
            position[1] += dy
            time.sleep(0.04)
        for down in (True, False):
            send([{'type': 'btn', 'data': {'button': 'left', 'down': down}}])
            time.sleep(0.08)

    def check(text, nested=True, dirty=False, cursor=None, error=False):
        x0, y0 = (144, 200) if nested else (510, 200)
        cursor = len(text) if cursor is None else cursor
        # This workload contains no newlines; row scrolling follows the cursor.
        top = max(0, cursor // 15 - 3)
        expected = bytearray(bytes.fromhex('182132') * (180 * 64))
        def pixel(x, y, color):
            expected[(y * 180 + x) * 3:(y * 180 + x) * 3 + 3] = bytes.fromhex(color)
        for index, char in enumerate(text):
            row = index // 15 - top
            if not 0 <= row < 4: continue
            for y, bits in enumerate(font[(ord(char) - 32) * 7:(ord(char) - 31) * 7]):
                for x in range(5):
                    if bits & (1 << (4 - x)):
                        for dy in range(2):
                            for dx in range(2): pixel(index % 15 * 12 + x * 2 + dx, row * 16 + y * 2 + dy, 'ffffff')
        for dx in range(10): pixel(cursor % 15 * 12 + dx, (cursor // 15 - top) * 16 + 14, '66ccff')
        deadline = time.monotonic() + 15
        while True:
            serial = (output / 'serial.log').read_text(errors='replace')
            assert 'user panic:' not in serial and 'CATHEDRAL_RS_PANIC' not in serial, serial[-1500:]
            width, height, data = pointer_test.read_scanout(stream, output)
            actual = b''.join(data[((y0 + y) * width + x0) * 3:((y0 + y) * width + x0 + 180) * 3] for y in range(64))
            bx, by = (140, 190) if nested else (505, 190)
            border = data[(by * width + bx) * 3:(by * width + bx) * 3 + 3]
            if actual == expected and border == bytes.fromhex('993f46' if error else '9b7028' if dirty else '27664a'):
                capture.save_png(output, width, height, data)
                return
            if time.monotonic() >= deadline:
                capture.save_png(output, width, height, data)
                raise AssertionError(f'Notes text/status differs: nested={nested}, text={text!r}; {output}')
            time.sleep(0.05)

    if reboot:
        check(FINAL)
        check(SIBLING, nested=False)
        return
    check('')
    check('', nested=False)
    click(140, 285)  # Inside note, below text: cursor overlay cannot obscure glyph checks.
    type_text('hello')
    check('hello', dirty=True)
    key('f1')
    if recovery:
        check('hello', dirty=True, error=True)
        deadline = time.monotonic() + 20
        while 'storage replaced; explicit grants restored' not in (output / 'serial.log').read_text(errors='replace'):
            assert time.monotonic() < deadline, 'watchdog did not restore storage'
            time.sleep(0.05)
        key('f1')  # Reconcile the lost acknowledgement, preserving the draft.
    check('hello')
    key('left', 2)
    key('backspace')
    type_text('l')
    key('right', 2)
    check('hello')  # Middle editing restored confirmed text without a disk write.
    type_text(' cathedral notes')
    key('f1')
    check('hello cathedral notes')
    type_text(FULL[len('hello cathedral notes'):])
    key('f1')
    check(FULL)
    key('backspace', 25)
    key('f1')
    check(FINAL)
    key('f1')  # Clean save must not create an extra revision.
    type_text('draft')
    check(FINAL + 'draft', dirty=True)
    key('f2')
    check(FINAL)
    click(505, 285)
    type_text('private sibling note')
    key('f1')
    if recovery:
        # Its old scope was revoked too. Reconciliation preserves this unsaved draft;
        # a subsequent deliberate save uses the newly granted scope and revision.
        check('private sibling note', nested=False, dirty=True, error=True)
        key('f1')
        check('private sibling note', nested=False, dirty=True)
        key('f1')
    check('private sibling note', nested=False)
    type_text(' 2')
    key('f1')
    check(SIBLING, nested=False)
    click(80, 520)
    type_text('x')
    key('f1')
    check(FINAL)
    check(SIBLING, nested=False)
    print('PASS: physical notes input, capacity, draft/reload, focus isolation and exact glyphs')
