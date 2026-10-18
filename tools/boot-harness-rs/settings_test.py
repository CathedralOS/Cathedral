"""Edit saved matrix layout through F5; verify cancel, cold apply and clone isolation."""
import argparse
import json
from datetime import datetime, timezone
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import capture
import database_image
import disk_image
import pointer_test
import run as harness


def exercise(stream, output, serial, boot, recovery=False):
    width, height, _ = pointer_test.read_scanout(stream, output)
    font = bytes.fromhex((harness.REPO / 'source-rs/platform/libraries/bitmap-font/font.hex').read_text(encoding='utf-8'))

    def click(x, y):
        capture.command(stream, 'input-send-event', {'events': [
            {'type': 'abs', 'data': {'axis': 'x', 'value': round(x * 32767 / (width - 1))}},
            {'type': 'abs', 'data': {'axis': 'y', 'value': round(y * 32767 / (height - 1))}}]})
        time.sleep(0.07)
        for down in (True, False):
            capture.command(stream, 'input-send-event', {'events': [
                {'type': 'btn', 'data': {'button': 'left', 'down': down}}]})
            time.sleep(0.1)

    def pixel(data, x, y):
        return data[(y * width + x) * 3:(y * width + x) * 3 + 3].hex()

    def text(data, x, y, value):
        for i, ch in enumerate(value):
            for row, bits in enumerate(font[(ord(ch) - 32) * 7:(ord(ch) - 31) * 7]):
                for col in range(5):
                    white = pixel(data, x + i * 12 + col * 2, y + row * 2) == 'ffffff'
                    if white != bool(bits & (1 << (4 - col))):
                        return False
        return True

    def scan(check, description):
        deadline = time.monotonic() + 30
        while True:
            log = (output / 'serial.log').read_text(encoding='utf-8', errors='replace')
            assert all(s not in log for s in ('user panic:', 'CATHEDRAL_RS_PANIC', 'storage offline')), log[-3000:]
            w, h, data = pointer_test.read_scanout(stream, output)
            if check(data):
                capture.save_png(output, w, h, data)
                return data
            if time.monotonic() >= deadline:
                capture.save_png(output, w, h, data)
                raise AssertionError(f'{description}: {output}')
            time.sleep(0.08)

    if boot == 3:
        click((width - 280) // 2 + 216, height // 2)  # NEW USER
        scan(lambda d: pixel(d, width // 2, height // 2 - 12) == 'c7a2ed', 'second user')
        click(width // 2, height // 2)
    else:
        click((width - 280) // 2 + 64, height // 2)
    margin, strip = (40, 256) if boot == 2 else (24, 192)
    center = margin + strip // 2
    scan(lambda d: pixel(d, center - 20, margin + 46) == 'd9e4ef', 'launcher')
    click(center, margin + 192)  # Colors stays running while settings are edited.
    app_x = margin + strip + 16
    scan(lambda d: pixel(d, app_x + 5, margin + 10) in ('315ca8', 'd18b35'), 'Colors')

    def open_settings():
        click(center, height - margin - 16)
        scan(lambda d: text(d, margin + (strip - 160) // 2 + 4, margin + 26, 'APP STRIP'), 'settings panel')

    def action(row, right):
        click(margin + (strip - 160) // 2 + (120 if right else 40), margin + 76 + row * 80)

    def values(expected_width, expected_margin):
        def x(value):
            return margin + (strip - 160) // 2 + (160 - len(str(value)) * 12) // 2
        scan(lambda d: text(d, x(expected_width), margin + 46, str(expected_width))
             and text(d, x(expected_margin), margin + 126, str(expected_margin)), 'saved values')

    open_settings()
    values(strip, margin)
    if boot == 1:
        action(0, True)
        action(1, True)
        values(256, 40)
        action(2, True)  # CANCEL
        open_settings()
        values(192, 24)
        action(0, True)
        action(1, True)
        action(2, False)  # SAVE
        if recovery:
            scan(lambda d: text(d, margin + (strip - 180) // 2, height - margin - 48, 'SAVE FAILED'), 'lost save acknowledgement')
            deadline = time.monotonic() + 45
            while 'storage replaced; explicit grants restored' not in (output / 'serial.log').read_text(encoding='utf-8', errors='replace'):
                assert time.monotonic() < deadline, 'provider was not restored'
                time.sleep(0.1)
            action(2, False)  # Reconcile the committed bytes before retrying.
        data = scan(lambda d: text(d, margin + (strip - 180) // 2, height - margin - 48, 'SAVED'), 'save acknowledgement')
        if recovery:
            log = (output / 'serial.log').read_text(encoding='utf-8', errors='replace')
            assert log.count('STORAGE_TRACE provider-begin 0 3 ') == 1, 'reconciliation repeated the committed write'
        assert pixel(data, app_x + 5, margin + 10) in ('315ca8', 'd18b35'), 'saving changed the live app layout'
        values(256, 40)
        shutil.copyfile(output / 'display.png', output / 'settings-panel.png')
    action(2, True)
    scan(lambda d: pixel(d, center - 20, margin + 46) == 'd9e4ef', 'return to apps')


def run(source, recovery=False):
    here = Path(__file__).resolve().parent
    output = harness.BUILD / 'settings' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    profile = harness.REPO / 'source-rs/distribution/profile.json'
    esp = harness.BUILD / 'interactive/esp'
    build = [sys.executable, str(here / 'run.py'), '--release', '--build-only']
    if recovery:
        composition = json.loads(profile.read_text(encoding='utf-8'))
        startup = composition['startup']
        for program in [startup['initial'], *startup['programs']]:
            if program.get('package') == 'cathedral-desktop':
                program['features'].append('notes-probes')
        startup['launches'][3]['features'] = ['recovery-lab']
        profile = output / 'settings-recovery.json'
        profile.write_text(json.dumps(composition, indent=2), encoding='utf-8', newline='\n')
        build += ['--profile', str(profile), '--trace-storage']
        esp = harness.BUILD / 'settings-recovery-interactive/esp'
    subprocess.run(build, cwd=harness.REPO, check=True)
    shutil.copytree(esp, output / 'esp')
    image = output / 'guest.raw'
    shutil.copyfile(source, image)
    programs = harness.seeded_programs(profile, True)
    migration = harness.REPO / 'source-rs/distribution/migrations/readable-labels.txt'
    database_image.prepare(image, programs, migration, True)
    qemu = harness.find_qemu()
    command = [qemu, '-machine', 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1', '-m', '128']
    command += harness.firmware_args(qemu, output)
    command += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none', '-no-reboot', '-display', 'none']
    command += disk_image.arguments(image)
    for boot in ((1, 2) if recovery else (1, 2, 3)):
        capture.run(command, output, 180, subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0,
                    'Cathedral desktop: ready; choose a user',
                    lambda stream, folder, serial: exercise(stream, folder, serial, boot, recovery))
        for name in ('serial.log', 'display.png'):
            shutil.copyfile(output / name, output / f'boot-{boot}-{name}')
        before = image.read_bytes()
        database_image.prepare(image, programs, migration, True)
        assert image.read_bytes() == before, 'installation overwrote the saved arrangement'
    print(f'PASS: cancel, saved layout, unchanged live app, cold apply; '
          f'{"lost save reply reconciled" if recovery else "independent user defaults"}; artifacts: {output}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('image', type=Path, help='one-user desktop fixture, opened only through copies')
    parser.add_argument('--recovery', action='store_true', help='lose one committed settings-save reply, then reconcile')
    args = parser.parse_args()
    run(args.image, args.recovery)
