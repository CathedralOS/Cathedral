"""Actual guest reboot/power cuts; cache-loss and torn sectors also have host tests."""
import shutil
import capture
import storage_migration


def run(command, output, timeout, creationflags):
    selected = active = saved = 0
    boot = 0

    def boot_with(exercise):
        nonlocal boot
        boot += 1
        capture.run(command, output, timeout, creationflags, exercise=exercise)
        for name in ("serial.log", "qemu.log", "display.png"):
            source = output / name
            if source.is_file():
                shutil.copyfile(source, output / f"boot-{boot:02}-{name}")

    def check(stream, folder, serial, process, limit):
        capture.wait_log(serial, process, 0, [f"Cathedral: selection={selected} toggles={active}",
                         f"Cathedral: storage=00 saved={saved:04}"], limit)
        capture.snapshot(stream, folder, selected, active, saved=saved)

    def change(stream, folder, serial, process, limit):
        nonlocal selected, active, saved
        check(stream, folder, serial, process, limit)
        for key in ("right", "ret"):
            offset = len(serial.read_bytes())
            if key == "right": selected = (selected + 1) % 3
            else: active ^= 1 << selected
            saved += 1
            capture.key_event(stream, key)
            capture.wait_log(serial, process, offset, [f"Cathedral: selection={selected} toggles={active}",
                             f"Cathedral: storage=00 saved={saved:04}"], limit)
            capture.snapshot(stream, folder, selected, active, saved=saved)

    boot_with(change)
    boot_with(check)
    print("PASS: acknowledged selection and toggles survive a full guest reboot", flush=True)

    for phase in range(1, 5):
        old = (selected, active, saved)
        new = (selected, active ^ (1 << selected), saved + 1)

        def cut(stream, folder, serial, process, limit):
            check(stream, folder, serial, process, limit)
            offset = len(serial.read_bytes())
            capture.key_event(stream, f"f{phase + 4}")
            capture.wait_log(serial, process, offset, [f"Cathedral: storage armed={phase}"], limit)
            capture.key_event(stream, "ret")
            capture.wait_log(serial, process, offset, [f"Cathedral: storage cut={phase}"], limit)
            # No guest shutdown or QMP quit: terminate the emulator immediately.
            process.kill()
            process.wait(timeout=5)

        boot_with(cut)

        def recovered(stream, folder, serial, process, limit):
            nonlocal selected, active, saved
            import re
            text = serial.read_text(encoding="utf-8", errors="replace")
            state = re.search(r"Cathedral: selection=(\d) toggles=(\d)", text)
            version = re.search(r"Cathedral: storage=00 saved=(\d{4})", text)
            if not state or not version:
                raise RuntimeError(f"Recovered state missing: {text}")
            result = (int(state[1]), int(state[2]), int(version[1]))
            allowed = [old] if phase < 3 else [new] if phase == 4 else [old, new]
            if result not in allowed:
                raise RuntimeError(f"Cut {phase}: recovered {result}, expected {allowed}")
            selected, active, saved = result
            check(stream, folder, serial, process, limit)

        boot_with(recovered)
        print(f"PASS: cut phase {phase}, complete recovered state {(selected, active, saved)}", flush=True)

    # Reboot again after recovery and make another acknowledged update, then
    # abruptly stop the VM to check acknowledgements independently of shutdown.
    def acknowledged_cut(*args):
        change(*args)
        args[3].kill()
        args[3].wait(timeout=5)
    boot_with(acknowledged_cut)
    boot_with(check)
    print("PASS: all write boundaries, lost acknowledgement, continued writes and acknowledged durability")

    storage_migration.run(boot_with, output)
