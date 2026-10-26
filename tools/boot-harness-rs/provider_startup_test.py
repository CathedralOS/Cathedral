"""Provider readiness failure/retry through the real F5 owner and recovery UI."""
import argparse
from datetime import datetime, timezone
import json
import os
import re
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


def exercise(stream, folder, mode):
    width, height, _ = pointer_test.read_scanout(stream, folder)
    font = bytes.fromhex((harness.WORKSPACE / 'platform/libraries/bitmap-font/font.hex').read_text())
    permanent = mode in ('repeat-crash', 'bad-volume')

    def log():
        value = (folder / 'serial.log').read_text(errors='replace').replace('Cathedral Rust lab: ', '')
        assert 'CATHEDRAL_RS_PANIC' not in value and 'initial program stopped' not in value, value[-3000:]
        for report in value.split('Cathedral user panic:')[1:]:
            if report.count('\n') < 2:
                continue
            location, message = report.splitlines()[:2]
            if mode in ('crash-once', 'repeat-crash'):
                assert 'storage' in location and message == 'deliberate storage startup failure', report
            elif mode == 'bad-volume':
                assert 'storage' in location and '"no valid volume header"' in message, report
            else:
                raise AssertionError(report)
        return value

    def pixel(data, x, y):
        return data[(y * width + x) * 3:(y * width + x) * 3 + 3].hex()

    def text(data, title, y):
        x = (width - len(title) * 12) // 2
        for i, ch in enumerate(title):
            for row, bits in enumerate(font[(ord(ch) - 32) * 7:(ord(ch) - 31) * 7]):
                for col in range(5):
                    if (pixel(data, x + i * 12 + col * 2, y + row * 2) == 'ffffff') != bool(bits & (1 << (4 - col))):
                        return False
        return True

    def scan(check, description, seconds=45):
        deadline = time.monotonic() + seconds
        while True:
            observed = log()
            w, h, data = pointer_test.read_scanout(stream, folder)
            if check(data, observed):
                capture.save_png(folder, w, h, data)
                return data
            if time.monotonic() >= deadline:
                capture.save_png(folder, w, h, data)
                raise AssertionError(f'{description}: {folder}; {observed[-3000:]}')
            time.sleep(0.05)

    def move(x, y):
        capture.command(stream, 'input-send-event', {'events': [
            {'type': 'abs', 'data': {'axis': 'x', 'value': round(x * 32767 / (width - 1))}},
            {'type': 'abs', 'data': {'axis': 'y', 'value': round(y * 32767 / (height - 1))}}]})
        time.sleep(0.08)

    def button(down):
        capture.command(stream, 'input-send-event', {'events': [
            {'type': 'btn', 'data': {'button': 'left', 'down': down}}]})
        time.sleep(0.1)

    def click(x, y):
        move(x, y)
        button(True)
        button(False)
        move(24, 24)

    def enter():
        for down in (True, False):
            capture.command(stream, 'input-send-event', {'events': [
                {'type': 'key', 'data': {'down': down, 'key': {'type': 'qcode', 'data': 'ret'}}}]})
            time.sleep(0.1)

    top = (height - 112) // 2
    retry_x, retry_y = width // 2 - 72, top + 68  # Inside button, outside its glyphs.
    def retry():
        move(retry_x, retry_y)
        button(True)
        button(False)  # Leave the cursor here: repeated attempts need one click.

    if mode == 'stall-once':
        scan(lambda d, l: text(d, 'STARTING...', top + 16), 'visible asynchronous readiness')
        move(70, 80)
        scan(lambda d, l: any(pixel(d, x, y) == 'ffffff' for x in range(70, 78) for y in range(80, 96)),
             'pointer responds while provider is blocked')
        move(retry_x, retry_y)
        button(True)  # Hold across timeout: old input must not authorize retry.

    def failure(attempt):
        scan(lambda d, l: text(d, 'SIGN-IN UNAVAILABLE', top + 16)
             and text(d, 'TRY AGAIN', top + 77) and l.count('storage-startup-failed') == attempt,
             f'failed storage attempt {attempt}')
        time.sleep(0.8)
        assert log().count('storage startup attempt') == attempt, 'automatic startup retry'
        retired = re.findall(r'storage-control-retired (\d+) (\d+) (\d+) (\d+) (\d+)', log())
        assert len(retired) == attempt, 'every failed setup must release control custody'
        assert all(tuple(map(int, row)) == (0, 0, 4, 0, 0) for row in retired), retired

    failure(1)
    if mode == 'stall-once':
        assert f'storage-startup-failed {(1 << 64) - 110}' in log(), 'startup did not time out'  # ABI TIMED_OUT.
        button(False)
    if permanent:
        for attempt in range(2, 5):
            retry()
            failure(attempt)
        assert log().count('sign-in retry requested') == 3
    else:
        retry()
        user_x = (width - 280) // 2 + 64
        scan(lambda d, l: pixel(d, user_x, height // 2 - 12) == '66ccff', 'picker after explicit provider retry')
        click(user_x, height // 2)
        scan(lambda d, l: pixel(d, 32, height-56) == 'd6e5ef', 'user workspace after retry')
        click(96, height - 48)
        data = scan(lambda d, l: pixel(d, 13, 18) in ('315ca8', 'd18b35') and pixel(d, 40, 50) == 'e5e7eb', 'Colors after retry')
        color = 'd18b35' if pixel(data, 13, 18) == '315ca8' else '315ca8'
        click(440, 180)
        enter()
        scan(lambda d, l: pixel(d, 13, 18) == color and pixel(d, 40, 50) == 'e5e7eb', 'save after provider retry')
        click(width - 22, 18)
        scan(lambda d, l: pixel(d, 13, 18) == '101827', 'close saved app')
        click(96, height - 48)
        scan(lambda d, l: pixel(d, 13, 18) == color and pixel(d, 40, 50) == 'e5e7eb', 'reopen saved app')
        assert log().count('shared storage control established') == (2 if mode == 'bad-ready-once' else 1)
        assert log().count('storage startup attempt') == 2 and log().count('storage startup accepted') == 1
    print(f'PASS: {mode}, visible readiness/recovery, explicit attempts and cleanup; {folder}', flush=True)


def run(source, cases=None):
    original = source.read_bytes()
    output = harness.BUILD / 'provider-startup' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    modes = {'crash-once': ['storage-startup-fault'],
             'bad-ready-once': ['storage-startup-bad-ready'],
             'bad-setup-once': ['storage-startup-bad-setup'],
             'stall-once': ['storage-startup-stall'],
             'repeat-crash': ['storage-startup-fault', 'storage-startup-repeat'],
             'bad-volume': []}
    completed = []
    for mode, features in modes.items():
        if cases and mode not in cases:
            continue
        folder = output / mode
        folder.mkdir()
        composition = json.loads((harness.WORKSPACE / 'distribution/profile.json').read_text())
        composition['startup']['initial']['features'] += features
        if features:
            composition['startup']['launches'][3]['features'] = ['startup-probes']
        profile = folder / 'provider-startup.json'
        profile.write_text(json.dumps(composition, indent=2), encoding='utf-8')
        subprocess.run([sys.executable, str(Path(__file__).with_name('run.py')), '--release', '--build-only',
                        '--profile', str(profile)], cwd=harness.REPO, check=True)
        shutil.copytree(harness.BUILD / 'provider-startup-interactive/esp', folder / 'esp')
        image = folder / 'guest.raw'
        image.write_bytes(original)
        database_image.prepare(image, harness.seeded_programs(profile, True),
                               harness.WORKSPACE / 'distribution/migrations/readable-labels.txt', True)
        if mode == 'bad-volume':
            with image.open('r+b') as stream:
                stream.seek(database_image.OFFSET)
                stream.write(bytes(8192))  # Both envelope headers, on this copy only.
        before = folder / 'before.raw'
        shutil.copyfile(image, before)
        qemu = harness.find_qemu()
        command = [qemu, '-machine', 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1', '-m', '128']
        command += harness.firmware_args(qemu, folder)
        command += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none', '-no-reboot', '-display', 'none']
        command += disk_image.arguments(image)
        capture.run(command, folder, 180, subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0,
                    'Cathedral desktop: interactive startup ready',
                    lambda stream, folder, serial: exercise(stream, folder, mode))
        if mode in ('repeat-crash', 'bad-volume'):
            assert image.read_bytes() == before.read_bytes(), 'pre-open failure changed disk bytes'
        completed.append(mode)
    assert source.read_bytes() == original, 'source image changed'
    (output / 'result.json').write_text(json.dumps({'cases': completed, 'passed': True}), encoding='utf-8')
    print(f'PASS: provider startup suite; {output}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('image', type=Path, help='one-user fixture; only copies are modified')
    parser.add_argument('--case', action='append', choices=('crash-once', 'bad-ready-once', 'bad-setup-once', 'stall-once', 'repeat-crash', 'bad-volume'))
    args = parser.parse_args()
    run(args.image, args.case)
