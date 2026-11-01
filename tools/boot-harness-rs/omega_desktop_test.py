"""Boot the desktop supervisor over the standalone Omega stock matrix, nested
once, and check its display, source, children, presentation and storage paths
through the real display, input and storage services at both levels.

The composition is the stock desktop's with the root matrix launch and the
user template, object three of the stock realm, both replaced by the Omega
program, and the supervisor built with `omega-root`, which admits the root as
F5 does, starts storage, runs the template nested over its own realm at an
inset, as a login does, and configures it through routed owner words to run
the template's launcher as a strip and open an app slot to the launcher's
launches, then parks. The proof reads the supervisor's, the services' and the
nested matrix's startup lines, sees the chooser paint its Colors tile inside
the nested area, clicks the tile so the chooser launches Colors through the
nested matrix, sees Colors paint its view, its indicator and its checker image
through both matrices, moves the pointer over QMP and watches the compositor's
cursor follow it, then clicks Colors for focus and presses Enter, which saves
a record through both storage relays and repaints the view in the saved
colour."""
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

ROOT = harness.REPO
MARKERS = (
    'Cathedral desktop: shared root control ready',
    'Cathedral matrix: shared display connection ready',
    'Cathedral input: shared source authority checked',
    'Cathedral input: shared source connection ready',
    'Cathedral desktop: shared input control ready',
    'Cathedral desktop: bootstrap accepted',
    'Cathedral: object storage ready',
    'Cathedral desktop: Omega root storage ready',
    'Cathedral matrix: shared storage connection ready',
)
# The nested matrix's own lines, once the root has run it.
NESTED_MARKERS = (
    'Cathedral desktop: Omega nested matrix placed',
    'Cathedral matrix: shared parent interaction ready',
    'Cathedral matrix: parent storage connection ready',
    'Cathedral desktop: Omega nested launcher placed',
)
# The nested matrix sits at this inset from every edge of the display, and its
# launcher strip is the bottom 96 rows of that area.
INSET = (32, 24)
STRIP = 96
# Colors is placed at (650, 100) 300x300 inside the nested matrix: its
# background fills the view, its indicator sits at (30, 40) 40x18 and its
# checker image at (180, 110) 32x32.
COLORS = (INSET[0] + 650, INSET[1] + 100)
INITIAL, SAVED = '315ca8', 'd18b35'
HEALTHY = 'e5e7eb'
CHECKER = ('ffcc44', '7755dd')


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
    # The stock realm as F5 installs it: the user template is object three,
    # with its launcher, object four, Notes and Colors beneath it. The Omega
    # program takes the template's place, so the root runs it nested over its
    # realm, it runs the launcher, the launcher launches Colors, and Colors
    # keeps its record beneath its own object.
    programs = composition['startup']['programs']
    template = programs[2]
    assert template.get('package') == 'cathedral-stock-matrix' and template.get('inside') == 1, 'object three is the user template'
    programs[2] = {'package': template['package'], 'entry': 'distribution/matrix/native/main.omg',
                   'target': template['target'], 'inside': template['inside'],
                   'display_title': template['display_title']}
    # The realm programs F5 adds beside the stock six are not run here, and
    # beside the larger Omega template they would exceed the root's quota.
    composition['startup']['realm_programs'] = []
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
    def serial_text():
        return (output / 'serial.log').read_text(encoding='utf-8', errors='replace')

    def wait_for(marker, seconds):
        deadline = time.monotonic() + seconds
        while marker not in serial_text():
            if time.monotonic() > deadline:
                raise RuntimeError(f'missing {marker!r}; logs: {output}')
            time.sleep(0.2)

    for marker in MARKERS:
        wait_for(marker, 30)
    if 'Cathedral input: absolute pointer ready' not in serial_text():
        raise RuntimeError(f'the input service did not offer an absolute pointer; logs: {output}')
    for marker in NESTED_MARKERS:
        wait_for(marker, 30)

    def pixels():
        width, height, data = pointer_test.read_scanout(stream, output)
        def pixel(x, y):
            return data[(y * width + x) * 3:(y * width + x) * 3 + 3].hex()
        return width, height, data, pixel

    def expect(description, seconds, condition):
        deadline = time.monotonic() + seconds
        while True:
            width, height, data, pixel = pixels()
            if condition(pixel):
                return width, height, data
            if time.monotonic() > deadline:
                observed = [pixel(COLORS[0] + 5, COLORS[1] + 10), pixel(COLORS[0] + 35, COLORS[1] + 45),
                            pixel(COLORS[0] + 184, COLORS[1] + 114), pixel(COLORS[0] + 192, COLORS[1] + 114)]
                raise RuntimeError(f'{description}: view/indicator/checker {observed}; logs: {output}')
            time.sleep(0.2)

    # The nested matrix paints nothing itself, so the backdrop shows through
    # it beside its children, as it does outside the inset.
    def painted(colour):
        return lambda pixel: (pixel(COLORS[0] + 5, COLORS[1] + 10) == colour
                              and pixel(COLORS[0] + 295, COLORS[1] + 295) == colour
                              and pixel(COLORS[0] + 35, COLORS[1] + 45) == HEALTHY
                              and pixel(COLORS[0] + 184, COLORS[1] + 114) == CHECKER[0]
                              and pixel(COLORS[0] + 192, COLORS[1] + 114) == CHECKER[1]
                              and pixel(100, 100) == '303030' and pixel(620, 460) == '303030'
                              and pixel(8, 8) == '303030')

    # The launcher lists the template's realm and paints Notes and Colors
    # tiles in its strip along the bottom of the nested area; the Colors card
    # colour marks it.
    width, height, _, _ = pixels()
    strip_top = height - INSET[1] - STRIP
    tile = (INSET[0] + 8 + 56 + 2, strip_top + 24 + 2)
    expect('the chooser did not paint its Colors tile', 30,
           lambda pixel: pixel(*tile) == '202d43' and pixel(100, 100) == '303030')
    # Clicking the tile makes the chooser launch Colors through the nested
    # matrix into the slot the owner opened; Colors then reads its realm (its
    # own object, empty beneath), creates its record, and paints its view, a
    # healthy indicator and the checker image over the shifted backdrop.
    def point(x, y):
        capture.command(stream, 'input-send-event', {'events': [
            {'type': 'abs', 'data': {'axis': 'x', 'value': round(x * 32767 / (width - 1))}},
            {'type': 'abs', 'data': {'axis': 'y', 'value': round(y * 32767 / (height - 1))}}]})
    def click(x, y):
        point(x, y)
        time.sleep(0.1)
        for down in (True, False):
            capture.command(stream, 'input-send-event', {'events': [
                {'type': 'btn', 'data': {'button': 'left', 'down': down}}]})
            time.sleep(0.1)
    click(INSET[0] + 8 + 56 + 24, strip_top + 24 + 24)
    width, height, before = expect('Colors not launched and painted through the Omega matrix', 30, painted(INITIAL))
    # Colors finishes its first scene a moment after its view fills; the
    # cursor check needs a frame nothing else is still drawing into.
    deadline = time.monotonic() + 10
    while True:
        time.sleep(0.3)
        _, _, after = pointer_test.read_scanout(stream, output)
        if after == before:
            break
        before = after
        if time.monotonic() > deadline:
            raise RuntimeError(f'the scene never settled after the launch; logs: {output}')
    target = (600, 400)
    point(*target)
    deadline = time.monotonic() + 10
    while True:
        time.sleep(0.2)
        _, _, after = pointer_test.read_scanout(stream, output)
        changed = list(changed_pixels(width, before, after))
        near = [(x, y) for x, y in changed if 0 <= x - target[0] <= 12 and 0 <= y - target[1] <= 16]
        origin = (INSET[0] + 8 + 56 + 24, strip_top + 24 + 24)
        far = [(x, y) for x, y in changed
               if not (0 <= x - target[0] <= 12 and 0 <= y - target[1] <= 16)
               and not (0 <= x - origin[0] <= 12 and 0 <= y - origin[1] <= 16)]
        if near and not far:
            break
        if time.monotonic() > deadline:
            raise RuntimeError(f'cursor did not follow the pointer: {len(near)} near, {far[:8]} far; logs: {output}')
    # Click inside Colors for focus, then Enter saves the toggled record
    # through the storage relay and repaints the view in the saved colour.
    click(COLORS[0] + 150, COLORS[1] + 200)
    for down in (True, False):
        capture.command(stream, 'input-send-event', {'events': [
            {'type': 'key', 'data': {'down': down, 'key': {'type': 'qcode', 'data': 'ret'}}}]})
        time.sleep(0.1)
    width, height, after = expect('Colors did not save through the Omega matrix', 20, painted(SAVED))
    capture.save_png(output, width, height, after)
    text = serial_text()
    if 'user panic:' in text or 'initial program stopped' in text or 'Cathedral matrix: child' in text:
        raise RuntimeError(f'a program failed after admission; logs: {output}')
    print(f'PASS: Omega stock matrix admitted at the display root and nested over the template realm, with the chooser launching Colors, which paints and saves through both; logs: {output}')


def main():
    output = harness.BUILD / 'omega-desktop' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    profile = compose(output)
    subprocess.run([sys.executable, str(Path(__file__).with_name('run.py')),
                    '--profile', str(profile), '--release', '--build-only'], cwd=ROOT, check=True)
    shutil.copytree(harness.BUILD / f'{profile.stem}-interactive/esp', output / 'esp')
    data = output / 'guest.raw'
    migration = ROOT / 'source-rs/distribution/migrations/readable-labels.txt'
    database_image.prepare(data, harness.seeded_programs(profile, True), migration, app_close=True,
                           realm_programs=harness.realm_programs(profile, True))
    qemu = harness.find_qemu()
    command = [qemu, '-machine', 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1', '-m', '128']
    command += harness.firmware_args(qemu, output)
    command += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none', '-no-reboot', '-display', 'none']
    command += disk_image.arguments(data)
    flags = subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0
    capture.run(command, output, 150, flags, ready_marker='Cathedral desktop: Omega root admitted', verify=check)


if __name__ == '__main__':
    main()
