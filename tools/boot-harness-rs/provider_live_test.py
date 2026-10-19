"""Lose live storage under the ordinary F5 desktop; retain drafts on copied media."""
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
import pointer_test
import run as harness

CASES = ('precommit', 'postcommit', 'exhausted', 'readiness-stall', 'nested')


def exercise(stream, folder, mode, saved=None):
    nested = mode == 'nested'
    offset_y = 32 if nested else 0
    width, height, _ = pointer_test.read_scanout(stream, folder)
    font = bytes.fromhex((harness.WORKSPACE / 'platform/libraries/bitmap-font/font.hex').read_text())

    def log():
        value = (folder / 'serial.log').read_text(errors='replace')
        assert all(s not in value for s in ('user panic:', 'CATHEDRAL_RS_PANIC', 'initial program stopped')), value[-3000:]
        return value

    def pixel(data, x, y):
        return data[(y * width + x) * 3:(y * width + x) * 3 + 3].hex()

    def bitmap(data):
        return b''.join(data[((44 + offset_y + y) * width + 242) * 3:((44 + offset_y + y) * width + 422) * 3] for y in range(64))

    def header(data):
        return pixel(data, 237, 34 + offset_y)

    def text(data, value, y):
        for i, ch in enumerate(value):
            for row, bits in enumerate(font[(ord(ch) - 32) * 7:(ord(ch) - 31) * 7]):
                for col in range(5):
                    if (pixel(data, 242 + i * 12 + col * 2, y + row * 2) == 'ffffff') != bool(bits & (1 << (4 - col))):
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
            time.sleep(0.08)

    def click(x, y):
        capture.command(stream, 'input-send-event', {'events': [
            {'type': 'abs', 'data': {'axis': 'x', 'value': round(x * 32767 / (width - 1))}},
            {'type': 'abs', 'data': {'axis': 'y', 'value': round(y * 32767 / (height - 1))}}]})
        time.sleep(0.08)
        for down in (True, False):
            capture.command(stream, 'input-send-event', {'events': [
                {'type': 'btn', 'data': {'button': 'left', 'down': down}}]})
            time.sleep(0.1)

    def key(code):
        for down in (True, False):
            capture.command(stream, 'input-send-event', {'events': [
                {'type': 'key', 'data': {'down': down, 'key': {'type': 'qcode', 'data': code}}}]})
            time.sleep(0.08)

    def focus():
        click(448, 154)

    def cancel_close(draft):
        click(width - 38, 34)
        scan(lambda d, _: text(d, 'SAVE ERROR', 44), 'failed save protects app close')
        click(278, 100)
        focus()
        scan(lambda d, _: bitmap(d) == draft and header(d) == '993f46', 'Cancel retains draft')

    click((width - 280) // 2 + 64, height // 2)
    scan(lambda d, _: pixel(d, 100, 70) == 'd9e4ef', 'user launcher')
    click(120, 336 if nested else 96)
    scan(lambda d, _: header(d) == '27664a', 'Notes loaded')
    focus()
    def sibling(data):
        x = 232 + (width - 24 - 232 - 12) // 2 + 12 + 10
        return b''.join(data[((76 + y) * width + x) * 3:((76 + y) * width + x + 180) * 3] for y in range(64))
    initial = bitmap(scan(lambda d, _: header(d) == '27664a', 'initial saved note'))
    if saved is not None:
        assert initial == saved, 'cold-open contents differ'
        assert log().count('note-loaded') == (2 if nested else 1)
        return saved

    right = sibling(pointer_test.read_scanout(stream, folder)[2]) if nested else None
    key('z')
    draft = bitmap(scan(lambda d, _: header(d) == '9b7028', 'dirty draft'))
    attempts = 4 if mode == 'exhausted' else 1
    for attempt in range(1, attempts + 1):
        scan(lambda d, s: s.count('live storage probe armed') == attempt and bitmap(d) == draft,
             'owner armed current provider after dirty draft')
        key('f1')
        scan(lambda d, _: header(d) == '993f46' and bitmap(d) == draft, 'failed acknowledgement retains draft')
        if mode == 'readiness-stall':
            scan(lambda d, s: 'storage replacement starting' in s and bitmap(d) == draft, 'asynchronous replacement began')
            key('x')
            draft = bitmap(scan(lambda d, _: header(d) == '993f46' and bitmap(d) != draft,
                                'input while replacement readiness is blocked'))
            cancel_close(draft)
            scan(lambda d, s: 'storage offline; replacement failed' in s and bitmap(d) == draft,
                 'replacement deadline leaves live session intact')
            assert 'storage replaced; explicit grants restored' not in log()
            break
        if mode == 'exhausted' and attempt == attempts:
            scan(lambda d, s: 'storage offline; restart budget exhausted' in s and bitmap(d) == draft,
                 'bounded restart exhaustion leaves app alive')
            break
        scan(lambda d, s: s.count('storage replaced; explicit grants restored') == attempt and bitmap(d) == draft,
             'fresh provider and explicit grants')
        key('f1')  # Reconcile; do not repeat the uncertain write.
        scan(lambda d, _: bitmap(d) == draft and header(d) == ('27664a' if mode == 'postcommit' else '9b7028'),
             'reconciliation distinguishes precommit from committed lost reply')

    offline = mode in ('exhausted', 'readiness-stall')
    if offline:
        key('x')
        draft = bitmap(scan(lambda d, _: bitmap(d) != draft and header(d) == '993f46', 'editing while offline'))
        key('f1')
        scan(lambda d, _: bitmap(d) == draft and header(d) == '993f46', 'offline save does not claim success')
        cancel_close(draft)
        # Trying Save in the close dialog must keep the process and dialog alive.
        click(width - 38, 34)
        scan(lambda d, _: text(d, 'SAVE ERROR', 44), 'close warning')
        click(278, 68)
        scan(lambda d, _: text(d, 'SAVE ERROR', 44), 'failed close-save keeps dialog')
        click(278, 100)
        focus()
        scan(lambda d, _: bitmap(d) == draft, 'draft after failed close-save')
        time.sleep(2)
        assert log().count('storage replacement starting') == (3 if mode == 'exhausted' else 1)
        # Only the configured launcher can request a single new attempt. This
        # goes through the real visible action and leaves the app process intact.
        click(80, height - 65)
        scan(lambda d, s: s.count('explicit save retry') == 1 and bitmap(d) == draft,
             'explicit retry accepted without replacing Notes')
        if mode == 'exhausted':
            scan(lambda d, s: s.count('storage replaced; explicit grants restored') == 4 and bitmap(d) == draft,
                 'explicit retry restores storage after budget exhaustion')
            focus(); key('f1')
            scan(lambda d, _: bitmap(d) == draft and header(d) == '9b7028', 'retry reconciles the retained draft')
            key('f1')
            scan(lambda d, _: bitmap(d) == draft and header(d) == '27664a', 'retry saves the same retained draft')
            expected = draft
        else:
            scan(lambda d, s: s.count('storage offline; replacement failed') == 2 and bitmap(d) == draft,
                 'failed explicit attempt stays bounded and retains draft')
            time.sleep(2)
            assert log().count('storage replacement starting') == 2
            expected = initial  # Both attempts failed before becoming ready.
    else:
        key('x')
        draft = bitmap(scan(lambda d, _: bitmap(d) != draft and header(d) == '9b7028', 'editing after recovery'))
        key('f1')
        scan(lambda d, _: bitmap(d) == draft and header(d) == '27664a', 'save after reconciliation')
        if nested:
            assert sibling(pointer_test.read_scanout(stream, folder)[2]) == right, 'sibling contents changed'
        click(width - 38, 34)
        scan(lambda d, _: header(d) == '101827', 'saved app closes')
        expected = draft
    assert log().count('note-loaded') == (2 if nested else 1), 'Notes task was restarted'
    assert 'user matrix closed' not in log() and 'sign-in recovery visible' not in log()
    return expected


def run(source, cases):
    original = source.read_bytes()
    output = harness.BUILD / 'provider-live' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    for mode in cases:
        folder = output / mode
        folder.mkdir()
        composition = json.loads((harness.WORKSPACE / 'distribution/profile.json').read_text())
        feature = {'precommit': 'storage-live-probes', 'postcommit': 'storage-live-postcommit',
                   'exhausted': 'storage-live-repeat', 'readiness-stall': 'storage-live-readiness-stall', 'nested': 'storage-live-nested'}[mode]
        composition['startup']['initial']['features'].append(feature)
        composition['startup']['launches'][3]['features'] = ['recovery-lab', 'startup-probes', 'trace-storage']
        profile = folder / 'provider-live.json'
        profile.write_text(json.dumps(composition, indent=2), encoding='utf-8', newline='\n')
        subprocess.run([sys.executable, str(Path(__file__).with_name('run.py')), '--release', '--build-only',
                        '--profile', str(profile)], cwd=harness.REPO, check=True)
        shutil.copytree(harness.BUILD / 'provider-live-interactive/esp', folder / 'esp')
        image = folder / 'guest.raw'
        image.write_bytes(original)
        database_image.prepare(image, harness.seeded_programs(profile, True),
                               harness.WORKSPACE / 'distribution/migrations/readable-labels.txt', True,
                               realm_programs=harness.realm_programs(profile, True))
        qemu = harness.find_qemu()
        command = [qemu, '-machine', 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1', '-m', '128']
        command += harness.firmware_args(qemu, folder)
        command += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none', '-no-reboot', '-display', 'none']
        command += disk_image.arguments(image)
        saved = [None]
        def verify(stream, folder, serial):
            saved[0] = exercise(stream, folder, mode, saved[0])
        for boot in (1, 2):
            capture.run(command, folder, 240, subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0,
                        'Cathedral desktop: ready; choose a user', verify)
            for name in ('serial.log', 'display.png'):
                shutil.copyfile(folder / name, folder / f'boot-{boot}-{name}')
        print(f'PASS: live storage {mode}, retained draft, close protection and cold persistence; {folder}', flush=True)
    assert source.read_bytes() == original, 'source image changed'
    (output / 'result.json').write_text(json.dumps({'cases': cases, 'source_unchanged': True}), encoding='utf-8')
    print(f'PASS: live provider suite; {output}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('image', type=Path, help='one-user default-layout fixture; only copies change')
    parser.add_argument('--case', choices=CASES, action='append')
    args = parser.parse_args()
    run(args.image, args.case or CASES)
