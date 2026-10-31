"""Boot the desktop supervisor over the standalone Omega stock matrix and check
its display and source paths through the real display and input services.

The composition is the stock desktop's with the root matrix launch replaced by
the Omega program and the supervisor built with `omega-root`, which admits the
root as F5 does and then parks instead of starting storage. The proof reads
the supervisor's and the drivers' startup lines, then moves the pointer over
QMP and watches the compositor's cursor follow it through the matrix."""
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

import capture
import pointer_test
import run as harness

ROOT = harness.REPO
MARKERS = (
    'Cathedral desktop: shared root control ready',
    'Cathedral matrix: shared display connection ready',
    'Cathedral input: shared source authority checked',
    'Cathedral input: shared source connection ready',
    'Cathedral desktop: shared input control ready',
    'Cathedral desktop: bootstrap accepted',
)


def compose(output):
    composition = json.loads((ROOT / 'source-rs/distribution/profile.json').read_text(encoding='utf-8'))
    composition['name'] = 'Cathedral desktop over the Omega stock matrix'
    composition['startup']['initial']['features'].append('omega-root')
    launches = composition['startup']['launches']
    replaced = 0
    for index, launch in enumerate(launches):
        if launch.get('package') == 'cathedral-stock-matrix':
            launches[index] = {'target': launch['target'], 'clock': True,
                               'entry': 'distribution/matrix/native/main.omg',
                               'ipc_channels': launch['ipc_channels'],
                               'private_pages': launch['private_pages'],
                               'shared_pages': launch['shared_pages']}
            replaced += 1
    assert replaced == 1, 'the stock composition launches one root matrix'
    # The catalogue's second executable: the Omega background the root runs
    # into its background slot during the proof.
    executables = composition['startup']['executables']
    assert len(executables) == 1, 'the shell catalogue carries only the recovery screen'
    executables.append({'entry': 'distribution/applications/background/native/main.omg',
                        'target': 'x86_64-unknown-none'})
    profile = output / f'omega-desktop-{output.name}.json'
    profile.write_text(json.dumps(composition, indent=2), encoding='utf-8', newline='\n')
    return profile


def changed_pixels(width, before, after):
    stride = width * 3
    for index in range(0, len(before), stride):
        if before[index:index + stride] != after[index:index + stride]:
            y = index // stride
            for x in range(width):
                at = index + x * 3
                if before[at:at + 3] != after[at:at + 3]:
                    yield x, y


def check(stream, output, serial):
    for marker in MARKERS:
        if marker not in serial:
            raise RuntimeError(f'missing {marker!r}; logs: {output}')
    if 'Cathedral input: absolute pointer ready' not in serial:
        raise RuntimeError(f'the input service did not offer an absolute pointer; logs: {output}')
    # The background child paints its whole view, placed at (200,150) 400x300,
    # in its own colour over the proof's shifted backdrop.
    deadline = time.monotonic() + 20
    while 'Cathedral desktop: Omega root background placed' not in (output / 'serial.log').read_text(encoding='utf-8', errors='replace'):
        if time.monotonic() > deadline:
            raise RuntimeError(f'the root never placed the background child; logs: {output}')
        time.sleep(0.2)
    deadline = time.monotonic() + 15
    while True:
        width, height, pixels = pointer_test.read_scanout(stream, output)
        def pixel(x, y):
            return pixels[(y * width + x) * 3:(y * width + x) * 3 + 3].hex()
        inside = [pixel(200, 150), pixel(399, 299), pixel(599, 449)]
        outside = [pixel(100, 100), pixel(620, 460), pixel(900, 700)]
        if all(p == '101827' for p in inside) and all(p == '303030' for p in outside):
            break
        if time.monotonic() > deadline:
            raise RuntimeError(f'background child not visible: inside {inside}, outside {outside}; logs: {output}')
        time.sleep(0.2)
    width, height, before = pointer_test.read_scanout(stream, output)
    target = (600, 400)
    capture.command(stream, 'input-send-event', {'events': [
        {'type': 'abs', 'data': {'axis': 'x', 'value': round(target[0] * 32767 / (width - 1))}},
        {'type': 'abs', 'data': {'axis': 'y', 'value': round(target[1] * 32767 / (height - 1))}}]})
    deadline = time.monotonic() + 10
    while True:
        time.sleep(0.2)
        _, _, after = pointer_test.read_scanout(stream, output)
        changed = list(changed_pixels(width, before, after))
        near = [(x, y) for x, y in changed if 0 <= x - target[0] <= 12 and 0 <= y - target[1] <= 16]
        far = [(x, y) for x, y in changed
               if not (0 <= x - target[0] <= 12 and 0 <= y - target[1] <= 16)
               and not (0 <= x - 40 <= 12 and 0 <= y - 40 <= 16)]
        if near and not far:
            break
        if time.monotonic() > deadline:
            raise RuntimeError(f'cursor did not follow the pointer: {len(near)} near, {far[:8]} far; logs: {output}')
    capture.save_png(output, width, height, after)
    text = (output / 'serial.log').read_text(encoding='utf-8', errors='replace')
    if 'user panic:' in text or 'initial program stopped' in text:
        raise RuntimeError(f'a program failed after admission; logs: {output}')
    print(f'PASS: Omega stock matrix admitted at the display root, source and cursor live; logs: {output}')


def main():
    output = harness.BUILD / 'omega-desktop' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    profile = compose(output)
    subprocess.run([sys.executable, str(Path(__file__).with_name('run.py')),
                    '--profile', str(profile), '--release', '--build-only'], cwd=ROOT, check=True)
    shutil.copytree(harness.BUILD / f'{profile.stem}-interactive/esp', output / 'esp')
    qemu = harness.find_qemu()
    command = [qemu, '-machine', 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1', '-m', '128']
    command += harness.firmware_args(qemu, output)
    command += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none', '-no-reboot', '-display', 'none']
    flags = subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0
    capture.run(command, output, 120, flags, ready_marker='Cathedral desktop: Omega root admitted', verify=check)


if __name__ == '__main__':
    main()
