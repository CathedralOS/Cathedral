"""Observe a headless guest protocol proof and stop QEMU on its success marker."""
import subprocess
import time


def run(command, output, timeout, creationflags, marker):
    serial = output / "serial.log"
    serial.write_text("", encoding="utf-8")
    with (output / "qemu.log").open("w", encoding="utf-8") as log:
        process = subprocess.Popen(command + ["-serial", "file:serial.log"], cwd=output,
                                   stdout=log, stderr=log, creationflags=creationflags)
        try:
            deadline = time.monotonic() + timeout
            while time.monotonic() < deadline:
                text = serial.read_text(encoding="utf-8", errors="replace")
                if any(failure in text for failure in ("PANIC", "user panic:", "initial program stopped", "initial program admission failed")):
                    raise RuntimeError(f"Protocol guest failed: {text}")
                if marker in text:
                    print(f"PASS: {marker}; logs: {output}")
                    return
                if process.poll() is not None:
                    raise RuntimeError(f"Protocol guest exited early: {text}")
                time.sleep(0.05)
            raise RuntimeError(f"Protocol guest timed out: {serial.read_text(encoding='utf-8', errors='replace')}")
        finally:
            if process.poll() is None:
                process.terminate()
            process.wait(timeout=5)
