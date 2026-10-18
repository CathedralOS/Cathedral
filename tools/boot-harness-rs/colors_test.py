"""Launch Colors first on a copy of an existing desktop disk, save and cold-open.

Retained images matter: accumulated upgrade garbage exposed a collection stall
that a newly formatted desktop never exercised. The supplied image is untouched.
"""
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


def exercise(stream, output, serial, users, user, saved, recovery=False):
    assert 'Cathedral input: absolute pointer ready' in serial
    width, height, _ = pointer_test.read_scanout(stream, output)

    def send(events):
        capture.command(stream, 'input-send-event', {'events': events})

    def click(x, y):
        send([{'type': 'abs', 'data': {'axis': 'x', 'value': round(x * 32767 / (width - 1))}},
              {'type': 'abs', 'data': {'axis': 'y', 'value': round(y * 32767 / (height - 1))}}])
        time.sleep(0.05)
        for down in (True, False):
            send([{'type': 'btn', 'data': {'button': 'left', 'down': down}}])
            time.sleep(0.08)

    def pixel(data, x, y):
        at = (y * width + x) * 3
        return data[at:at + 3].hex()

    def scan(check, description):
        start = time.monotonic()
        while True:
            log = (output / 'serial.log').read_text(errors='replace')
            failures = ['user panic:', 'CATHEDRAL_RS_PANIC', 'initial program stopped', 'storage offline']
            if not recovery:
                failures += ['stalled storage', 'storage replaced']
            for failure in failures:
                assert failure not in log, log[-3000:]
            w, h, data = pointer_test.read_scanout(stream, output)
            if check(data):
                capture.save_png(output, w, h, data)
                return data, time.monotonic() - start
            if time.monotonic() - start >= 15:
                capture.save_png(output, w, h, data)
                raise AssertionError(f'{description}; screenshot/logs: {output}')
            time.sleep(0.05)

    def enter():
        click(237, 144)
        for down in (True, False):
            send([{'type': 'key', 'data': {'down': down, 'key': {'type': 'qcode', 'data': 'ret'}}}])
            time.sleep(0.08)

    def reconcile():
        deadline = time.monotonic() + 45
        while True:
            log = (output / 'serial.log').read_text(errors='replace')
            assert 'user panic:' not in log and 'storage offline' not in log, log[-3000:]
            if 'storage replaced; explicit grants restored' in log:
                break
            assert time.monotonic() < deadline, log[-3000:]
            time.sleep(0.05)
        enter()  # Reload the committed state after its acknowledgement was lost.
        scan(lambda d: pixel(d, 264, 66) == 'e5e7eb', 'reconciled storage')

    cold = saved is not None
    count = min(users + 1, 3)
    click((width - (count * 152 - 24)) // 2 + (user - 1) * 152 + 64, height // 2)
    scan(lambda d: pixel(d, 100, 70) == 'd9e4ef', 'user app strip')
    click(120, 216)  # Colors, before any Notes save can drain the garbage.
    data, latency = scan(lambda d: pixel(d, 237, 34) in ('315ca8', 'd18b35')
                        and (recovery or pixel(d, 264, 66) == 'e5e7eb'), 'Colors launch')
    if recovery and pixel(data, 264, 66) == 'e96f6f':
        reconcile()
        data = pointer_test.read_scanout(stream, output)[2]
    current = pixel(data, 237, 34)
    if saved is not None:
        assert current == saved, 'cold-open lost the saved color'
    else:
        enter()
        saved = 'd18b35' if current == '315ca8' else '315ca8'
        if recovery:
            data, _ = scan(lambda d: pixel(d, 237, 34) == saved or pixel(d, 264, 66) == 'e96f6f', 'toggle result')
            if pixel(data, 264, 66) == 'e96f6f':
                reconcile()
        scan(lambda d: pixel(d, 237, 34) == saved and pixel(d, 264, 66) == 'e5e7eb', 'saved toggle')
    click(width - 38, 34)
    scan(lambda d: pixel(d, 237, 34) == '101827', 'clean app-owned close')
    click(120, 216)
    scan(lambda d: pixel(d, 237, 34) == saved and pixel(d, 264, 66) == 'e5e7eb', 'reopened color')
    log = (output / 'serial.log').read_text(errors='replace')
    assert log.count('picker stopped; loading user launcher') == 1, 'storage failure restarted login'
    if recovery:
        assert log.count('storage replaced; explicit grants restored') == (0 if cold else 1), log[-3000:]
    return saved, latency


def run(args):
    root = harness.REPO
    output = harness.BUILD / 'colors' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    profile = root / 'source-rs/distribution/profile.json'
    build = [sys.executable, str(Path(__file__).with_name('run.py')), '--release', '--build-only']
    esp = harness.BUILD / 'interactive/esp'
    if args.recovery:
        composition = json.loads(profile.read_text())
        startup = composition['startup']
        for program in [startup['initial'], *startup['programs']]:
            if program.get('package') == 'cathedral-desktop':
                program['features'].append('notes-probes')
        startup['launches'][3]['features'] = ['recovery-lab']
        profile = output / 'colors-recovery.json'
        profile.write_text(json.dumps(composition, indent=2), encoding='utf-8')
        build += ['--profile', str(profile)]
        esp = harness.BUILD / 'colors-recovery-interactive/esp'
    subprocess.run(build, cwd=root, check=True)
    shutil.copytree(esp, output / 'esp')
    path = output / 'guest.raw'
    shutil.copyfile(args.image, path)
    database_image.prepare(path, harness.seeded_programs(profile, True),
                           root / 'source-rs/distribution/migrations/readable-labels.txt', app_close=True)
    qemu = harness.find_qemu()
    command = [qemu, '-machine', 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1', '-m', '128']
    command += harness.firmware_args(qemu, output)
    command += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none',
                '-no-reboot', '-display', 'none'] + disk_image.arguments(path)
    result = [None]
    latencies = []

    def verify(stream, output, serial):
        result[0], elapsed = exercise(stream, output, serial, args.users, args.user, result[0], args.recovery)
        latencies.append(elapsed)

    for boot in (1, 2):
        capture.run(command, output, 180, subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0,
                    'Cathedral desktop: ready; choose a user', verify)
        for name in ('serial.log', 'display.png'):
            shutil.copyfile(output / name, output / f'boot-{boot}-{name}')
    (output / 'result.json').write_text(json.dumps({'image': str(args.image.resolve()),
        'user': args.user, 'recovery': args.recovery, 'launch_seconds_after_click': latencies,
        'saved_color': result[0]}, indent=2), encoding='utf-8')
    print(f'PASS: Colors first launch, save, close, reopen and cold boot; launch seconds {latencies}; artifacts: {output}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('image', type=Path)
    parser.add_argument('--users', type=int, choices=(1, 2, 3), default=1)
    parser.add_argument('--user', type=int, choices=(1, 2, 3), default=1)
    parser.add_argument('--recovery', action='store_true', help='lose one committed save reply; keep the user alive and reconcile')
    args = parser.parse_args()
    if args.user > args.users:
        parser.error('selected user must exist')
    run(args)
