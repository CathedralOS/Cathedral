"""Boot QEMU with QMP attached, wait for the guest's ready marker, then hand the
stream to a fixture's verification; save_png writes a scanout losslessly."""
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


def run(qemu_command, output, timeout, creationflags, ready_marker, verify, *, allow_initial_exit=False):
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
                        # User writes split at page boundaries; each chunk has a
                        # boot diagnostic prefix, even in the middle of a marker.
                        text = serial.read_text(encoding="utf-8", errors="replace").replace("Cathedral Rust lab: ", "")
                        if "CATHEDRAL_RS_PANIC" in text or "CATHEDRAL_RS_FAULT:" in text:
                            raise RuntimeError(f"Capture boot failed; logs: {output}")
                        failures = ["Cathedral: startup failed", "admission failed", "requires unavailable framebuffer"]
                        if not allow_initial_exit:
                            failures.append("initial program stopped")
                        if any(marker in text for marker in failures):
                            raise RuntimeError(f"Userspace startup failed; logs: {output}")
                        if ready_marker in text:
                            break
                        if process.poll() is not None or time.monotonic() >= deadline:
                            raise RuntimeError(f"Capture boot did not complete; logs: {output}")
                        time.sleep(0.05)
                    if "Cathedral kernel: starting supplied initial program" not in text:
                        raise RuntimeError("Initial program handoff missing")
                    if any(marker in text for marker in ("ring3 private memory", "fault contained mode=", "display service faulted and restarted")):
                        raise RuntimeError("Ordinary boot unexpectedly ran lab exercises")
                    print(f"Userspace scene ready after {time.monotonic() - (deadline - timeout):.2f}s from QMP connection", flush=True)
                    verify(stream, output, text)
                    if process.poll() is None:
                        command(stream, "quit")
            process.wait(timeout=5)
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=5)


def save_png(output, width, height, pixels):
    # Lossless screenshot conversion with only the Python standard library.
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", binascii.crc32(kind + data))
    rows = b"".join(b"\0" + pixels[row * width * 3:(row + 1) * width * 3] for row in range(height))
    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">2I5B", width, height, 8, 2, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(rows)) + chunk(b"IEND", b"")
    destination = output / "display.png"
    destination.write_bytes(png)
