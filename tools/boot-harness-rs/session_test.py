"""Real device input, process recovery, attenuation and independent scanout oracle."""
from pathlib import Path
import time
import capture
import pointer_test
import session_latency


def verify(stream, output, serial):
    # Runtime writes split at page/ABI boundaries; each syscall adds this prefix,
    # including when the split falls inside a single guest evidence message.
    prefix = "Cathedral Rust lab: "
    serial = serial.replace(prefix, "")
    if "interactive startup; destructive probes disabled" not in serial:
        for marker in ("idle interval had zero", "real child scope clipping wrong-owner stale hosting and attenuated focus denials passed"):
            if marker not in serial:
                raise RuntimeError(f"Session evidence missing: {marker}")
    position = [40, 40]
    log = output / "serial.log"
    def text():
        return log.read_text(encoding="utf-8", errors="replace").replace(prefix, "")
    def wait(marker, offset):
        deadline = time.monotonic() + 25
        while marker not in text()[offset:]:
            if time.monotonic() > deadline:
                raise RuntimeError(f"Missing session evidence: {marker}; {output}")
            time.sleep(0.025)
    def send(events):
        capture.command(stream, "input-send-event", {"events": events})
    def button(down):
        send([{"type": "btn", "data": {"button": "left", "down": down}}])
    def key(name, down=None):
        for state in ((True, False) if down is None else (down,)):
            send([{"type": "key", "data": {"down": state, "key": {"type": "qcode", "data": name}}}])
            time.sleep(0.04)
    def move(x, y):
        while position != [x, y]:
            dx, dy = (max(-100, min(100, target - old)) for target, old in zip((x, y), position))
            send([{"type": "rel", "data": {"axis": "x", "value": dx}},
                  {"type": "rel", "data": {"axis": "y", "value": dy}}])
            position[0] += dx
            position[1] += dy
            time.sleep(0.04)
    def click(x, y, app, value):
        move(x, y)
        offset = len(text())
        button(True)
        wait(f"app={app} pointer a=0 b=1 value={value}", offset)
        button(False)
        wait(f"app={app} pointer a=0 b=0 value={value}", offset)
    def press(name, app, code, value, selected):
        offset = len(text())
        key(name)
        wait(f"app={app} key a={code} b=1 value={value} selected={selected}", offset)
        wait(f"app={app} key a={code} b=0", offset)

    snapshot(stream, output, [0, 0, 0], [0, 0, 0], [None] * 3, (40, 40), (424, 184))
    move(50, 40)
    move(90, 170)  # Painted decoration has no input region.
    offset = len(text())
    button(True)
    button(False)
    key("ret")
    time.sleep(0.15)
    if " key a=" in text()[offset:]:
        raise RuntimeError("Decorative pixels acquired keyboard focus")

    click(120, 200, 1, 1)
    press("ret", 1, 5, 2, 0)
    press("right", 1, 4, 2, 1)
    press("ret", 1, 5, 0, 1)
    press("left", 1, 3, 0, 0)
    press("ret", 1, 5, 1, 0)
    offset = len(text())
    key("f7")  # App two drives its own synthetic lease while app one keeps focus.
    wait("synthetic click preserved keyboard focus", offset)
    wait("app=2 pointer a=1 b=1 value=1", offset)
    press("ret", 1, 5, 2, 0)

    click(540, 280, 3, 1)
    offset = len(text())
    key("right")  # Child was delegated Enter only.
    time.sleep(0.15)
    if " key a=4 " in text()[offset:]:
        raise RuntimeError("Child's attenuated keyboard mask expanded")
    press("ret", 3, 5, 2, 0)

    move(120, 280)
    offset = len(text())
    button(True)
    wait("app=1 pointer a=0 b=1 value=2", offset)
    move(440, 280)
    time.sleep(0.1)
    offset = len(text())
    key("left", True)
    wait("app=1 key a=3 b=1 value=2 selected=0", offset)
    offset = len(text())
    key("f4")
    wait("compositor restored; three app identities and values preserved; capture and focus cleared", offset)
    position[:] = [40, 40]  # Rebuilt compositor starts with a neutral root cursor.
    snapshot(stream, output, [2, 1, 2], [0, 0, 0], [(244, 120), None, None], (40, 40), (424, 184), "DISPLAY RESTORED")
    offset = len(text())
    key("ret")
    time.sleep(0.15)
    if " key a=" in text()[offset:]:
        raise RuntimeError("Compositor replacement resurrected keyboard focus")
    button(False)
    move(50, 40)

    # The surviving source seeds held keys into the replacement router. A new
    # physical click may restore focus, but the old held key stays suppressed.
    move(120, 280)
    offset = len(text())
    button(True)
    wait("app=1 pointer a=0 b=1 value=2", offset)
    move(324, 280)  # Same retained marker at local (244,120).
    time.sleep(0.1)
    offset = len(text())
    button(False)
    wait("app=1 pointer a=0 b=0 value=2", offset)
    offset = len(text())
    key("left", True)
    time.sleep(0.15)
    if " key a=3 " in text()[offset:]:
        raise RuntimeError("Compositor replacement reactivated a held key")
    key("left", False)
    move(50, 40)

    offset = len(text())
    key("f6")
    wait("host loss hid child and revoked descendant input", offset)
    wait("host replaced; child process and value preserved", offset)
    snapshot(stream, output, [2, 0, 2], [0, 0, 0], [(244, 120), None, None], (50, 40), (424, 184), "HOST RESTORED", "APP ONE")
    click(540, 280, 3, 3)
    offset = len(text())
    key("ret", True)
    wait("app=3 key a=5 b=1 value=4", offset)
    key("f5")
    wait("input source restored", offset)
    key("ret", False)  # Key-up must survive decoder replacement.
    move(541, 280)  # Neutral mouse report clears conservative source-reset suppression.
    click(540, 280, 3, 5)
    press("ret", 3, 5, 6, 0)
    offset = len(text())
    key("f7")
    wait("synthetic click preserved keyboard focus", offset)
    wait("app=2 pointer a=1 b=1 value=1", offset)
    press("ret", 3, 5, 7, 0)
    snapshot(stream, output, [2, 1, 7], [0, 0, 0], [(244, 120), None, None], (540, 280), (440, 200), "SYNTHETIC CLICK", "CHILD")
    session_latency.drag(stream, output, move, button, text, wait)
    position[:] = [240, 280]
    press("right", 1, 4, 2, 1)
    press("left", 1, 3, 2, 0)
    snapshot(stream, output, [2, 1, 7], [0, 0, 0], [(160, 120), None, None], (240, 280), (440, 200), "SYNTHETIC CLICK", "APP ONE")
    print(f"PASS: explicit hit regions, focus attenuation, sustained dragging, synthetic isolation, nested process scopes, compositor/host/source recovery and complete scanout; {output / 'display.png'}")


def snapshot(stream, output, values, selections, markers, physical, synthetic, status="READY", focus="NONE"):
    width, height = 1024, 768
    expected = bytearray(bytes.fromhex("101827") * width * height)
    font = (Path(__file__).resolve().parents[2] / "source-rs/platform/libraries/bitmap-font/font.hex").read_text().splitlines()
    def put(x, y, color):
        if 0 <= x < width and 0 <= y < height:
            offset = (y * width + x) * 3
            expected[offset:offset + 3] = bytes.fromhex(color)
    def root_label(x, y, text):
        for i, char in enumerate(text):
            for row, bits in enumerate(bytes.fromhex(font[ord(char) - 32])):
                for col in range(5):
                    if bits & (1 << (4 - col)):
                        for dy in range(2):
                            for dx in range(2): put(x + i*12 + col*2 + dx, y + row*2 + dy, "c5d3e8")
    for x, y, text in ((80,40,"CATHEDRAL LAB"),(80,76,"CLICK A BUTTON"),(400,76,"ARROWS / ENTER"),
                      (700,160,"APP ONE"),(700,190,"INDEPENDENT APP"),(700,240,"HOST AND CHILD"),
                      (700,270,"SEPARATE APPS"),(700,300,"CHILD: ENTER"),
                      (80,480,"F4 RESET DISPLAY"),(80,510,"F5 RESET INPUT"),(80,540,"F6 RESET HOST"),
                      (80,570,"F7 SYNTH CLICK"),(400,480,"ESC CANCEL INPUT"),(400,510,"DRAG IN THE BOX"),
                      (400,540,"WHITE: MOUSE"),(400,570,"GOLD: SYNTHETIC"),
                      (80,420,status),(400,420,"KEYS: " + focus)):
        root_label(x,y,text)
    for index, (ox, oy, w, h, color, title) in enumerate(((80, 160, 256, 200, "713b48", "APP ONE"), (400, 160, 256, 200, "214b49", "HOST"), (500, 240, 156, 120, "2a4066", "CHILD"))):
        def rect(x, y, rw, rh, color):
            for py in range(y, min(y + rh, h)):
                for px in range(x, min(x + rw, w)):
                    put(ox + px, oy + py, color)
        def label(x, y, text, scale=2):
            for i, char in enumerate(text):
                for row, bits in enumerate(bytes.fromhex(font[ord(char) - 32])):
                    for col in range(5):
                        if bits & (1 << (4 - col)):
                            rect(x + (i * 6 + col)*scale, y + row*scale, scale, scale, "ffffff")
        rect(0, 0, 256, 200, color)
        label(16, 12, title)
        rect(16, 32, 64, 24, "37618c" if selections[index] == 0 else "101827")
        label(24, 40, "ADD")
        rect(96, 32, 64, 24, "37618c" if selections[index] == 1 else "101827")
        label(98, 40, "CLEAR")
        label(16, 68, f"N {values[index]:03}")
        rect(16, 96, 224, 92, "17293b")
        label(24, 104, "DRAG", 1)
        if markers[index]:
            rect(*markers[index], 12, 12, "101827")
    for tag, position, color in (("1", physical, "ffffff"), ("2", synthetic, "ffd166")):
        def cursor_pixel(dx, dy):
            x, y = position[0] + dx, position[1] + dy
            if tag == "2" and (not (400 <= x < 656 and 160 <= y < 360) or (500 <= x < 656 and 240 <= y < 360)):
                return
            put(x, y, color)
        for row in range(8):
            for col in range(row // 2 + 1):
                cursor_pixel(col, row)
        for row, bits in enumerate(bytes.fromhex(font[ord(tag) - 32])):
            for col in range(5):
                if bits & (1 << (4 - col)):
                    cursor_pixel(col, row + 10)
    deadline = time.monotonic() + 3
    while True:
        actual_w, actual_h, pixels = pointer_test.read_scanout(stream, output)
        if (actual_w, actual_h) != (width, height): raise RuntimeError("Unexpected session geometry")
        if pixels == expected or time.monotonic() >= deadline: break
        time.sleep(0.025)
    if pixels != expected:
        first = next(i // 3 for i, (a, b) in enumerate(zip(pixels, expected)) if a != b)
        raise RuntimeError(f"Session scanout mismatch at ({first % width},{first // width}); {output / 'display.ppm'}")
    capture.save_png(output, width, height, pixels)
