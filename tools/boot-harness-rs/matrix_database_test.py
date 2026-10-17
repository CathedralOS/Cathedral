#!/usr/bin/env python3
"""Save through recursive matrix capabilities and cold-open the database desktop."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import capture
import disk_image
import pointer_test
import run as harness
import database_image
import storage_timing
import notes_test

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def desktop(stream, output, serial, reboot):
    assert 'Cathedral: database catalog ready' in serial
    position = [40, 40]

    def send(events):
        capture.command(stream, 'input-send-event', {'events': events})

    def click(x, y):
        while position != [x, y]:
            dx = max(-70, min(70, x - position[0]))
            dy = max(-70, min(70, y - position[1]))
            send([{'type': 'rel', 'data': {'axis': 'x', 'value': dx}},
                  {'type': 'rel', 'data': {'axis': 'y', 'value': dy}}])
            position[0] += dx
            position[1] += dy
            time.sleep(0.04)
        for down in (True, False):
            send([{'type': 'btn', 'data': {'button': 'left', 'down': down}}])
            time.sleep(0.08)

    def enter():
        for down in (True, False):
            send([{'type': 'key', 'data': {'down': down, 'key': {'type': 'qcode', 'data': 'ret'}}}])
            time.sleep(0.08)

    def pixels(left, right):
        deadline = time.monotonic() + 10
        checks = [(140, 190, left), (510, 190, right)]
        while True:
            text = (output / 'serial.log').read_text(encoding='utf-8', errors='replace')
            if 'user panic:' in text or 'CATHEDRAL_RS_PANIC' in text:
                raise RuntimeError(f'Database desktop process failed; logs: {output}')
            width, height, data = pointer_test.read_scanout(stream, output)
            assert (width, height) == (1024, 768)
            if all(data[(y * width + x) * 3:(y * width + x) * 3 + 3] == bytes.fromhex(color)
                   for x, y, color in checks):
                capture.save_png(output, width, height, data)
                break
            if time.monotonic() >= deadline:
                raise RuntimeError(f'Database desktop scanout mismatch: {checks}; logs: {output}')
            time.sleep(0.05)

    if reboot:
        pixels('d18b35', 'd18b35')
    else:
        pixels('315ca8', '2c8762')
        click(170, 220)
        enter()
        pixels('d18b35', '2c8762')
        click(550, 220)
        enter()
        pixels('d18b35', 'd18b35')
        click(80, 500)  # Outside both children: no app may save another toggle.
        enter()
        pixels('d18b35', 'd18b35')
    text = (output / 'serial.log').read_text(encoding='utf-8', errors='replace').replace('Cathedral Rust lab: ', '')
    assert 'user panic:' not in text
    assert text.count('matrix app: nested key') == (0 if reboot else 1)
    assert text.count('matrix app: sibling key') == (0 if reboot else 1)


def run(args):
    name = 'notes-recovery' if args.notes_recovery else 'notes' if args.notes else 'matrix-database-recovery' if args.recovery else 'matrix-database'
    profile = HERE / f'profiles/{name}.json'
    build = [sys.executable, str(HERE / 'run.py'), '--profile', str(profile), '--build-only']
    if args.release: build.append('--release')
    if args.notes: build.append('--trace-storage')
    subprocess.run(build, cwd=ROOT, check=True)
    lab = ROOT / 'tools/storage-lab'
    subprocess.run(['cargo', 'build', '--locked', '--release', '--features', 'engines'], cwd=lab, check=True)
    executable = lab / 'target/release' / ('storage-layout.exe' if os.name == 'nt' else 'storage-layout')
    output = harness.BUILD / 'matrix-database' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    shutil.copytree(harness.BUILD / f'{name}-interactive/esp', output / 'esp')
    path = output / "guest.raw"
    database_image.prepare(path)
    original = path.read_bytes()
    qemu = harness.find_qemu()
    command = [qemu, '-machine', 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1', '-m', '128']
    command += harness.firmware_args(qemu, output)
    command += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none', '-no-reboot', '-display', 'none']
    command += disk_image.arguments(path)
    flags = subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0
    for boot in (1, 2):
        capture.run(command, output, args.timeout, flags, ready_marker='Cathedral matrix desktop: ready',
                    verify=lambda stream, output, serial: notes_test.desktop(stream, output, serial, boot == 2, args.notes_recovery) if args.notes else desktop(stream, output, serial, boot == 2))
        for name in ('serial.log', 'qemu.log', 'display.png'):
            shutil.copyfile(output / name, output / f'boot-{boot}-{name}')
        if args.recovery or args.notes:
            storage_timing.summarize(output, (output / 'serial.log').read_text(encoding='utf-8', errors='replace'), boot)
        if args.recovery and boot == 1:
            serial = (output / 'serial.log').read_text(encoding='utf-8', errors='replace').replace('Cathedral Rust lab: ', '')
            assert 'live stall watchdog restored grants' in serial
            assert 'uncertain CREATE reconciled without duplication' in serial
            assert 'precommit and lost-reply replacement preserved live siblings and exact revisions' in serial
        actual = path.read_bytes()
        database_image.prepare(path)
        assert path.read_bytes() == actual  # Ordinary relaunch never reformats.
        assert len(actual) == len(original)
        assert actual[:4096] == original[:4096]
        assert actual[4096 + database_image.VOLUME_BYTES:] == original[4096 + database_image.VOLUME_BYTES:]
        # Host opens a copy, leaving the next guest to recover its own dirty engine.
        snapshot = output / f'host-reopen-{boot}.raw'
        with snapshot.open('xb') as file: file.write(actual)
        result = subprocess.run([str(executable), 'engines', 'volume', ('verify-notes' if args.notes else 'verify-catalog-recovery' if args.recovery else 'verify-catalog'), str(snapshot),
                                 '4096', str(database_image.VOLUME_BYTES)], cwd=lab, check=True, capture_output=True, text=True)
        (output / f'host-{boot}.txt').write_text(result.stdout, encoding='utf-8')
        print(result.stdout, end='')
    sources = [profile, Path(__file__), HERE / 'notes_test.py', HERE / 'storage_timing.py', HERE / 'database_image.py', ROOT / 'source-rs/Cargo.lock',
               *(ROOT / 'source-rs/distribution/applications/notes').rglob('*.rs'),
               *(ROOT / 'source-rs/distribution/desktop/session').rglob('*.rs'),
               *(ROOT / 'source-rs/platform/services/matrix/service').rglob('*.rs'),
               *(ROOT / 'source-rs/platform/services/matrix/world').rglob('*.rs'),
               *(ROOT / 'source-rs/platform/services/input/decoder').rglob('*.rs'),
               ROOT / 'source-rs/platform/services/input/matrix.rs',
               ROOT / 'source-rs/platform/contracts/input.rs', ROOT / 'source-rs/platform/contracts/matrix.rs',
               *(ROOT / 'source-rs/platform/libraries/object-store/database').glob('*.rs'),
               *(ROOT / 'source-rs/platform/services/storage/database').glob('*.rs'),
               ROOT / 'tools/storage-lab/engines/volume/catalog.rs']
    context = {'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
               'release': args.release, 'recovery': args.recovery, 'notes': args.notes, 'notes_recovery': args.notes_recovery,
               'qemu': subprocess.check_output([qemu, '--version'], text=True).splitlines()[0],
               'efi_sha256': hashlib.sha256((output / 'esp/EFI/BOOT/BOOTX64.EFI').read_bytes()).hexdigest(),
               'source_sha256': {p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in sources},
               'limits': 'Two private catalogs, ordinary matrix IPC. QMP quit leaves the guest database open; it does not emulate physical cache loss.'}
    (output / 'context.json').write_text(json.dumps(context, indent=2) + '\n', encoding='utf-8')
    print(f'PASS: matrix-scoped database persistence and cold desktop restoration; artifacts: {output}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--release', action='store_true', required=True,
                        help='required for matrix storage response deadlines under TCG')
    parser.add_argument('--notes-recovery', action='store_true', help='notes post-commit stall and draft-preserving reconciliation')
    parser.add_argument('--notes', action='store_true', help='physical scratchpad edits, atomic saves and exact cold restoration')
    parser.add_argument('--recovery', action='store_true', help='kill/restart provider across ambiguous writes and collect timing')
    parser.add_argument('--timeout', type=float, default=180)
    args = parser.parse_args()
    args.notes = args.notes or args.notes_recovery
    if args.notes and args.recovery: parser.error('notes and recovery select separate compositions')
    if args.timeout <= 0: parser.error('timeout must be positive')
    run(args)
