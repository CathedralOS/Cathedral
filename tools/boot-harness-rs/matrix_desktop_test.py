"""Physical input and real scanout through the matrix-owned process tree."""
import time
import re
import capture
import pointer_test


def verify(stream, output, serial, pressure=False):
    assert serial.count("focus click consumed before source loss") == 1, serial
    for marker in ("shared root control ready", "shared bootstrap display ready"):
        assert serial.count(marker) == 3, (marker, serial)
    if "nested peer survived catalog revocation, provider replacement and sibling death" not in serial:
        raise RuntimeError("Missing nested peer lifetime isolation proof")
    if "nested sibling consumed pointer and keyboard and repainted during blocked persistence" not in serial:
        raise RuntimeError("Missing recursive storage/interaction isolation proof")
    if "timeout, late reply and lost commit acknowledgement reconciled" not in serial:
        raise RuntimeError("Missing stalled provider isolation/recovery proof")
    if "explicit tree recovery restored persisted apps" not in serial:
        raise RuntimeError("Missing input/display provider recovery proof")
    if "atomic presentation, rejection and commit retry passed" not in serial:
        raise RuntimeError("Missing recursive atomic presentation proof")
    if "scoped cursor clipping, transparency, retained move and atomic publication passed" not in serial:
        raise RuntimeError("Missing recursive cursor presentation proof")
    if "subtree replacement cleared pixels and preserved sibling" not in serial:
        raise RuntimeError("Missing matrix-owned subtree recovery evidence")
    if "retained move without app repaint" not in serial:
        raise RuntimeError("Missing matrix idle/retention/budget evidence")
    if pressure and "capped spinner active" not in serial:
        raise RuntimeError("Missing capped matrix pressure peer")
    if "dynamic catalogue admission, rollback and pool reuse passed" not in serial:
        raise RuntimeError("Missing dynamic matrix admission evidence")
    if "owned surface quotas, stale names, removal and subtree stacking passed" not in serial:
        raise RuntimeError("Missing owned surface lifecycle evidence")
    if "physical admission exhaustion preserved sibling IPC and rendering" not in serial:
        raise RuntimeError("Missing physical memory isolation evidence")
    if "sealed bitmap forwarding, clipping, replay and subtree reclamation passed" not in serial:
        raise RuntimeError("Missing sealed bitmap proof")
    if "private persisted colors, revoked writes, staged-work death and sibling recovery passed" not in serial:
        raise RuntimeError("Missing integrated storage/recovery proof")
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
                      (316, 292, "ffcc44"), (316, 299, "7755dd"),
                      (682, 292, "ffcc44"), (334, 292, "101827"),
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
    text = (output / "serial.log").read_text(encoding="utf-8", errors="replace").replace("Cathedral Rust lab: ", "")
    if ("user panic:" in text
            or text.count("matrix app: nested key") - serial.count("matrix app: nested key") != 1
            or text.count("matrix app: sibling key") - serial.count("matrix app: sibling key") != 1):
        raise RuntimeError(f"Matrix input confinement failed: {text}")
    pending = None
    for stage, ticks, cpu in re.findall(r"INPUT_TRACE (app-key|app-painted) (\d+) (\d+)", text):
        ticks, cpu = int(ticks), int(cpu)
        if stage == "app-key":
            pending = (ticks, cpu)
        elif pending:
            print(f"Guest key handling through presentation: {(ticks - pending[0]) * 10} ms elapsed, {cpu - pending[1]} us app CPU")
            pending = None
    print(f"PASS: matrix-owned desktop, nested clipping, physical focus, subtree recovery; {output / 'display.png'}")


def verify_reboot(stream, output, serial):
    for marker in ("shared root control ready", "shared bootstrap display ready"):
        assert serial.count(marker) == 1, (marker, serial)
    if "persisted colors restored after reboot" not in serial or "user panic:" in serial:
        raise RuntimeError(f"Missing persisted reboot evidence; logs: {output}")
    width, height, data = pointer_test.read_scanout(stream, output)
    for x, y in [(140, 190), (510, 190)]:
        if data[(y * width + x) * 3:(y * width + x) * 3 + 3] != bytes.fromhex("d18b35"):
            raise RuntimeError(f"Saved color did not survive reboot at {x},{y}")
    capture.save_png(output, width, height, data)
    print("PASS: both private app colors survived a fresh QEMU process and guest boot")
