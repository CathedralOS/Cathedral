"""Observe the guest's real-IPC matrix checks; this profile has no display grant."""
import subprocess
import time


def run(command, output, timeout, creationflags):
    serial = output / "serial.log"
    serial.write_text("", encoding="utf-8")
    with (output / "qemu.log").open("w", encoding="utf-8") as log:
        process = subprocess.Popen(command + ["-serial", "file:serial.log"], cwd=output,
                                   stdout=log, stderr=log, creationflags=creationflags)
        try:
            deadline = time.monotonic() + timeout
            while time.monotonic() < deadline:
                text = serial.read_text(encoding="utf-8", errors="replace")
                if any(marker in text for marker in ("PANIC", "initial program stopped")):
                    raise RuntimeError(f"Matrix guest failed: {text}")
                if "Cathedral matrix: two identical worlds, local-only views, attenuated input and recovery passed" in text:
                    print(f"PASS: recursive matrix protocol over isolated processes; logs: {output}")
                    return
                if process.poll() is not None:
                    raise RuntimeError(f"Matrix guest exited early: {text}")
                time.sleep(0.05)
            raise RuntimeError(f"Matrix guest timed out: {serial.read_text(encoding='utf-8', errors='replace')}")
        finally:
            if process.poll() is None:
                process.terminate()
            process.wait(timeout=5)
