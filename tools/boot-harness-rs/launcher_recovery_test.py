"""Crash the configured launcher while Notes holds an unsaved draft; copies only."""
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
    width, height, _ = pointer_test.read_scanout(stream, output)
    font = bytes.fromhex((harness.WORKSPACE / 'platform/libraries/bitmap-font/font.hex').read_text())
    margin, strip = (24, 320) if saved is not None else (24, 192)

    def log():
        value = (output / 'serial.log').read_text(encoding='utf-8', errors='replace')
        assert all(s not in value for s in ('CATHEDRAL_RS_PANIC', 'initial program stopped', 'storage offline', 'launcher-recovery-refused')), value[-3000:]
        for report in value.replace('Cathedral Rust lab: ', '').split('Cathedral user panic:')[1:]:
            if report.count('\n') >= 2:
                assert report.splitlines()[1] in ('deliberate launcher failure', 'deliberate launcher startup failure', 'deliberate launcher fallback failure'), value[-3000:]
        return value

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
        deadline = time.monotonic() + 45
        while True:
            current = log()
            w, h, data = pointer_test.read_scanout(stream, output)
            if check(data, current):
                capture.save_png(output, w, h, data)
                return data
            if time.monotonic() >= deadline:
                capture.save_png(output, w, h, data)
                raise AssertionError(f'{description}: {output}; {current[-3000:]}')
            time.sleep(0.1)

    def bitmap(data):
        app = margin + strip + 16
        return b''.join(data[((margin + 20 + y) * width + app + 10) * 3:
                            ((margin + 20 + y) * width + app + 190) * 3] for y in range(64))

    def header(data):
        return pixel(data, margin + strip + 21, margin + 10)

    def focus():
        click(margin + strip + 216, margin + 130)

    def settings():
        click(margin + strip // 2, height - margin - 16)
        scan(lambda d, _: text(d, margin + (strip - 160) // 2 + 4, margin + 26, 'APP STRIP'), 'settings')

    def action(row, right):
        click(margin + (strip - 160) // 2 + (120 if right else 40), margin + 76 + row * 80)

    click((width - 280) // 2 + 64, height // 2)
    if mode == 'startup':
        scan(lambda d, s: s.count('deliberate launcher startup failure') == 2
             and s.count('launcher recovered; retry consumed') == 1
             and text(d, 72, height // 2 - 40, 'LAUNCHER'), 'startup retry is bounded and fallback is visible')
        time.sleep(3)
        assert log().count('deliberate launcher startup failure') == 2
        for attempt in (1, 2):
            click(120, height // 2 + 28)
            scan(lambda d, s: s.count('explicit launcher retry') == attempt
                 and s.count('deliberate launcher startup failure') == 2 + attempt
                 and text(d, 72, height // 2 - 40, 'LAUNCHER'), 'one startup attempt per explicit retry')
            time.sleep(2)
            assert log().count('deliberate launcher startup failure') == 2 + attempt
        return None
    scan(lambda d, _: pixel(d, margin + strip // 2 - 20, margin + 46) == 'd9e4ef', 'launcher')
    click(margin + strip // 2, margin + 72)
    scan(lambda d, _: header(d) == '27664a', 'Notes')
    focus()
    if saved is not None:
        scan(lambda d, _: bitmap(d) == saved, 'saved draft survived cold reboot')
        assert log().count('note-loaded') == 1
        return saved
    key('z')
    draft = bitmap(scan(lambda d, _: header(d) == '9b7028', 'unsaved draft'))
    if mode == 'recover':
        settings()
        action(0, True)
        action(1, True)
        action(2, False)
        margin, strip = 40, 256
        scan(lambda d, _: text(d, margin + (strip - 180) // 2, height - margin - 48, 'APPLIED')
             and bitmap(d) == draft, 'live arrangement before crash')
        action(2, True)
    if mode.startswith('owner-'):
        scan(lambda d, s: 'launcher owner probe applied' in s, 'explicit owner action')
    # Focus the empty part of the strip so the fault key belongs to the launcher.
    if mode != 'owner-stop':
        click(margin + strip // 2, margin + 490)
        key('f7' if mode == 'clean' else 'f8')
    restores = mode == 'recover' or mode.startswith('fallback-')
    expected = 1 if restores else 0
    scan(lambda d, s: s.count('launcher recovered; retry consumed') == expected
         and pixel(d, margin + strip // 2 - 20, margin + 46) == ('d9e4ef' if restores else '101827')
         and bitmap(d) == draft and header(d) == '9b7028', 'draft retained across launcher termination')
    assert log().count('note-loaded') == 1, 'Notes was reloaded/restarted'
    if mode.startswith('fallback-'):
        click(120, 354)
        key('f8')
        focus()
        scan(lambda d, s: s.count('deliberate launcher failure') == 2
             and text(d, 72, height // 2 - 40, 'LAUNCHER') and bitmap(d) == draft, 'fallback before termination')
        click(120, height // 2 - 40)
        key('f7' if mode == 'fallback-clean' else 'f8')
        scan(lambda d, s: pixel(d, 120, height // 2 + 28) == '101827'
             and bitmap(d) == draft, 'fallback termination preserves app')
        time.sleep(2)
        assert log().count('launcher fallback admitted') == 1
        assert 'explicit launcher retry' not in log()
    if mode == 'recover':
        # The replacement received fresh layout authority; this also checks the
        # standing placement used by later app launches follows the new layout.
        settings()
        action(0, True)
        action(1, False)
        action(2, False)
        margin, strip = 24, 320
        scan(lambda d, _: text(d, margin + (strip - 180) // 2, height - margin - 48, 'APPLIED')
             and bitmap(d) == draft, 'replacement layout authority and retained draft')
        action(2, True)
        focus()
        key('x')
        draft = bitmap(scan(lambda d, _: bitmap(d) != draft, 'input after launcher recovery'))
        click(margin + strip // 2, margin + 490)
        key('f8')
        scan(lambda d, s: s.count('deliberate launcher failure') == 2
             and pixel(d, 100, 70) == '101827' and bitmap(d) == draft, 'second crash leaves app alive')
        focus()  # Keep the cursor out of the fallback's glyph scan.
        for attempt in (1, 2):
            left = margin + (strip - 160) // 2
            scan(lambda d, _: text(d, left + 32, height // 2 - 40, 'LAUNCHER')
                 and text(d, left + 26, height // 2 + 21, 'TRY AGAIN') and bitmap(d) == draft,
                 'fallback stays within launcher view; app draft retained')
            shutil.copyfile(output / 'display.png', output / f'fallback-{attempt}.png')
            if attempt == 1:
                click(margin + strip // 2, height // 2 + 28)
            else:
                click(margin + strip // 2, height // 2 - 40)
                key('ret')
            expected += 1
            scan(lambda d, s: s.count('explicit launcher retry') == attempt
                 and s.count('launcher recovered; retry consumed') == expected
                 and pixel(d, margin + strip // 2 - 20, margin + 46) == 'd9e4ef'
                 and bitmap(d) == draft, 'explicit retry restores launcher without restarting Notes')
            if attempt == 1:
                click(margin + strip // 2, margin + 490)
                key('f8')
                focus()
                scan(lambda d, s: s.count('deliberate launcher failure') == 3
                     and text(d, left + 32, height // 2 - 40, 'LAUNCHER'), 'manual retry does not replenish automatic retries')
    focus()
    key('y')
    draft = bitmap(scan(lambda d, _: bitmap(d) != draft, 'app input without launcher'))
    click(width - margin - 14, margin + 10)
    scan(lambda d, _: text(d, margin + strip + 26, margin + 20, 'UNSAVED NOTE'), 'close still protects draft')
    click(margin + strip + 46, margin + 76)  # Cancel.
    focus()
    scan(lambda d, _: bitmap(d) == draft, 'cancel preserved draft')
    key('f1')
    scan(lambda d, _: header(d) == '27664a', 'app can save without launcher')
    assert log().count('note-loaded') == 1
    time.sleep(3)
    assert log().count('launcher recovered; retry consumed') == expected, 'unexpected additional retry'
    if not restores:
        assert 'launcher fallback admitted' not in log(), 'explicit stop/clean exit/revocation must not launch fallback'
    click(width - margin - 14, margin + 10)
    scan(lambda d, _: header(d) == '101827', 'app-owned close without launcher')
    return draft


def run(source, selected=None):
    before = source.read_bytes()
    output = harness.BUILD / 'launcher-recovery' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    # Same composition, separate ESP: do not replace a running F5 guest's boot files.
    profile = output / 'launcher-recovery.json'
    profile.write_text((harness.WORKSPACE / 'distribution/profile.json').read_text(), encoding='utf-8', newline='\n')
    subprocess.run([sys.executable, str(Path(__file__).with_name('run.py')), '--release', '--build-only',
                    '--profile', str(profile)], cwd=harness.REPO, check=True)
    images = {}
    for feature in ('recovery-probes', 'launcher-startup-fault'):
        subprocess.run(['cargo', 'build', '--locked', '--release', '--target', 'x86_64-unknown-none',
                        '--target-dir', str(harness.BUILD / 'cargo'), '--package', 'cathedral-chooser',
                        '--features', feature], cwd=harness.WORKSPACE, check=True)
        images[feature] = output / (feature + '.elf')
        elf_trim.trim(harness.BUILD / 'cargo/x86_64-unknown-none/release/cathedral-chooser', images[feature], None)
    oracle = database_image.oracle()
    programs = harness.seeded_programs(profile, True)
    modes = ('recover', 'clean', 'startup', 'owner-stop', 'owner-revoke', 'fallback-clean', 'fallback-fault')
    if selected is not None:
        modes = tuple(selected)
    for mode in modes:
        folder = output / mode
        folder.mkdir()
        esp = harness.BUILD / 'launcher-recovery-interactive/esp'
        if mode.startswith('fallback-'):
            composition = json.loads(profile.read_text())
            composition['startup']['executables'][0]['features'] = ['fallback-probes']
            fallback_profile = folder / 'launcher-fallback-probes.json'
            fallback_profile.write_text(json.dumps(composition), encoding='utf-8', newline='\n')
            subprocess.run([sys.executable, str(Path(__file__).with_name('run.py')), '--release', '--build-only',
                            '--profile', str(fallback_profile)], cwd=harness.REPO, check=True)
            esp = harness.BUILD / 'launcher-fallback-probes-interactive/esp'
        if mode.startswith('owner-'):
            composition = json.loads(profile.read_text())
            composition['startup']['initial']['features'].append('launcher-' + mode[6:] + '-probe')
            owner_profile = folder / ('launcher-' + mode + '.json')
            owner_profile.write_text(json.dumps(composition), encoding='utf-8', newline='\n')
            subprocess.run([sys.executable, str(Path(__file__).with_name('run.py')), '--release', '--build-only',
                            '--profile', str(owner_profile)], cwd=harness.REPO, check=True)
            esp = harness.BUILD / (owner_profile.stem + '-interactive/esp')
        shutil.copytree(esp, folder / 'esp')
        image = folder / 'guest.raw'
        shutil.copyfile(source, image)
        database_image.prepare(image, programs, harness.WORKSPACE / 'distribution/migrations/readable-labels.txt', True)
        subprocess.run([str(oracle), 'engines', 'volume', 'failure-fixture', str(image),
                        str(database_image.OFFSET), str(database_image.VOLUME_BYTES), 'launcher',
                        str(images['launcher-startup-fault' if mode == 'startup' else 'recovery-probes'])], check=True)
        qemu = harness.find_qemu()
        command = [qemu, '-machine', 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1', '-m', '128']
        command += harness.firmware_args(qemu, folder)
        command += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none', '-no-reboot', '-display', 'none']
        command += disk_image.arguments(image)
        saved = [None]
        def verify(stream, folder, serial):
            saved[0] = exercise(stream, folder, serial, mode, saved[0])
        for boot in range(2 if mode == 'recover' else 1):
            capture.run(command, folder, 180, subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0,
                        'Cathedral desktop: ready; choose a user', verify)
            for name in ('serial.log', 'display.png'):
                shutil.copyfile(folder / name, folder / f'boot-{boot + 1}-{name}')
    assert source.read_bytes() == before, 'source fixture changed'
    (output / 'result.json').write_text(json.dumps({'source_unchanged': True, 'modes': modes}), encoding='utf-8')
    print(f'PASS: configured launcher recovery modes={modes}; source unchanged; artifacts: {output}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('image', type=Path, help='one-user desktop fixture; only copies are changed')
    parser.add_argument('--mode', nargs='+', choices=('recover', 'clean', 'startup', 'owner-stop', 'owner-revoke', 'fallback-clean', 'fallback-fault'))
    args = parser.parse_args()
    run(args.image, args.mode)
