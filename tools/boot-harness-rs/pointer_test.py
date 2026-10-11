"""Drive the emulated PS/2 device through QMP; observe app delivery and actual scanout."""
import time
import capture
import compositor_test
from pathlib import Path


def verify(stream, output, serial):
    if "hardware spoof wrong owner unearned capture bounds and stale lease passed" not in serial:
        raise RuntimeError("Pointer authority probes missing")
    log = output / "serial.log"
    position = [40, 40]

    def text():
        return log.read_text(encoding="utf-8", errors="replace")

    def wait(marker, offset):
        deadline = time.monotonic() + 12
        while marker not in text()[offset:]:
            if time.monotonic() > deadline:
                raise RuntimeError(f"Missing pointer evidence: {marker}; logs: {output}")
            time.sleep(0.02)

    def event(app, pointer, kind, x, y, buttons):
        return f"app={app} id={pointer} kind={kind} x={x} y={y} buttons={buttons}"

    def send(events):
        capture.command(stream, "input-send-event", {"events": events})

    def move(x, y):
        # Bounded batches avoid turning a host test into controller overflow.
        while position != [x, y]:
            dx = max(-100, min(100, x - position[0]))
            dy = max(-100, min(100, y - position[1]))
            send([{"type": "rel", "data": {"axis": "x", "value": dx}},
                  {"type": "rel", "data": {"axis": "y", "value": dy}}])
            position[0] += dx
            position[1] += dy
            time.sleep(0.04)

    def button(name, down):
        send([{"type": "btn", "data": {"button": name, "down": down}}])

    def key(name):
        for down in (True, False):
            send([{"type": "key", "data": {"down": down, "key": {"type": "qcode", "data": name}}}])
            time.sleep(0.03)

    move(50, 40)  # Neutral report clears source-reset suppression.
    start = len(text())
    move(120, 200)
    wait(event(1, 0, 1, 40, 40, 0), start)
    start = len(text())
    button("left", True)
    wait(event(1, 0, 2, 40, 40, 1), start)
    start = len(text())
    move(400, 220)
    wait(event(1, 0, 1, 320, 60, 1), start)
    if "app=2 id=0" in text()[start:]:
        raise RuntimeError("Captured drag leaked to the overlapped app")
    start = len(text())
    key("esc")
    wait(event(1, 0, 3, 320, 60, 0), start)
    button("left", False)
    time.sleep(0.08)

    start = len(text())
    move(280, 200)
    button("left", True)
    wait(event(2, 0, 2, 40, 40, 1), start)
    if "app=1 id=0 kind=2" in text()[start:]:
        raise RuntimeError("Overlap press reached the covered app")
    button("left", False)
    wait(event(2, 0, 2, 40, 40, 0), start)
    start = len(text())
    button("right", True)
    wait(event(2, 1, 2, 80, 80, 1), start)
    button("right", False)
    wait(event(2, 1, 2, 80, 80, 0), start)

    move(120, 200)
    start = len(text())
    button("left", True)
    wait(event(1, 0, 2, 40, 40, 1), start)
    button("right", True)  # Deliberate fault with a live capture.
    wait("Cathedral pointer: app 1 recovered", start)
    start = len(text())
    move(280, 200)
    time.sleep(0.15)
    if "app=2 id=0 kind=2" in text()[start:]:
        raise RuntimeError("Dead app's held press was retargeted")
    button("right", False)
    button("left", False)
    time.sleep(0.08)

    move(120, 200)
    start = len(text())
    button("left", True)
    wait(event(1, 0, 2, 40, 40, 1), start)
    key("f5")  # Lab-only input-provider replacement.
    wait(event(1, 0, 3, 40, 40, 0), start)
    wait("Cathedral pointer: source recovered", start)
    button("left", False)
    move(121, 200)
    time.sleep(0.1)

    start = len(text())
    button("middle", True)
    wait("Cathedral pointer: app 1 flooding", start)
    move(280, 200)
    _, _, during_flood = read_scanout(stream, output)
    if "Cathedral pointer: app 1 recovered" in text()[start:]:
        raise RuntimeError("Cursor flood probe ran after recovery; no live-flood evidence")
    if during_flood[(200 * 1024 + 280) * 3:(200 * 1024 + 281) * 3] != bytes.fromhex("ffffff"):
        raise RuntimeError("Root cursor stopped progressing during client flood")
    button("middle", False)
    wait("Cathedral pointer: app 1 recovered", start)
    # Once every button is up, a fresh click must progress through the live peer.
    move(281, 200)
    start = len(text())
    button("left", True)
    wait(event(2, 0, 2, 41, 40, 1), start)
    button("left", False)
    wait(event(2, 0, 2, 41, 40, 0), start)
    wait(event(2, 1, 1, 81, 80, 0), start)
    screenshot(stream, output)
    print(f"PASS: PS/2 motion/buttons, overlap, capture, Escape, synthetic scope, app/source recovery and flood recovery; {output / 'display.png'}")


def read_scanout(stream, output):
    capture.command(stream, "screendump", {"filename": str(output / "display.ppm")})
    with (output / "display.ppm").open("rb") as source:
        if source.readline() != b"P6\n":
            raise RuntimeError("Unexpected scanout encoding")
        width, height = map(int, source.readline().split())
        if source.readline() != b"255\n" or (width, height) != (1024, 768):
            raise RuntimeError("Expected 1024x768 scanout")
        pixels = source.read()
    return width, height, pixels


def screenshot(stream, output):
    width, height, pixels = read_scanout(stream, output)
    expected = compositor_test.expected_scene(((80, "e96f6f", "APP ONE"), (240, "79c99e", "APP TWO")))
    def put(x, y, color):
        offset = (y * width + x) * 3
        expected[offset:offset + 3] = bytes.fromhex(color)
    # App one's replacement has no marker; app two retains its final physical drag.
    for y in range(200, 212):
        for x in range(281, 293):
            put(x, y, "101827")
    font = (Path(__file__).resolve().parents[2] / "source-rs/platform/libraries/bitmap-font/font.hex").read_text().splitlines()
    for x, y, color, tag in ((281, 200, "ffffff", "1"), (321, 240, "ffd166", "2")):
        for row in range(8):
            for column in range(row // 2 + 1):
                put(x + column, y + row, color)
        for row, bits in enumerate(bytes.fromhex(font[ord(tag) - 32])):
            for column in range(5):
                if bits & (1 << (4 - column)):
                    put(x + column, y + 10 + row, color)
    if pixels != expected:
        mismatch = next(i // 3 for i, (actual, wanted) in enumerate(zip(pixels, expected)) if actual != wanted)
        raise RuntimeError(f"Pointer scanout mismatch at ({mismatch % width},{mismatch // width}); {output / 'display.ppm'}")
    capture.save_png(output, width, height, pixels)
