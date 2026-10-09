"""Capture the real QEMU scanout through QMP and check the distribution pattern."""
import binascii
import json
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


def run(qemu_command, output, timeout, creationflags, input_test=False):
    serial = output / "serial.log"
    serial.write_text("", encoding="utf-8")
    with socket.socket() as listener, (output / "qemu.log").open("w") as log:
        listener.bind(("127.0.0.1", 0))
        listener.listen(1)
        listener.settimeout(timeout)
        port = listener.getsockname()[1]
        qemu_command += ["-qmp", f"tcp:127.0.0.1:{port},server=off", "-serial", "file:serial.log"]
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
                    snapshot(stream, output)
                    if input_test:
                        exercise_input(stream, output, serial, process, timeout)
                    command(stream, "quit")
            process.wait(timeout=5)
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=5)


def validate(output, selected=0, active=0):
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


def snapshot(stream, output, selected=0, active=0):
    command(stream, "screendump", {"filename": str(output / "display.ppm")})
    validate(output, selected, active)


def exercise_input(stream, output, serial, process, timeout):
    # Each response follows a completed redraw. Matching only new log bytes avoids
    # treating an earlier identical selection as evidence for the current key.
    actions = [("right", 1, 0), ("ret", 1, 2), ("down", 2, 2),
               ("f1", 2, 2), ("left", 1, 2), ("f2", 1, 2),
               ("up", 0, 2), ("left", 2, 2), ("ret", 2, 6)]
    actions += [("f1", 2, 6), ("f2", 2, 6)] * 4
    for key, selected, active in actions:
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
        if key in ("f1", "f2"):
            service = "input" if key == "f1" else "display"
            if f"Cathedral: {service} restarted" not in text:
                raise RuntimeError(f"Missing {service} restart marker")
        snapshot(stream, output, selected, active)
    print("PASS: real keyboard events, navigation, toggles and 10 independent provider restarts preserve scene state")
