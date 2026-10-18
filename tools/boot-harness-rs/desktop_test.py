#!/usr/bin/env python3
"""Exercise the ordinary F5 composition: make a user, launch its programs, edit, switch and cold-open."""
import argparse
from datetime import datetime, timezone
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

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def desktop(stream, output, serial, reboot):
    assert 'Cathedral: object storage ready' in serial
    position = [40, 40]
    font = bytes.fromhex((ROOT / 'source-rs/platform/libraries/bitmap-font/font.hex').read_text())

    def send(events):
        capture.command(stream, 'input-send-event', {'events': events})

    def key(code):
        for down in (True, False):
            send([{'type': 'key', 'data': {'down': down, 'key': {'type': 'qcode', 'data': code}}}])
            time.sleep(0.07)

    def move(x, y):
        while position != [x, y]:
            dx = max(-70, min(70, x - position[0]))
            dy = max(-70, min(70, y - position[1]))
            send([{'type': 'rel', 'data': {'axis': 'x', 'value': dx}},
                  {'type': 'rel', 'data': {'axis': 'y', 'value': dy}}])
            position[0] += dx
            position[1] += dy
            time.sleep(0.04)

    def click(x, y):
        move(x, y)
        for down in (True, False):
            send([{'type': 'btn', 'data': {'button': 'left', 'down': down}}])
            time.sleep(0.08)

    def scan(check, description):
        deadline = time.monotonic() + 15
        while True:
            serial = (output / 'serial.log').read_text(errors='replace')
            assert 'user panic:' not in serial and 'CATHEDRAL_RS_PANIC' not in serial, serial[-3000:]
            width, height, data = pointer_test.read_scanout(stream, output)
            if check(width, data):
                capture.save_png(output, width, height, data)
                return
            if time.monotonic() >= deadline:
                capture.save_png(output, width, height, data)
                raise AssertionError(f'{description}; screenshot/logs: {output}')
            time.sleep(0.05)

    def rgb(width, data, x, y):
        return data[(y * width + x) * 3:(y * width + x) * 3 + 3]

    def text_image(text, cursor=False):
        image = bytearray(bytes.fromhex('182132') * (180 * 64))
        def pixel(x, y, color):
            image[(y * 180 + x) * 3:(y * 180 + x) * 3 + 3] = bytes.fromhex(color)
        for index, ch in enumerate(text):
            for y, bits in enumerate(font[(ord(ch) - 32) * 7:(ord(ch) - 31) * 7]):
                for x in range(5):
                    if bits & (1 << (4 - x)):
                        for dy in range(2):
                            for dx in range(2):
                                pixel(index % 15 * 12 + x * 2 + dx, index // 15 * 16 + y * 2 + dy, 'ffffff')
        if cursor:
            for dx in range(10):
                pixel(len(text) % 15 * 12 + dx, len(text) // 15 * 16 + 14, '66ccff')
        return image

    # Selection is centered; entering a user gives it the whole display.
    width, height, _ = pointer_test.read_scanout(stream, output)
    PICKER = ((width - 220) // 2, (height - 180) // 2)
    APP = (232, 24)

    def note(text, dirty=False, temporary=False, error=False):
        expected = text_image(text, cursor=True)
        def check(width, data):
            actual = b''.join(data[((APP[1] + 20 + y) * width + APP[0] + 10) * 3:((APP[1] + 20 + y) * width + APP[0] + 190) * 3] for y in range(64))
            return actual == expected and rgb(width, data, APP[0] + 5, APP[1] + 10) == bytes.fromhex('993f46' if error else '9b7028' if dirty else '55447a' if temporary else '27664a')
        scan(check, f'note text/status {text!r}, dirty={dirty}')

    def rows_image(lines):
        """The chooser bitmap: 160 px wide rows of 13 glyph cells."""
        image = bytearray(bytes.fromhex('182132') * (160 * 16 * len(lines)))
        def pixel(x, y, color):
            image[(y * 160 + x) * 3:(y * 160 + x) * 3 + 3] = bytes.fromhex(color)
        for row, text in enumerate(lines):
            for index, ch in enumerate(text):
                for y, bits in enumerate(font[(ord(ch) - 32) * 7:(ord(ch) - 31) * 7]):
                    for x in range(5):
                        if bits & (1 << (4 - x)):
                            for dy in range(2):
                                for dx in range(2):
                                    pixel(index * 12 + x * 2 + dx, row * 16 + y * 2 + dy, 'ffffff')
        return image

    def chooser_row(left, origin_y, row, text, description):
        move(8, 8)  # Keep the cursor overlay outside the checked glyphs.
        expected = rows_image([text])
        top = origin_y + 36 + 16 * row
        def check(width, data):
            # A selected row has a different background; compare glyph pixels only.
            actual = b''.join(data[((top + y) * width + left) * 3:((top + y) * width + left + 160) * 3] for y in range(16))
            return all(actual[i:i + 3] == b'\xff\xff\xff' for i in range(0, len(expected), 3) if expected[i:i + 3] == b'\xff\xff\xff') \
                and all(actual[i:i + 3] != b'\xff\xff\xff' for i in range(0, len(expected), 3) if expected[i:i + 3] != b'\xff\xff\xff')
        scan(check, f'{description} row {row} {text!r}')

    # Only the current phase has a visible chooser.
    def users(row, text):
        chooser_row(PICKER[0] + 10, PICKER[1], row, text, 'users')

    def chrome(row, text):
        chooser_row(34, 24, row, text, 'chrome')

    def click_users(row):
        click(PICKER[0] + 40, PICKER[1] + 44 + 16 * row)

    def click_chrome(row):
        click(64, 68 + 16 * row)

    def focus_app():
        click(APP[0] + 5, APP[1] + 120)

    def app_close():
        click(width - 38, APP[1] + 10)

    def dialog(row, text):
        move(8, 8)
        expected = rows_image([text])
        def check(width, data):
            top = APP[1] + 20 + row * 16
            actual = b''.join(data[((top + y) * width + APP[0] + 10) * 3:((top + y) * width + APP[0] + 170) * 3] for y in range(16))
            return actual == expected
        scan(check, f'Notes close dialog {text!r}')

    def choose_close(row):
        click(APP[0] + 40, APP[1] + 28 + row * 16)
        move(8, 8)  # Do not compare a software cursor overlay with document pixels.

    def color(value):
        scan(lambda width, data: rgb(width, data, APP[0] + 5, APP[1] + 10) == bytes.fromhex(value), f'workspace color {value}')

    def tabs(count, current):
        # One tab per note below the text; the current note's tab is bright.
        def check(width, data):
            for index in range(count + 1):
                expected = '182132' if index == count else '66ccff' if index == current else '3a4a63'
                if rgb(width, data, APP[0] + 16 + 16 * index, APP[1] + 92) != bytes.fromhex(expected):
                    return False
            return True
        scan(check, f'note tabs count={count} current={current}')

    NEW, CLOSE, STATUS = 4, 5, 6
    # Objects: the template user is 3; a user made from it is 7 with chrome 8,
    # Notes 9 and Colors 10 inside.
    users(STATUS, 'CHOOSE')
    users(0, 'USER TEMPLATE')
    users(NEW, 'NEW USER')
    if not reboot:
        click_users(NEW)
        users(STATUS, 'COPIED')
    users(1, 'USER 1')
    shutil.copyfile(output / 'display.png', output / 'user-picker.png')
    click_users(1)
    chrome(STATUS, 'CHOOSE')
    # The picker and its heading must disappear, not remain behind
    # the user. This region is empty workspace until an app is launched.
    def picker_gone(width, data):
        return all(rgb(width, data, x, y) == bytes.fromhex('101827')
                   for y in range(PICKER[1], PICKER[1] + 180)
                   for x in range(PICKER[0], PICKER[0] + 220))
    scan(picker_gone, 'boot picker and headings retired on user entry')
    shutil.copyfile(output / 'display.png', output / 'user-workspace.png')
    chrome(0, 'LAUNCHER')
    chrome(1, 'NOTES')
    chrome(2, 'COLORS')
    click_chrome(1)
    chrome(STATUS, 'RUNNING')
    note('hello cathedral' if reboot else '')
    scan(lambda width, data: rgb(width, data, width - 30, height - 30) == bytes.fromhex('27664a'),
         'user app uses the available display extent')
    tabs(2 if reboot else 0, 0)
    if reboot:
        focus_app()
        key('f4')
        note('hello cathedral two')
        tabs(2, 1)
        app_close()
        color('101827')
        click_chrome(1)
        note('hello cathedral')
        tabs(2, 0)
        app_close()
        color('101827')
        return
    focus_app()
    for char in 'hello cathedral':
        key('spc' if char == ' ' else char)
    note('hello cathedral', dirty=True)
    click_users(CLOSE)  # Former boot controls must no longer receive input.
    note('hello cathedral', dirty=True)
    click_chrome(CLOSE)  # Closing must not discard the unsaved draft.
    chrome(STATUS, 'KEPT')
    note('hello cathedral', dirty=True)
    app_close()
    dialog(0, 'UNSAVED NOTE')
    dialog(1, 'SAVE')
    dialog(2, 'DISCARD')
    dialog(3, 'CANCEL')
    key('x')  # The modal close decision must not edit the underlying draft.
    dialog(0, 'UNSAVED NOTE')
    choose_close(3)
    note('hello cathedral', dirty=True)
    app_close()
    dialog(0, 'UNSAVED NOTE')
    choose_close(1)  # Save and exit through the app itself, not the launcher.
    color('101827')
    click_chrome(1)
    chrome(STATUS, 'RUNNING')
    note('hello cathedral')
    tabs(1, 0)
    # A clean user would close if the old boot control still received clicks.
    click_users(CLOSE)
    note('hello cathedral')
    focus_app()
    # Duplicate shares the saved content; editing the copy leaves the original.
    key('f3')
    tabs(2, 1)
    note('hello cathedral')
    for char in ' two':
        key('spc' if char == ' ' else char)
    note('hello cathedral two', dirty=True)
    key('f3')  # A draft is never duplicated; save first.
    tabs(2, 1)
    key('f1')
    note('hello cathedral two')
    key('f4')
    tabs(2, 0)
    note('hello cathedral')
    app_close()  # A clean Notes exits immediately.
    color('101827')
    click_chrome(2)
    chrome(STATUS, 'RUNNING')
    color('315ca8')
    focus_app()
    key('ret')
    color('d18b35')
    app_close()  # Colors handles its own close too.
    color('101827')
    # Repeated actual teardown/admission catches leaked loans, surfaces or
    # handles. Colors keeps its color: its state nests under its own program.
    for _ in range(3):
        click_chrome(1)
        chrome(STATUS, 'RUNNING')
        note('hello cathedral')
        click_chrome(CLOSE)
        chrome(STATUS, 'CLOSED')
        click_chrome(2)
        chrome(STATUS, 'RUNNING')
        color('d18b35')
        click_chrome(CLOSE)
        chrome(STATUS, 'CLOSED')
    # Keyboard navigation also works once the chrome has focus: from CLOSE,
    # four steps up reach Notes.
    for _ in range(4):
        key('up')
    key('ret')
    chrome(STATUS, 'RUNNING')
    note('hello cathedral')
    focus_app()
    key('x')
    click_chrome(CLOSE)
    chrome(STATUS, 'KEPT')
    note('hello cathedralx', dirty=True)
    focus_app()
    key('f2')
    note('hello cathedral')
    click_chrome(CLOSE)
    chrome(STATUS, 'CLOSED')
    color('101827')
    click_chrome(1)
    chrome(STATUS, 'RUNNING')
    note('hello cathedral')
    # The user's app remains replaceable; whole-user close is now a headless
    # matrix mechanism proof, not a logout control in the normal desktop.
    click_chrome(CLOSE)
    chrome(STATUS, 'CLOSED')
    # Same Notes bytes, different bindings. Colors requires storage and must
    # refuse cleanly without occupying the slot; Notes explicitly permits None.
    click_chrome(NEW)  # The user launcher uses this row for STORAGE ON/OFF.
    chrome(NEW, 'STORAGE OFF')
    click_chrome(2)
    chrome(STATUS, 'NEEDS SERVICE')
    click_chrome(1)
    chrome(STATUS, 'RUNNING')
    note('', temporary=True)
    def temporary_banner(width, data):
        expected = rows_image(['TEMP NO SAVE'])
        actual = b''.join(data[((APP[1] + 88 + y) * width + APP[0] + 10) * 3:((APP[1] + 88 + y) * width + APP[0] + 170) * 3] for y in range(16))
        return actual == expected
    scan(temporary_banner, 'Notes clearly identifies temporary unsavable state')
    shutil.copyfile(output / 'display.png', output / 'temporary-notes.png')
    focus_app()
    key('x')
    note('x', dirty=True, temporary=True)
    key('f1')
    note('x', dirty=True, temporary=True, error=True)
    key('f2')  # No storage means reload cannot silently discard the draft.
    note('x', dirty=True, temporary=True, error=True)
    click_chrome(CLOSE)
    chrome(STATUS, 'KEPT')
    app_close()
    dialog(1, 'NO STORAGE')
    choose_close(1)  # Unavailable save cannot pretend to have persisted the draft.
    dialog(2, 'DISCARD')
    choose_close(3)
    note('x', dirty=True, temporary=True, error=True)
    app_close()
    dialog(2, 'DISCARD')
    shutil.copyfile(output / 'display.png', output / 'notes-close.png')
    choose_close(2)
    color('101827')
    click_chrome(NEW)
    chrome(NEW, 'STORAGE ON')
    click_chrome(1)
    chrome(STATUS, 'RUNNING')
    note('hello cathedral')  # The temporary launch never touched saved Notes.
    tabs(2, 0)
    click_chrome(CLOSE)
    chrome(STATUS, 'CLOSED')
    click_chrome(2)
    chrome(STATUS, 'RUNNING')
    color('d18b35')
    print('PASS: app-owned close, save/discard/cancel, required-service refusal, optional Notes storage and persistent replacement')


def run(timeout, trace_storage=False):
    build = [sys.executable, str(HERE / 'run.py'), '--release', '--build-only']
    if trace_storage:
        build.append('--trace-storage')
    subprocess.run(build, cwd=ROOT, check=True)
    output = harness.BUILD / 'desktop' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    shutil.copytree(harness.BUILD / 'interactive/esp', output / 'esp')
    path = output / 'guest.raw'
    programs = harness.seeded_programs(ROOT / 'source-rs/distribution/profile.json', True)
    migration = ROOT / 'source-rs/distribution/migrations/readable-labels.txt'
    database_image.prepare(path, programs, migration, app_close=True)
    qemu = harness.find_qemu()
    command = [qemu, '-machine', 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1', '-m', '128']
    command += harness.firmware_args(qemu, output)
    command += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none', '-no-reboot', '-display', 'none']
    command += disk_image.arguments(path)
    flags = subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0
    for boot in (1, 2):
        capture.run(command, output, timeout, flags, ready_marker='Cathedral desktop: ready; choose a user',
                    verify=lambda stream, output, serial: desktop(stream, output, serial, boot == 2))
        for name in ('serial.log', 'qemu.log', 'display.png'):
            shutil.copyfile(output / name, output / f'boot-{boot}-{name}')
        contents = path.read_bytes()
        database_image.prepare(path, programs, migration, app_close=True)
        assert contents == path.read_bytes(), 'relaunch must not reformat storage or reinstall unchanged programs'
    # An independent host model checks the retained records on a copy: the
    # installed programs, the user copied from the template, two notes sharing
    # one manifest under the user's Notes, the color under its Colors, nothing
    # in the ungranted root.
    snapshot = output / 'host-reopen.raw'
    snapshot.write_bytes(contents)
    result = subprocess.run([str(database_image.oracle()), 'engines', 'volume', 'verify-desktop', str(snapshot),
                             str(database_image.OFFSET), str(database_image.VOLUME_BYTES),
                             *[str(item) for parent, file in programs for item in (parent, file)]],
                            cwd=ROOT / 'tools/storage-lab', check=True, capture_output=True, text=True)
    (output / 'host.txt').write_text(result.stdout, encoding='utf-8')
    print(result.stdout, end='')
    print(f'PASS: default F5 desktop, a copied user, duplicated notes and saved state after cold reboot; artifacts: {output}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--timeout', type=float, default=180)
    parser.add_argument('--trace-storage', action='store_true', help='build with storage phase traces in the serial log')
    args = parser.parse_args()
    if args.timeout <= 0:
        parser.error('timeout must be positive')
    run(args.timeout, args.trace_storage)
