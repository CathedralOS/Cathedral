"""Exercise refused programs, crashing apps and dying user startup on copied F5 media."""
import argparse
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
import elf_trim
import pointer_test
import run as harness


def exercise(stream, output, serial, mode):
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
                    if (pixel(data, x + i * 12 + col * 2, y + row * 2) == 'ffffff') != bool(bits & (1 << (4 - col))):
                        return False
        return True

    def scan(check, description, failures=0):
        deadline = time.monotonic() + 45
        while True:
            log = (output / 'serial.log').read_text(encoding='utf-8', errors='replace')
            assert 'CATHEDRAL_RS_PANIC' not in log and 'initial program stopped' not in log, log[-3000:]
            # The panic location and message are separate serial lines. Inspect
            # each completed report, allowing only this mode's injected panic.
            expected_panic = ('deliberate matrix startup failure' if mode == 'user-fault'
                              else 'deliberate Notes startup failure' if mode == 'early-fault'
                              else 'deliberate Notes admitted failure' if mode == 'app-fault' else None)
            for report in log.replace('Cathedral Rust lab: ', '').split('Cathedral user panic:')[1:]:
                if report.count('\n') >= 2:
                    assert expected_panic is not None and report.splitlines()[1] == expected_panic, log[-3000:]
            w, h, data = pointer_test.read_scanout(stream, output)
            observed = log.count('user-start-refused') if mode == 'user-fault' else log.count(expected_panic) if expected_panic else 0
            if check(data) and observed >= failures:
                capture.save_png(output, w, h, data)
                return data
            if time.monotonic() >= deadline:
                capture.save_png(output, w, h, data)
                raise AssertionError(f'{description}: {output}; {log[-2000:]}')
            time.sleep(0.1)

    user_x = (width - 280) // 2 + 64
    if mode == 'user-fault':
        for attempt in range(1, 4):
            click(user_x, height // 2)
            scan(lambda d: text(d, (width - 180) // 2, height // 2 + 88, 'CANNOT OPEN')
                 and pixel(d, user_x, height // 2 - 12) == '66ccff', 'failed user returns to picker', attempt)
        # Template was untouched. A newly copied user can still furnish and run apps.
        click(user_x + 152, height // 2)
        scan(lambda d: pixel(d, width // 2, height // 2 - 12) == 'c7a2ed', 'new user after startup failures')
        click(width // 2, height // 2)
    else:
        click(user_x, height // 2)
    scan(lambda d: pixel(d, 32, height-56) == 'd6e5ef', 'launcher available')
    if mode != 'user-fault':
        for attempt in range(1, 4):
            click(40, height - 48)
            expected = ('CANNOT OPEN',) if mode == 'bad-program' else ('CANNOT OPEN', 'APP STOPPED') if mode == 'early-fault' else ('APP STOPPED',)
            scan(lambda d: any(text(d, width - 192, height - 56, message) for message in expected)
                 and pixel(d, 13, 18) == '101827', 'failed app leaves launcher usable',
                 attempt if mode in ('early-fault', 'app-fault') else 0)
    click(96, height - 48)
    data = scan(lambda d: pixel(d, 13, 18) in ('315ca8', 'd18b35')
                and pixel(d, 40, 50) == 'e5e7eb', 'Colors starts after repeated failures')
    original = pixel(data, 13, 18)
    click(440, 180)
    for down in (True, False):
        capture.command(stream, 'input-send-event', {'events': [
            {'type': 'key', 'data': {'down': down, 'key': {'type': 'qcode', 'data': 'ret'}}}]})
        time.sleep(0.07)
    scan(lambda d: pixel(d, 13, 18) != original and pixel(d, 40, 50) == 'e5e7eb', 'save after failures')
    click(width - 22, 18)
    scan(lambda d: pixel(d, 13, 18) == '101827', 'normal close after failures')
    # A normal exit must not be mislabeled as a crash by a delayed observation.
    time.sleep(1.2)
    scan(lambda d: not text(d, width - 192, height - 56, 'APP STOPPED'), 'clean exit receipt')


def run(source):
    source_before = source.read_bytes()
    output = harness.BUILD / 'lifecycle' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    here = Path(__file__).resolve().parent
    subprocess.run([sys.executable, str(here / 'run.py'), '--release', '--build-only'], cwd=harness.REPO, check=True)
    # Capture test-only programs separately; the default kernel/ESP remains normal.
    fault_images = {}
    for target, package, feature in [('early', 'cathedral-notes', 'startup-fault'),
                                     ('notes', 'cathedral-notes', 'admitted-fault'),
                                     ('user', 'cathedral-stock-matrix', 'startup-fault')]:
        subprocess.run(['cargo', 'build', '--locked', '--release', '--target', 'x86_64-unknown-none',
                        '--target-dir', str(harness.BUILD / 'cargo'), '--package', package,
                        '--features', feature], cwd=harness.WORKSPACE, check=True)
        image = output / f'{target}-fault.elf'
        elf_trim.trim(harness.BUILD / 'cargo/x86_64-unknown-none/release' / package, image, None)
        fault_images[target] = image
    bad = output / 'bad-program.bin'
    bad.write_bytes(b'This executable entry contains no ELF program.')
    oracle = database_image.oracle()
    programs = harness.seeded_programs(harness.WORKSPACE / 'distribution/profile.json', True)
    for mode, target, program in [('bad-program', 'notes', bad), ('early-fault', 'notes', fault_images['early']),
                                  ('app-fault', 'notes', fault_images['notes']),
                                  ('user-fault', 'user', fault_images['user'])]:
        folder = output / mode
        folder.mkdir()
        shutil.copytree(harness.BUILD / 'interactive/esp', folder / 'esp')
        image = folder / 'guest.raw'
        shutil.copyfile(source, image)
        database_image.prepare(image, programs, harness.WORKSPACE / 'distribution/migrations/readable-labels.txt', True)
        subprocess.run([str(oracle), 'engines', 'volume', 'failure-fixture', str(image),
                        str(database_image.OFFSET), str(database_image.VOLUME_BYTES), target, str(program)], check=True)
        qemu = harness.find_qemu()
        command = [qemu, '-machine', 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1', '-m', '128']
        command += harness.firmware_args(qemu, folder)
        command += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none', '-no-reboot', '-display', 'none']
        command += disk_image.arguments(image)
        capture.run(command, folder, 180, subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0,
                    'Cathedral desktop: ready; choose a user',
                    lambda stream, folder, serial: exercise(stream, folder, serial, mode))
    assert source.read_bytes() == source_before, 'source image changed'
    print(f'PASS: repeated admission refusals, app crashes, user-startup death, surviving launcher and healthy app saves; artifacts: {output}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('image', type=Path, help='one-user desktop fixture; only copies are modified')
    run(parser.parse_args().image)
