"""End a hung app through the F5 launcher; verify scoped authority on copied media."""
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
import database_image
import disk_image
import elf_trim
import pointer_test
import run as harness


def exercise(stream, output, serial, mode, saved=None):
    recover = mode in ('recovered', 'revoked')
    width, height, _ = pointer_test.read_scanout(stream, output)
    font = bytes.fromhex((harness.WORKSPACE / 'platform/libraries/bitmap-font/font.hex').read_text())

    def click(x, y):
        capture.command(stream, 'input-send-event', {'events': [
            {'type': 'abs', 'data': {'axis': 'x', 'value': round(x * 32767 / (width - 1))}},
            {'type': 'abs', 'data': {'axis': 'y', 'value': round(y * 32767 / (height - 1))}}]})
        time.sleep(0.07)
        for down in (True, False):
            capture.command(stream, 'input-send-event', {'events': [
                {'type': 'btn', 'data': {'button': 'left', 'down': down}}]})
            time.sleep(0.1)

    def key(code):
        for down in (True, False):
            capture.command(stream, 'input-send-event', {'events': [
                {'type': 'key', 'data': {'down': down, 'key': {'type': 'qcode', 'data': code}}}]})
            time.sleep(0.08)

    def pixel(data, x, y):
        return data[(y * width + x) * 3:(y * width + x) * 3 + 3].hex()

    def text(data, x, y, value):
        for i, ch in enumerate(value):
            for row, bits in enumerate(font[(ord(ch) - 32) * 7:(ord(ch) - 31) * 7]):
                for col in range(5):
                    if (pixel(data, x + i * 12 + col * 2, y + row * 2) == 'ffffff') != bool(bits & (1 << (4 - col))):
                        return False
        return True

    def scan(check, description):
        deadline = time.monotonic() + 50
        while True:
            log = (output / 'serial.log').read_text(encoding='utf-8', errors='replace')
            assert all(s not in log for s in ('CATHEDRAL_RS_PANIC', 'initial program stopped', 'storage offline')), log[-3000:]
            for report in log.replace('Cathedral Rust lab: ', '').split('Cathedral user panic:')[1:]:
                if report.count('\n') >= 2:
                    assert recover and report.splitlines()[1] == 'deliberate launcher failure', log[-3000:]
            w, h, data = pointer_test.read_scanout(stream, output)
            if check(data, log):
                capture.save_png(output, w, h, data)
                return data
            if time.monotonic() >= deadline:
                capture.save_png(output, w, h, data)
                raise AssertionError(f'{description}: {output}; {log[-3000:]}')
            time.sleep(0.1)

    def bitmap(data):
        return b''.join(data[((44 + y) * width + 242) * 3:((44 + y) * width + 422) * 3] for y in range(64))

    def warning():
        click(120, height - 90)
        scan(lambda d, _: text(d, 48, 52, 'UNSAVED WORK') and text(d, 78, 136, 'END APP')
             and text(d, 84, 184, 'CANCEL'), 'explicit loss warning and both choices')

    click((width - 280) // 2 + 64, height // 2)
    scan(lambda d, _: pixel(d, 100, 70) == 'd9e4ef', 'launcher')
    click(120, 96)
    scan(lambda d, s: pixel(d, 237, 34) == '27664a' and 'ungranted management denied' in s, 'Notes and denied app authority')
    if saved is not None:
        scan(lambda d, _: bitmap(d) == saved, 'only saved work survives cold reboot')
        return saved
    if mode == 'revoked':
        click(120, 350)
        key('f6')
        scan(lambda d, s: 'management ticket retained' in s, 'ticket before revocation')
    click(440, 180)
    key('z')
    draft = bitmap(scan(lambda d, _: pixel(d, 237, 34) == '9b7028', 'dirty Notes'))
    if mode == 'revoked':
        scan(lambda d, s: 'launcher owner probe applied' in s
             and not text(d, 78, height - 96, 'END APP'), 'owner revocation removes management visibility')
        click(120, 350)
        key('f5')
        scan(lambda d, s: 'stale management ticket denied' in s and bitmap(d) == draft, 'revoked ticket cannot stop app')
        key('f8')
        scan(lambda d, s: 'deliberate launcher failure' in s and pixel(d, 100, 70) == '101827'
             and bitmap(d) == draft, 'explicit revoke cancels automatic launcher policy')
        click(440, 180)
        key('f1')
        scan(lambda d, s: pixel(d, 237, 34) == '27664a' and 'launcher recovered' not in s, 'app saves after authority revocation')
        click(width - 38, 34)
        scan(lambda d, _: pixel(d, 237, 34) == '101827', 'app-owned close after revocation')
        return draft
    warning()
    key('ret')  # Cancel is the default keyboard choice.
    scan(lambda d, s: bitmap(d) == draft and pixel(d, 100, 70) == 'd9e4ef'
         and 'managed task stopped' not in s, 'default Cancel preserves the task and draft')
    warning()
    click(120, 192)  # Explicit pointer Cancel.
    scan(lambda d, _: bitmap(d) == draft and pixel(d, 100, 70) == 'd9e4ef', 'pointer Cancel')
    click(440, 180)
    key('f1')
    saved = bitmap(scan(lambda d, _: pixel(d, 237, 34) == '27664a', 'saved baseline'))
    key('x')
    draft = bitmap(scan(lambda d, _: pixel(d, 237, 34) == '9b7028', 'unsaved suffix'))
    key('f8')
    scan(lambda d, s: 'deliberate management hang' in s and bitmap(d) == draft, 'non-yielding app')
    if recover:
        click(120, 350)
        key('f8')
        scan(lambda d, s: 'launcher recovered; retry consumed' in s
             and pixel(d, 100, 70) == 'd9e4ef' and bitmap(d) == draft, 'new launcher with the existing hung app')
    click(120, 350)
    key('f6')
    scan(lambda d, s: 'management ticket retained' in s, 'retain the old task ticket')
    warning()
    shutil.copyfile(output / 'display.png', output / 'confirmation.png')
    click(120, 144)
    scan(lambda d, s: pixel(d, 237, 34) == '101827' and s.count('managed task stopped') == 1, 'explicit stop reclaims hung task')
    click(120, 96)
    scan(lambda d, s: pixel(d, 237, 34) == '27664a' and bitmap(d) == saved
         and s.count('ungranted management denied') == 2, 'replacement Notes contains only saved work')
    click(120, 350)
    key('f5')
    scan(lambda d, s: 'stale management ticket denied' in s and bitmap(d) == saved
         and pixel(d, 237, 34) == '27664a', 'old confirmation cannot stop replacement')
    click(width - 38, 34)
    scan(lambda d, _: pixel(d, 237, 34) == '101827', 'ordinary app close')
    click(120, 216)
    before = pixel(scan(lambda d, _: pixel(d, 237, 34) in ('315ca8', 'd18b35'), 'Colors after force stop'), 237, 34)
    click(440, 180)
    key('ret')
    scan(lambda d, _: pixel(d, 237, 34) in ('315ca8', 'd18b35') and pixel(d, 237, 34) != before
         and pixel(d, 264, 66) == 'e5e7eb', 'Colors still saves')
    return saved


def run(source):
    before = source.read_bytes()
    output = harness.BUILD / 'management' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    profile = output / 'management-check.json'
    shutil.copyfile(harness.WORKSPACE / 'distribution/profile.json', profile)
    subprocess.run([sys.executable, str(Path(__file__).with_name('run.py')), '--release', '--build-only',
                    '--profile', str(profile)], cwd=harness.REPO, check=True)
    images = {}
    for target, package, feature in [('notes', 'cathedral-notes', 'management-probes'),
                                     ('launcher', 'cathedral-chooser', 'recovery-probes')]:
        subprocess.run(['cargo', 'build', '--locked', '--release', '--target', 'x86_64-unknown-none',
                        '--target-dir', str(harness.BUILD / 'cargo'), '--package', package,
                        '--features', feature], cwd=harness.WORKSPACE, check=True)
        images[target] = output / (target + '.elf')
        elf_trim.trim(harness.BUILD / 'cargo/x86_64-unknown-none/release' / package, images[target], None)
    oracle = database_image.oracle()
    modes = ('original', 'recovered', 'revoked')
    for mode in modes:
        folder = output / mode
        folder.mkdir()
        esp = harness.BUILD / 'management-check-interactive/esp'
        if mode == 'revoked':
            composition = json.loads(profile.read_text())
            composition['startup']['initial']['features'].append('management-revoke-probe')
            owner_profile = folder / 'management-revoke.json'
            owner_profile.write_text(json.dumps(composition), encoding='utf-8', newline='\n')
            subprocess.run([sys.executable, str(Path(__file__).with_name('run.py')), '--release', '--build-only',
                            '--profile', str(owner_profile)], cwd=harness.REPO, check=True)
            esp = harness.BUILD / 'management-revoke-interactive/esp'
        shutil.copytree(esp, folder / 'esp')
        image = folder / 'guest.raw'
        shutil.copyfile(source, image)
        database_image.prepare(image, harness.seeded_programs(profile, True),
                               harness.WORKSPACE / 'distribution/migrations/readable-labels.txt', True)
        for target, program in images.items():
            # Upgrading old code and installing two probes can exhaust the small
            # engine's physical history. Compact only this disposable copy.
            subprocess.run([str(oracle), 'engines', 'volume', 'compact', str(image),
                            str(database_image.OFFSET), str(database_image.VOLUME_BYTES)], check=True)
            subprocess.run([str(oracle), 'engines', 'volume', 'failure-fixture', str(image),
                            str(database_image.OFFSET), str(database_image.VOLUME_BYTES), target, str(program)], check=True)
        qemu = harness.find_qemu()
        command = [qemu, '-machine', 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1', '-m', '128']
        command += harness.firmware_args(qemu, folder)
        command += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none', '-no-reboot', '-display', 'none']
        command += disk_image.arguments(image)
        saved = [None]
        def verify(stream, folder, serial):
            saved[0] = exercise(stream, folder, serial, mode, saved[0])
        for boot in ((1,) if mode == 'revoked' else (1, 2)):
            capture.run(command, folder, 240, subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0,
                        'Cathedral desktop: ready; choose a user', verify)
            for name in ('serial.log', 'display.png'):
                shutil.copyfile(folder / name, folder / f'boot-{boot}-{name}')
    assert source.read_bytes() == before, 'source fixture changed'
    (output / 'result.json').write_text(json.dumps({'source_unchanged': True, 'modes': modes}), encoding='utf-8', newline='\n')
    print(f'PASS: hung app termination, both Cancel actions, stale/ungranted authority, launcher recovery, replacement apps and cold persistence; artifacts: {output}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('image', type=Path, help='one-user desktop fixture; only copies are changed')
    run(parser.parse_args().image)
