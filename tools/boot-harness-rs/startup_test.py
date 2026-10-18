"""Missing paths, invalid/crashing Login and manual recovery on copied F5 media."""
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import capture
import colors_test
import database_image
import disk_image
import elf_trim
import pointer_test
import run as harness


def exercise(stream, output, serial, mode):
    width, height, _ = pointer_test.read_scanout(stream, output)
    font = bytes.fromhex((harness.WORKSPACE / 'platform/libraries/bitmap-font/font.hex').read_text())

    def text(data, x, y, value):
        for i, ch in enumerate(value):
            for row, bits in enumerate(font[(ord(ch) - 32) * 7:(ord(ch) - 31) * 7]):
                for col in range(5):
                    at = ((y + row * 2) * width + x + i * 12 + col * 2) * 3
                    if (data[at:at + 3] == b'\xff\xff\xff') != bool(bits & (1 << (4 - col))):
                        return False
        return True

    def scan(attempt):
        deadline = time.monotonic() + 45
        while True:
            log = (output / 'serial.log').read_text(errors='replace')
            assert 'CATHEDRAL_RS_PANIC' not in log and 'initial program stopped' not in log, log[-3000:]
            for report in log.replace('Cathedral Rust lab: ', '').split('Cathedral user panic:')[1:]:
                if report.count('\n') >= 2:
                    assert mode == 'picker-fault' and report.splitlines()[1] == 'deliberate picker startup failure', log[-3000:]
            w, h, data = pointer_test.read_scanout(stream, output)
            ready = (text(data, (width - len('SIGN-IN UNAVAILABLE') * 12) // 2, (height - 112) // 2 + 16, 'SIGN-IN UNAVAILABLE')
                     and text(data, (width - 108) // 2, (height - 112) // 2 + 77, 'TRY AGAIN'))
            if ready and log.count('login-start-refused') >= attempt + 1:
                capture.save_png(output, w, h, data)
                return
            assert time.monotonic() < deadline, f'recovery attempt {attempt}: {output}; {log[-3000:]}'
            time.sleep(0.05)

    def retry(keyboard=False):
        if keyboard:
            # A replacement screen has a fresh input lease. Focus its blank
            # card margin without activating the retry button first.
            capture.command(stream, 'input-send-event', {'events': [
                {'type': 'abs', 'data': {'axis': 'x', 'value': 16384}},
                {'type': 'abs', 'data': {'axis': 'y', 'value': round(((height - 112) // 2 + 48) * 32767 / (height - 1))}}]})
            for down in (True, False):
                capture.command(stream, 'input-send-event', {'events': [
                    {'type': 'btn', 'data': {'button': 'left', 'down': down}}]})
                time.sleep(0.1)
            for down in (True, False):
                capture.command(stream, 'input-send-event', {'events': [
                    {'type': 'key', 'data': {'down': down, 'key': {'type': 'qcode', 'data': 'ret'}}}]})
                time.sleep(0.1)
        else:
            x, y = width // 2, (height - 112) // 2 + 84
            capture.command(stream, 'input-send-event', {'events': [
                {'type': 'abs', 'data': {'axis': 'x', 'value': round(x * 32767 / (width - 1))}},
                {'type': 'abs', 'data': {'axis': 'y', 'value': round(y * 32767 / (height - 1))}}]})
            time.sleep(0.08)
            for down in (True, False):
                capture.command(stream, 'input-send-event', {'events': [
                    {'type': 'btn', 'data': {'button': 'left', 'down': down}}]})
                time.sleep(0.1)
            # Leave focus on the button, but keep the cursor out of the glyphs
            # whose actual scanout the next assertion checks.
            capture.command(stream, 'input-send-event', {'events': [
                {'type': 'abs', 'data': {'axis': 'x', 'value': 1000}},
                {'type': 'abs', 'data': {'axis': 'y', 'value': 1000}}]})

    scan(0)
    if mode == 'transient':
        retry()
        deadline = time.monotonic() + 30
        while True:
            # The real picker has one existing user and the NEW USER tile.
            data = pointer_test.read_scanout(stream, output)[2]
            x, y = (width - 280) // 2 + 64, height // 2 - 12
            at = (y * width + x) * 3
            if data[at:at + 3].hex() == '66ccff':
                break
            assert time.monotonic() < deadline, 'manual retry did not start the picker'
            time.sleep(0.05)
        colors_test.exercise(stream, output, (output / 'serial.log').read_text(errors='replace'), 1, 1, None)
    else:
        for attempt in range(1, 4):
            retry(keyboard=attempt == 2)
            scan(attempt)
        time.sleep(1.5)
        log = (output / 'serial.log').read_text(errors='replace')
        assert log.count('sign-in retry requested') == 3, 'unsolicited retry loop'
        assert log.count('login-start-refused') == 4, 'admission repeated without a user request'
    print(f'PASS: startup {mode}, visible recovery and explicit retry; {output}')


def run(source):
    source_before = source.read_bytes()
    output = harness.BUILD / 'startup' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    profile = output / 'startup-check.json'
    shutil.copyfile(harness.WORKSPACE / 'distribution/profile.json', profile)
    subprocess.run([sys.executable, str(Path(__file__).with_name('run.py')), '--release', '--build-only',
                    '--profile', str(profile)], cwd=harness.REPO, check=True)
    shutil.copytree(harness.BUILD / 'startup-check-interactive/esp', output / 'esp')
    base = output / 'base.raw'
    base.write_bytes(source_before)
    database_image.prepare(base, harness.seeded_programs(profile, True),
                           harness.WORKSPACE / 'distribution/migrations/readable-labels.txt', True)
    subprocess.run(['cargo', 'build', '--locked', '--release', '--target', 'x86_64-unknown-none',
                    '--target-dir', str(harness.BUILD / 'cargo'), '--package', 'cathedral-chooser',
                    '--features', 'picker-startup-fault'], cwd=harness.WORKSPACE, check=True)
    fault = output / 'picker-fault.elf'
    elf_trim.trim(harness.BUILD / 'cargo/x86_64-unknown-none/release/cathedral-chooser', fault, None)
    bad = output / 'invalid.bin'
    bad.write_bytes(b'Not an executable.')
    oracle = database_image.oracle()
    for mode in ('missing-login', 'missing-users', 'bad-login', 'picker-fault', 'transient'):
        folder = output / mode
        folder.mkdir()
        if mode == 'transient':
            composition = json.loads(profile.read_text())
            composition['startup']['initial']['features'].append('login-recovery-probe')
            transient = output / 'startup-transient.json'
            transient.write_text(json.dumps(composition, indent=2), encoding='utf-8')
            subprocess.run([sys.executable, str(Path(__file__).with_name('run.py')), '--release', '--build-only',
                            '--profile', str(transient)], cwd=harness.REPO, check=True)
            shutil.copytree(harness.BUILD / 'startup-transient-interactive/esp', folder / 'esp')
        else:
            shutil.copytree(output / 'esp', folder / 'esp')
        image = folder / 'guest.raw'
        shutil.copyfile(base, image)
        region = [str(image), str(database_image.OFFSET), str(database_image.VOLUME_BYTES)]
        if mode.startswith('missing-'):
            subprocess.run([str(oracle), 'engines', 'volume', 'startup-path-fixture', *region,
                            mode.removeprefix('missing-')], check=True)
        elif mode in ('bad-login', 'picker-fault'):
            subprocess.run([str(oracle), 'engines', 'volume', 'failure-fixture', *region, 'login',
                            str(bad if mode == 'bad-login' else fault)], check=True)
        before = folder / 'before.raw'
        shutil.copyfile(image, before)
        qemu = harness.find_qemu()
        command = [qemu, '-machine', 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1', '-m', '128']
        command += harness.firmware_args(qemu, folder)
        command += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none', '-no-reboot', '-display', 'none']
        command += disk_image.arguments(image)
        capture.run(command, folder, 180, subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0,
                    'Cathedral desktop: ready; choose a user',
                    lambda stream, folder, serial: exercise(stream, folder, serial, mode))
        if mode != 'transient':
            subprocess.run([str(oracle), 'engines', 'volume', 'verify-unchanged', str(before), str(image),
                            str(database_image.OFFSET), str(database_image.VOLUME_BYTES)], check=True)
    assert source.read_bytes() == source_before, 'source fixture changed'
    (output / 'result.json').write_text(json.dumps({'source': str(source), 'cases': 5, 'passed': True}), encoding='utf-8')
    print(f'PASS: root startup recovery, preserved data and successful manual retry; {output}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('image', type=Path, help='one-user desktop image; only copies are modified')
    run(parser.parse_args().image)
