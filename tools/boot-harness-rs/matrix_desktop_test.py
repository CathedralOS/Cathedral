"""Physical input and real scanout through the matrix-owned process tree."""
import time
import capture
import pointer_test


def verify(stream, output, serial, pressure=False):
    if "subtree replacement cleared pixels and preserved sibling" not in serial:
        raise RuntimeError("Missing matrix-owned subtree recovery evidence")
    if "retained move without app repaint" not in serial:
        raise RuntimeError("Missing matrix idle/retention/budget evidence")
    if pressure and "capped spinner active" not in serial:
        raise RuntimeError("Missing capped matrix pressure peer")
    if "dynamic catalogue admission, rollback and pool reuse passed" not in serial:
        raise RuntimeError("Missing dynamic matrix admission evidence")
    position = [40, 40]

    def send(events):
        capture.command(stream, "input-send-event", {"events": events})

    def move(x, y):
        while position != [x, y]:
            dx = max(-70, min(70, x - position[0]))
            dy = max(-70, min(70, y - position[1]))
            send([{"type": "rel", "data": {"axis": "x", "value": dx}},
                  {"type": "rel", "data": {"axis": "y", "value": dy}}])
            position[0] += dx
            position[1] += dy
            time.sleep(0.04)

    def click(x, y):
        move(x, y)
        for down in (True, False):
            send([{"type": "btn", "data": {"button": "left", "down": down}}])
            time.sleep(0.08)

    def enter():
        for down in (True, False):
            send([{"type": "key", "data": {"down": down, "key": {"type": "qcode", "data": "ret"}}}])
            time.sleep(0.08)

    def pixels(left, right):
        deadline = time.monotonic() + 5
        while True:
            width, height, data = pointer_test.read_scanout(stream, output)
            checks = [(140, 190, left), (510, 190, right),
                      (334, 310, "101827"), (720, 360, "101827")]
            if all(data[(y * width + x) * 3:(y * width + x) * 3 + 3] == bytes.fromhex(color)
                   for x, y, color in checks):
                capture.save_png(output, width, height, data)
                return
            if time.monotonic() > deadline:
                raise RuntimeError(f"Matrix scanout mismatch: {checks}; logs: {output}")
            time.sleep(0.05)

    pixels("315ca8", "2c8762")
    click(170, 220)
    started = time.monotonic()
    enter()
    pixels("d18b35", "2c8762")
    print(f"Nested key to verified scanout (pressure={pressure}): {(time.monotonic() - started) * 1000:.0f} ms")
    click(550, 220)
    enter()
    pixels("d18b35", "d18b35")
    click(80, 500)  # No child may acquire focus here.
    enter()
    pixels("d18b35", "d18b35")
    text = (output / "serial.log").read_text(encoding="utf-8", errors="replace")
    if "user panic:" in text or text.count("matrix app: nested key") != 1 or text.count("matrix app: sibling key") != 1:
        raise RuntimeError(f"Matrix input confinement failed: {text}")
    print(f"PASS: matrix-owned desktop, nested clipping, physical focus, subtree recovery; {output / 'display.png'}")
