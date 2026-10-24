"""Launch the installed Omega UI in the ordinary desktop and use physical input.

The disk and ESP are disposable. Existing saved desktops are never opened.
"""
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools/boot-harness-rs'))
import capture
import database_image
import desktop_test
import disk_image
import pointer_test
import run as harness


def exercise(stream, output, click, key, scan, width, height, reboot):
    def serial():
        return (output / 'serial.log').read_text(errors='replace').replace('Cathedral Rust lab: ', '')

    def marker(value):
        scan(lambda _width, _pixels: value in serial(), value)

    def point(x, y):
        capture.command(stream, 'input-send-event', {'events': [
            {'type': 'abs', 'data': {'axis': 'x', 'value': round(x * 32767 / (width - 1))}},
            {'type': 'abs', 'data': {'axis': 'y', 'value': round(y * 32767 / (height - 1))}},
        ]})

    def button(down):
        capture.command(stream, 'input-send-event', {'events': [
            {'type': 'btn', 'data': {'button': 'left', 'down': down}},
        ]})

    # The fifth ordinary launcher entry follows Notes, Colors, Split and Nested.
    click(256, height - 48)
    marker('Omega UI surface ready')
    point(2, 2)
    _, _, initial = pointer_test.read_scanout(stream, output)
    shutil.copyfile(output / 'display.png', output / ('cold-dark.png' if reboot else 'omega-ui-dark.png'))
    old_background = initial[(25 * width + 25) * 3:(25 * width + 25) * 3 + 3]
    left = (width - 16 - 360) // 2 + 8
    top = (height - 104 - 336) // 2 + 8
    click(left + 52, top + 100)
    marker('Omega UI button applied')
    scan(lambda w, pixels: pixels[(25 * w + 25) * 3:(25 * w + 25) * 3 + 3] != old_background,
         'compiled theme button changed displayed pixels')
    point(left + 70, top + 185)
    button(True)
    for x in (100, 140, 180, 220, 260):
        point(left + x, top + 185)
        time.sleep(0.08)
    button(False)
    marker('Omega UI slider above 65')
    click(left + 305, top + 148)
    marker('Omega UI switch applied')
    scan(lambda w, pixels: pixels[((top + 185) * w + left + 200) * 3:((top + 185) * w + left + 200) * 3 + 3]
         == pixels[(25 * w + 25) * 3:(25 * w + 25) * 3 + 3],
         'conditional view retired the slider image atomically')
    switch_count = serial().count('Omega UI switch applied')
    click(left + 305, top + 148)
    scan(lambda _w, _pixels: serial().count('Omega UI switch applied') > switch_count,
         'conditional view restored its slider through the reusable presenter')
    click(left + 46, top + 270)
    for character in 'omega':
        key(character)
    marker('Omega UI text: omega')
    key('left')
    key('backspace')
    key('x')
    marker('Omega UI text: omexa')
    point(2, 2)
    scan(lambda _width, _pixels: True, 'final compiled surface')
    shutil.copyfile(output / 'display.png', output / ('cold-ui.png' if reboot else 'omega-ui.png'))
    click(left + 280, top + 100)
    marker('Omega UI field cleared')
    click(left + 179, top + 100)
    marker('Omega UI controls reset')
    # The component's shared ghost button closes this instance.
    click(left + 312, top + 48)
    ready = serial().count('Omega UI surface ready')
    click(256, height - 48)
    scan(lambda _width, _pixels: serial().count('Omega UI surface ready') > ready,
         'closed native surface reopened through the launcher')
    print('PASS: installed modern native buttons, animated switch, slider, text editing, clear/reset and close/reopen', flush=True)


def main():
    output = harness.BUILD / 'omega-ui' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    profile = output / 'omega-ui-desktop.json'
    composition = json.loads((ROOT / 'source-rs/distribution/profile.json').read_text())
    for program in composition['startup']['realm_programs']:
        if program.get('package') == 'cathedral-omega-ui-demo':
            program['features'] = ['trace-input']
    profile.write_text(json.dumps(composition, indent=2))
    subprocess.run([sys.executable, str(ROOT / 'tools/boot-harness-rs/run.py'),
                    '--release', '--build-only', '--profile', str(profile)], cwd=ROOT, check=True)
    shutil.copytree(harness.BUILD / f'{profile.stem}-interactive/esp', output / 'esp')
    disk = output / 'guest.raw'
    database_image.prepare(disk, harness.seeded_programs(profile, True),
                           ROOT / 'source-rs/distribution/migrations/readable-labels.txt',
                           app_close=True, realm_programs=harness.realm_programs(profile, True))
    qemu = harness.find_qemu()
    command = [qemu, '-machine', 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1', '-m', '128']
    command += harness.firmware_args(qemu, output)
    command += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none', '-no-reboot', '-display', 'none']
    command += disk_image.arguments(disk)
    for boot in (1, 2):
        capture.run(command, output, 120, subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0,
                    'Cathedral desktop: ready; choose a user',
                    lambda stream, output, serial: desktop_test.desktop(
                        stream, output, serial, boot == 2,
                        user_check=lambda *arguments: exercise(stream, *arguments)))
        shutil.copyfile(output / 'serial.log', output / f'boot-{boot}-serial.log')
    print(f'Guest evidence: {output}', flush=True)


if __name__ == '__main__':
    main()
