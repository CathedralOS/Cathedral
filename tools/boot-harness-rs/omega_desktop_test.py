"""Boot the stock desktop supervisor over the standalone Omega stock matrix as
the root and as the user template, and check the login, the user matrix's
own configuration, and its display, input, children, presentation and
storage paths through the real display, input and storage services.

The composition is the stock desktop's with the root matrix launch and the
user template, object three of the stock realm, both replaced by the Omega
program; the supervisor is the ordinary `shell` build, so every word of F5's
login and user entry reaches the Omega root unchanged. The proof reads the
supervisor's and the services' startup lines, sees the picker offer a new
user, creates one and enters it, sees the root admit the user's copy of the
template nested over its realm and that matrix configure itself from the
realm's saved record, sees its launcher paint its tiles along the bottom,
clicks the Colors tile so the launcher launches Colors into the workspace,
sees Colors paint its view, its indicator and its checker image through both
matrices, moves the pointer over QMP and watches the compositor's cursor
follow it, then clicks Colors for focus and presses Enter, which saves a
record through both storage relays and repaints the view in the saved
colour."""
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
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
    'Cathedral desktop: interactive startup ready',
    'Cathedral: object storage ready',
    'Cathedral matrix: shared storage connection ready',
    'Cathedral desktop: storage startup accepted',
    'Cathedral desktop: ready; choose a user',
)
# The user matrix's own lines once the picker's choice admitted it.
USER_MARKERS = (
    'Cathedral desktop: picker stopped; loading user launcher',
    'Cathedral matrix: shared parent interaction ready',
    'Cathedral matrix: shared parent storage ready',
    'Cathedral matrix: configured from realm paths',
    'Cathedral desktop: user matrix furnished',
)
# The stock configuration: an 8 px margin, an 80 px launcher bar and an 8 px
# gap, so the workspace starts at (8, 8) and the bar's tiles sit 16 px below
# its top edge, 56 px apart.
MARGIN, BAR, GAP = 8, 80, 8
APP = (MARGIN, MARGIN)
# Colors fills the workspace: its indicator sits at (30, 40) 40x18 and its
# checker image at (180, 110) 32x32 inside its view.
INITIAL, SAVED = '315ca8', 'd18b35'
HEALTHY = 'e5e7eb'
CHECKER = ('ffcc44', '7755dd')
BACKDROP = '101827'
TILE_CARD = '202d43'
FONT = bytes.fromhex((ROOT / 'source-rs/platform/libraries/bitmap-font/font.hex').read_text())


def compose(output, rust_user=False):
    composition = json.loads((ROOT / 'source-rs/distribution/profile.json').read_text(encoding='utf-8'))
    composition['name'] = 'Cathedral desktop over the Omega stock matrix'
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
    # program takes the template's place, so a user made from it runs the
    # Omega matrix over the user's realm, which configures itself from the
    # Matrix record the installer puts beside the launcher.
    programs = composition['startup']['programs']
    template = programs[2]
    assert template.get('package') == 'cathedral-stock-matrix' and template.get('inside') == 1, 'object three is the user template'
    if rust_user:
        # The unprofiled template asks for the owner's entire large offer.
        # Use the existing bounded desktop declaration beneath the Omega root.
        template.setdefault('features', []).append('desktop-host')
    else:
        programs[2] = {'package': template['package'], 'entry': 'distribution/matrix/native/main.omg',
                       'target': template['target'], 'inside': template['inside'],
                       'display_title': template['display_title']}
    # Every stock matrix F5 installs in the realm, the nested desktops and
    # their user templates, is the Omega program too, so the store shares one
    # image's chunks between them as the swapped distribution will.
    realm = composition['startup']['realm_programs']
    for index, entry in enumerate(realm):
        if not rust_user and entry.get('package') == 'cathedral-stock-matrix':
            realm[index] = {key: value for key, value in entry.items() if key not in ('features', 'entry')}
            realm[index]['entry'] = 'distribution/matrix/native/main.omg'
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


def glyphs_match(pixel, left, top, text):
    """Whether the chooser's white label `text` stands at (left, top)."""
    for index, character in enumerate(text):
        for row, bits in enumerate(FONT[(ord(character) - 32) * 7:(ord(character) - 31) * 7]):
            for column in range(5):
                lit = pixel(left + index * 12 + column * 2, top + row * 2) == 'ffffff'
                if lit != bool(bits & (1 << (4 - column))):
                    return False
    return True


def failed_children(text):
    """Unexpected child exits or a matrix's own failure during cleanup."""
    failures = []
    for match in re.finditer(r'Cathedral matrix: child (\d) ([0-9a-f]{16}) ([0-9a-f]{16})', text):
        kind, value = int(match.group(2), 16), int(match.group(3), 16)
        if kind != 3 and not (kind == 1 and value == 0):
            failures.append(match.group(0))
    # Record the matrix's failure even if its owner has not collected/reported
    # the exit yet. Native entry independently forwards Program::run's status.
    for match in re.finditer(r'Matrix control: [0-9a-f]{16} 7 [0-7] ([0-9a-f]{16})', text):
        if int(match.group(1), 16) != 0:
            failures.append(match.group(0))
    return failures


def check(stream, output, serial):
    def serial_text():
        return (output / 'serial.log').read_text(encoding='utf-8', errors='replace')

    def wait_for(marker, seconds):
        deadline = time.monotonic() + seconds
        while marker not in serial_text():
            if time.monotonic() > deadline:
                raise RuntimeError(f'missing {marker!r}; logs: {output}')
            failures = failed_children(serial_text())
            if failures or 'user panic:' in serial_text():
                raise RuntimeError(f'a program failed while waiting for {marker!r}: {failures}; logs: {output}')
            time.sleep(0.2)

    for marker in MARKERS:
        wait_for(marker, 40)
    if 'Cathedral input: absolute pointer ready' not in serial_text():
        raise RuntimeError(f'the input service did not offer an absolute pointer; logs: {output}')

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
                observed = [pixel(APP[0] + 5, APP[1] + 10), pixel(APP[0] + 35, APP[1] + 45),
                            pixel(APP[0] + 184, APP[1] + 114), pixel(APP[0] + 192, APP[1] + 114)]
                raise RuntimeError(f'{description}: view/indicator/checker {observed}; logs: {output}')
            failures = failed_children(serial_text())
            if failures:
                raise RuntimeError(f'a program failed during {description!r}: {failures}; logs: {output}')
            time.sleep(0.2)

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

    width, height, _, _ = pixels()

    # The picker centres one 128 px tile per user, 152 px apart, and labels
    # each 76 px below its top edge; a fresh store offers only a new user.
    def user_tile(index, count):
        return ((width - (count * 152 - 24)) // 2 + index * 152, (height - 96) // 2)

    def tile_label(index, count, text):
        left, top = user_tile(index, count)
        start = left + (128 - len(text) * 12) // 2
        point(8, 8)
        expect(f'the picker did not show the {text} tile', 30,
               lambda pixel: glyphs_match(pixel, start, top + 76, text))

    def click_user_tile(index, count):
        x, y = user_tile(index, count)
        click(x + 64, y + 48)

    tile_label(0, 1, 'NEW USER')
    click_user_tile(0, 1)
    tile_label(0, 2, 'USER 1')
    click_user_tile(0, 2)
    # The picker's choice admits the user's matrix, the Omega program over the
    # user's realm; the shell retires the picker, routes CONFIGURE to it and
    # the matrix configures its workspace from the realm's saved record.
    for marker in USER_MARKERS:
        wait_for(marker, 60)

    # The launcher stands as the bottom bar and lists the user's realm: the
    # Colors tile is the second card, 56 px after the Notes tile.
    bar_top = height - MARGIN - BAR
    tile = (MARGIN + 8 + 56 + 2, bar_top + 16 + 2)
    expect('the launcher did not paint its Colors tile', 30,
           lambda pixel: pixel(*tile) == TILE_CARD and pixel(4, 4) == BACKDROP)

    def painted(colour):
        return lambda pixel: (pixel(APP[0] + 5, APP[1] + 10) == colour
                              and pixel(width - MARGIN - 5, height - MARGIN - BAR - GAP - 5) == colour
                              and pixel(APP[0] + 35, APP[1] + 45) == HEALTHY
                              and pixel(APP[0] + 184, APP[1] + 114) == CHECKER[0]
                              and pixel(APP[0] + 192, APP[1] + 114) == CHECKER[1]
                              and pixel(4, 4) == BACKDROP and pixel(width // 2, height - MARGIN - BAR - 4) == BACKDROP)

    # Clicking the tile makes the launcher launch Colors through the user's
    # matrix into the workspace; Colors then reads its realm, creates its
    # record, and paints its view, a healthy indicator and the checker image.
    click(MARGIN + 8 + 56 + 24, bar_top + 16 + 24)
    width, height, before = expect('Colors not launched and painted through the user matrix', 40, painted(INITIAL))
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
        origin = (MARGIN + 8 + 56 + 24, bar_top + 16 + 24)
        far = [(x, y) for x, y in changed
               if not (0 <= x - target[0] <= 12 and 0 <= y - target[1] <= 16)
               and not (0 <= x - origin[0] <= 12 and 0 <= y - origin[1] <= 16)]
        if near and not far:
            break
        if time.monotonic() > deadline:
            raise RuntimeError(f'cursor did not follow the pointer: {len(near)} near, {far[:8]} far; logs: {output}')
    # Click inside Colors for focus, then Enter saves the toggled record
    # through both storage relays and repaints the view in the saved colour.
    click(APP[0] + 150, APP[1] + 200)
    for down in (True, False):
        capture.command(stream, 'input-send-event', {'events': [
            {'type': 'key', 'data': {'down': down, 'key': {'type': 'qcode', 'data': 'ret'}}}]})
        time.sleep(0.1)
    width, height, after = expect('Colors did not save through the user matrix', 20, painted(SAVED))
    capture.save_png(output, width, height, after)
    text = serial_text()
    failures = failed_children(text)
    if 'user panic:' in text or 'initial program stopped' in text or failures:
        raise RuntimeError(f'a program failed after admission: {failures}; logs: {output}')
    print(f'PASS: Omega stock matrix as the F5 root and user template: a new user entered through the picker, '
          f'the user matrix configured itself, and its launcher launched Colors, which paints and saves through both; logs: {output}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--profile-only', action='store_true',
                        help='write the composition and print its path, for desktop_test.py --profile to measure F5 parity')
    parser.add_argument('--rust-user', action='store_true',
                        help='keep the Rust user/nested matrices with their desktop resource declaration; test the reverse IPC boundary')
    args = parser.parse_args()
    output = harness.BUILD / 'omega-desktop' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    profile = compose(output, rust_user=args.rust_user)
    if args.profile_only:
        print(profile)
        return
    if args.rust_user:
        import desktop_test
        desktop_test.run(240, source_profile=profile)
        return
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
    capture.run(command, output, 200, flags, ready_marker='Cathedral desktop: interactive startup ready', verify=check)


if __name__ == '__main__':
    main()
