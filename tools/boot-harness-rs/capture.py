"""Capture the real QEMU scanout through QMP and check the distribution pattern."""
import binascii
import json
from pathlib import Path
import socket
import struct
import subprocess
import time
import zlib


def command(stream, name, arguments=None):
    request = {"execute": name}
    if arguments is not None:
        request["arguments"] = arguments
    stream.write((json.dumps(request) + "\n").encode())
    stream.flush()
    while True:
        line = stream.readline()
        if not line:
            raise RuntimeError("QMP disconnected")
        reply = json.loads(line)
        if "error" in reply:
            raise RuntimeError(f"QMP {name}: {reply['error']}")
        if "return" in reply:
            return reply["return"]


def run(qemu_command, output, timeout, creationflags, input_test=False, recovery_test=False, exercise=None):
    serial = output / "serial.log"
    serial.write_text("", encoding="utf-8")
    with socket.socket() as listener, (output / "qemu.log").open("w") as log:
        listener.bind(("127.0.0.1", 0))
        listener.listen(1)
        listener.settimeout(timeout)
        port = listener.getsockname()[1]
        qemu_command = qemu_command + ["-qmp", f"tcp:127.0.0.1:{port},server=off", "-serial", "file:serial.log"]
        process = subprocess.Popen(qemu_command, cwd=output, stdout=log, stderr=log,
                                   creationflags=creationflags)
        try:
            with listener.accept()[0] as connection:
                connection.settimeout(timeout)
                with connection.makefile("rwb") as stream:
                    if "QMP" not in json.loads(stream.readline()):
                        raise RuntimeError("Missing QMP greeting")
                    command(stream, "qmp_capabilities")
                    deadline = time.monotonic() + timeout
                    while True:
                        text = serial.read_text(encoding="utf-8", errors="replace")
                        if "CATHEDRAL_RS_PANIC" in text or "CATHEDRAL_RS_FAULT:" in text:
                            raise RuntimeError(f"Capture boot failed; logs: {output}")
                        if any(marker in text for marker in ("Cathedral: startup failed", "initial program stopped", "admission failed", "requires unavailable framebuffer")):
                            raise RuntimeError(f"Userspace startup failed; logs: {output}")
                        if "Cathedral: startup ready" in text:
                            break
                        if process.poll() is not None or time.monotonic() >= deadline:
                            raise RuntimeError(f"Capture boot did not complete; logs: {output}")
                        time.sleep(0.05)
                    if "Cathedral kernel: starting supplied initial program" not in text:
                        raise RuntimeError("Initial program handoff missing")
                    if any(marker in text for marker in ("ring3 private memory", "fault contained mode=", "display service faulted and restarted")):
                        raise RuntimeError("Ordinary boot unexpectedly ran lab exercises")
                    print(f"Userspace scene ready after {time.monotonic() - (deadline - timeout):.2f}s from QMP connection", flush=True)
                    if recovery_test or exercise is not None:
                        for marker in ("two-root catalog bounds isolation and backpressure passed",
                                       "counter replacement discarded staged changes",
                                       "counter recovered; status app and providers preserved"):
                            if marker not in text:
                                raise RuntimeError(f"Catalog exercise missing: {marker}")
                    if exercise is not None:
                        exercise(stream, output, serial, process, timeout)
                    else:
                        snapshot(stream, output, health=(1, 1, 0) if recovery_test else (0, 0, 0))
                    if recovery_test:
                        if not all(f"Cathedral: {service} recovered" in text for service in ("input", "display")):
                            raise RuntimeError("First-generation startup wedges were not recovered")
                        exercise_recovery(stream, output, serial, process, timeout)
                    if input_test:
                        exercise_input(stream, output, serial, process, timeout)
                    if process.poll() is None:
                        command(stream, "quit")
            process.wait(timeout=5)
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=5)


def validate(output, selected=0, active=0, health=(0, 0, 0), last=0, storage=0, saved=0, legacy=False):
    with (output / "display.ppm").open("rb") as source:
        if source.readline() != b"P6\n":
            raise RuntimeError("Unexpected screenshot format")
        width, height = map(int, source.readline().split())
        if source.readline() != b"255\n" or (width, height) != (1024, 768):
            raise RuntimeError("Screenshot requires the lab's 1024x768 RGB mode")
        pixels = source.read()
    expected = bytearray(bytes.fromhex("101827") * width * height)
    rectangles = [(64, 64, 896, 8, "59d9cc"),
                              (64, 160, 256, 384, "e96f6f"),
                              (384, 160, 256, 384, "79c99e"),
                              (704, 160, 256, 384, "779bea")]
    for index in range(3):
        x = 64 + index * 320
        if active & (1 << index):
            rectangles.append((x + 16, 344, 224, 16, "101827"))
        if index == selected:
            rectangles.extend([(x - 4, 156, 264, 4, "e8edf4"),
                               (x - 4, 544, 264, 4, "e8edf4"),
                               (x - 4, 160, 4, 384, "e8edf4"),
                               (x + 256, 160, 4, 384, "e8edf4")])
    for x, y, w, h, color in rectangles:
        for row in range(y, y + h):
            start = (row * width + x) * 3
            expected[start:start + w * 3] = bytes.fromhex(color) * w
    font = (Path(__file__).resolve().parents[2] / "source-rs/platform/libraries/bitmap-font/font.hex").read_text().splitlines()
    def label(x, y, text, color):
        for index, char in enumerate(text):
            for row, bits in enumerate(bytes.fromhex(font[ord(char) - 32])):
                for column in range(5):
                    if bits & (1 << (4 - column)):
                        for dy in range(2):
                            start = ((y + row * 2 + dy) * width + x + index * 12 + column * 2) * 3
                            expected[start:start + 6] = bytes.fromhex(color) * 2
    label(64, 96, "CATHEDRAL / STATUS", "e8edf4")
    for x, title in ((80, "DISPLAY"), (400, "INPUT"), (720, "APPLICATION")):
        label(x, 184, title, "101827")
    label(720, 232, "SAVED RECORDS", "101827")
    if legacy:
        label(720, 272, "1 SCENE (LEGACY)", "101827")
    elif saved:
        label(720, 272, f"1 SELECTION {selected}", "101827")
        label(720, 304, f"2 TOGGLES   {active}", "101827")
    else:
        label(720, 272, "1 EMPTY", "101827")
    label(64, 592, "ARROWS SELECT / ENTER TOGGLE", "e8edf4")
    label(64, 632, f"DISPLAY READY {health[0]:02}  INPUT READY {health[1]:02}  APP {health[2]:02}", "59d9cc")
    label(64, 664, "LAST RECOVERY: " + ("NONE", "DISPLAY", "INPUT", "APPLICATION", "STORAGE")[last], "e8edf4")
    label(64, 696, f"STORAGE READY {storage:02}  SAVED {saved:04}", "59d9cc")
    if pixels != expected:
        raise RuntimeError(f"Scanout differs from the expected pattern: {output / 'display.ppm'}")
    # Lossless screenshot conversion with only the Python standard library.
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", binascii.crc32(kind + data))
    rows = b"".join(b"\0" + pixels[row * width * 3:(row + 1) * width * 3] for row in range(height))
    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">2I5B", width, height, 8, 2, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(rows)) + chunk(b"IEND", b"")
    destination = output / "display.png"
    destination.write_bytes(png)
    print(f"PASS: all {width * height} scanout pixels match the startup scene; screenshot: {destination}")


def snapshot(stream, output, selected=0, active=0, health=(0, 0, 0), last=0, storage=0, saved=0, legacy=False):
    command(stream, "screendump", {"filename": str(output / "display.ppm")})
    validate(output, selected, active, health, last, storage, saved, legacy)


def exercise_input(stream, output, serial, process, timeout):
    # Each response follows a completed redraw. Matching only new log bytes avoids
    # treating an earlier identical selection as evidence for the current key.
    actions = [("right", 1, 0), ("ret", 1, 2), ("down", 2, 2),
               ("left", 1, 2), ("up", 0, 2), ("left", 2, 2), ("ret", 2, 6)]
    for saved, (key, selected, active) in enumerate(actions, 1):
        offset = len(serial.read_bytes())
        command(stream, "input-send-event", {"events": [
            {"type": "key", "data": {"down": down, "key": {"type": "qcode", "data": key}}}
            for down in (True, False)]})
        marker = f"Cathedral: selection={selected} toggles={active}"
        deadline = time.monotonic() + timeout
        while True:
            text = serial.read_bytes()[offset:].decode("utf-8", errors="replace")
            if marker in text:
                break
            if process.poll() is not None or time.monotonic() >= deadline or "startup failed" in text:
                raise RuntimeError(f"Input {key} did not produce {marker}; logs: {output}; new log: {text}")
            time.sleep(0.02)
        if " recovered" in text or " restarted" in text or "Cathedral: counter recovered" in text:
            raise RuntimeError(f"Healthy input caused a restart: {text}")
        snapshot(stream, output, selected, active, saved=saved)
    print("PASS: real keyboard events, navigation and toggles in the separate application")


def key_event(stream, key):
    command(stream, "input-send-event", {"events": [
        {"type": "key", "data": {"down": down, "key": {"type": "qcode", "data": key}}}
        for down in (True, False)]})


def wait_log(serial, process, offset, markers, timeout):
    deadline = time.monotonic() + timeout
    while True:
        text = serial.read_bytes()[offset:].decode("utf-8", errors="replace")
        if any(bad in text for bad in ("startup failed", "initial program stopped", "CATHEDRAL_RS_PANIC")):
            raise RuntimeError(f"Recovery guest failed: {text}")
        if all(marker in text for marker in markers):
            return text
        if process.poll() is not None or time.monotonic() >= deadline:
            raise RuntimeError(f"Missing recovery markers {markers}: {text}")
        time.sleep(0.02)


def exercise_recovery(stream, output, serial, process, timeout):
    health = [1, 1, 0]
    last = storage = saved = 0
    selected = active = 0
    def markers():
        return [f"Cathedral: selection={selected} toggles={active}",
                f"Cathedral: health={health[0]:02}/{health[1]:02}/{health[2]:02} last={last}",
                f"Cathedral: storage={storage:02} saved={saved:04}"]
    def check():
        snapshot(stream, output, selected, active, health, last, storage, saved)
    def navigate(key):
        nonlocal selected, active, saved
        offset = len(serial.read_bytes())
        if key == "right": selected = (selected + 1) % 3
        else: active ^= 1 << selected
        saved += 1
        key_event(stream, key)
        wait_log(serial, process, offset, markers(), timeout)
        check()

    if "Cathedral: disk transport checked copies and bounds passed" not in serial.read_text():
        raise RuntimeError("Disk transport probes missing")
    if "Cathedral: application authority isolated" not in serial.read_text():
        raise RuntimeError("Application authority probes missing")
    navigate("right")
    navigate("ret")
    offset = len(serial.read_bytes())
    until = time.monotonic() + 2.5
    while time.monotonic() < until:
        time.sleep(0.05)
        text = serial.read_bytes()[offset:].decode("utf-8", errors="replace")
        if any(bad in text for bad in ("recovered", "restarted", "startup failed", "initial program stopped")):
            raise RuntimeError(f"Healthy idle caused a restart or failure: {text}")
    check()
    for mode in ("crash", "spin", "block"):
        for index, service, key in ((0, "display", "f2"), (1, "input", "f1"), (2, "storage", "f4")):
            offset = len(serial.read_bytes())
            key_event(stream, key)
            wait_log(serial, process, offset, [f"Cathedral: probe {mode} armed"], timeout)
            if service == "display" and mode != "crash":
                key_event(stream, "right")
                selected = (selected + 1) % 3
                saved += 1
            if service == "storage":
                storage += 1
                last = 4
            else:
                health[index] += 1
                last = index + 1
            text = wait_log(serial, process, offset, markers() + [f"Cathedral: {service} recovered",
                           "Cathedral: recovered with sibling and application identities preserved"], timeout)
            if any(f"Cathedral: {other} recovered" in text for other in ("input", "display", "storage") if other != service) or " restarted" in text or "Cathedral: counter recovered" in text:
                raise RuntimeError(f"Recovery unexpectedly restarted another task: {text}")
            check()
            navigate("right")
    for _ in range(3):
        offset = len(serial.read_bytes())
        key_event(stream, "f3")
        health[2] += 1
        last = 3
        text = wait_log(serial, process, offset, markers() + ["Cathedral: application restarted; providers preserved",
                        "Cathedral: application authority isolated", "Cathedral: application ready"], timeout)
        if " recovered" in text:
            raise RuntimeError(f"Application failure restarted a provider: {text}")
        check()
        navigate("right")
        navigate("ret")
    # Commit is durable but reply never arrives. Init replaces the wedged service;
    # the live app reads/reconciles instead of applying the toggle twice.
    offset = len(serial.read_bytes())
    key_event(stream, "f8")
    wait_log(serial, process, offset, ["Cathedral: storage armed=4"], timeout)
    storage += 1
    last = 4
    navigate("ret")
    print("PASS: independent provider/app recovery, private storage authority, retained state and lost-reply reconciliation")
