#!/usr/bin/env python3
"""Exercise the ordinary F5 composition: choose apps, edit, switch, clone the world and cold-open."""
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

    def note(text, dirty=False):
        expected = text_image(text, cursor=True)
        def check(width, data):
            actual = b''.join(data[((200 + y) * width + 340) * 3:((200 + y) * width + 520) * 3] for y in range(64))
            return actual == expected and rgb(width, data, 335, 190) == bytes.fromhex('9b7028' if dirty else '27664a')
        scan(check, f'note text/status {text!r}, dirty={dirty}')

    def menu_row(row, text, description):
        move(270, 300)  # Keep the cursor overlay outside the checked glyphs.
        expected = text_image(text)[:180 * 16 * 3]
        top = 160 + 16 * row
        scan(lambda width, data: b''.join(data[((top + y) * width + 50) * 3:((top + y) * width + 230) * 3] for y in range(16)) == expected,
             f'{description} {text!r}')

    def status(text):
        menu_row(5, text, 'menu status')

    def world(index, count):
        menu_row(6, f'WORLD {index} OF {count}', 'menu world line')

    def color(value):
        scan(lambda width, data: rgb(width, data, 335, 190) == bytes.fromhex(value), f'workspace color {value}')

    def tabs(count, current):
        # One tab per note below the text; the current note's tab is bright.
        def check(width, data):
            for index in range(count + 1):
                expected = '182132' if index == count else '66ccff' if index == current else '3a4a63'
                if rgb(width, data, 346 + 16 * index, 272) != bytes.fromhex(expected):
                    return False
            return True
        scan(check, f'note tabs count={count} current={current}')

    status('CHOOSE PROGRAM')
    world(1, 2 if reboot else 1)
    click(80, 166)  # Actual menu row, not host-side launch control.
    status('NOTES OPEN')
    note('hello cathedral' if reboot else '')
    tabs(2 if reboot else 0, 0)
    if reboot:
        click(335, 300)
        key('f4')
        note('hello cathedral two')
        tabs(2, 1)
        # The cloned world kept its own edit; the original kept none of it.
        click(80, 230)
        world(2, 2)
        status('NOTES OPEN')
        note('hello cathedral')
        tabs(2, 0)
        click(335, 300)
        key('f4')
        note('hello cathedral two clone')
        tabs(2, 1)
        click(80, 230)
        world(1, 2)
        note('hello cathedral')
        return
    click(335, 300)
    for char in 'hello cathedral':
        key('spc' if char == ' ' else char)
    note('hello cathedral', dirty=True)
    click(80, 182)  # Switching must not discard the unsaved draft.
    status('SAVE NOTE FIRST')
    note('hello cathedral', dirty=True)
    click(335, 300)
    key('f1')
    note('hello cathedral')
    tabs(1, 0)
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
    click(80, 182)
    status('COLORS OPEN')
    color('2c8762')
    click(335, 300)
    key('ret')
    color('d18b35')
    click(80, 198)
    status('CHOOSE PROGRAM')
    color('101827')
    # Repeated actual teardown/admission catches leaked loans, surfaces or handles.
    for _ in range(5):
        click(80, 166)
        status('NOTES OPEN')
        note('hello cathedral')
        click(80, 182)
        status('COLORS OPEN')
        color('2c8762')
    # Keyboard navigation also works once the launcher has focus.
    key('up')
    key('ret')
    status('NOTES OPEN')
    note('hello cathedral')
    click(335, 300)
    key('x')
    click(80, 198)
    status('SAVE NOTE FIRST')
    note('hello cathedralx', dirty=True)
    click(335, 300)
    key('f2')
    note('hello cathedral')
    click(80, 198)
    status('CHOOSE PROGRAM')
    color('101827')
    click(80, 166)
    status('NOTES OPEN')
    note('hello cathedral')
    # Cloning the world copies both notes by reference and reopens Notes in
    # the clone; a draft refuses the clone just as it refuses a switch.
    click(335, 300)
    key('y')
    note('hello cathedraly', dirty=True)
    click(80, 214)
    status('SAVE NOTE FIRST')
    world(1, 1)
    click(335, 300)
    key('f2')
    note('hello cathedral')
    click(80, 214)
    world(2, 2)
    status('NOTES OPEN')
    note('hello cathedral')
    tabs(2, 0)
    click(335, 300)
    key('f4')
    note('hello cathedral two')
    tabs(2, 1)
    for char in ' clone':
        key('spc' if char == ' ' else char)
    key('f1')
    note('hello cathedral two clone')
    # Back in the first world the original notes are untouched.
    click(80, 230)
    world(1, 2)
    status('NOTES OPEN')
    note('hello cathedral')
    tabs(2, 0)
    click(335, 300)
    key('f4')
    note('hello cathedral two')
    print('PASS: in-guest mouse/keyboard launch, draft protection, duplicate/switch, save/reload, repeated app replacement and world cloning')


def run(timeout):
    subprocess.run([sys.executable, str(HERE / 'run.py'), '--release', '--build-only'], cwd=ROOT, check=True)
    output = harness.BUILD / 'desktop' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    shutil.copytree(harness.BUILD / 'interactive/esp', output / 'esp')
    path = output / 'guest.raw'
    database_image.prepare(path)
    qemu = harness.find_qemu()
    command = [qemu, '-machine', 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1', '-m', '128']
    command += harness.firmware_args(qemu, output)
    command += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none', '-no-reboot', '-display', 'none']
    command += disk_image.arguments(path)
    flags = subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0
    for boot in (1, 2):
        capture.run(command, output, timeout, flags, ready_marker='Cathedral desktop: ready; choose a program',
                    verify=lambda stream, output, serial: desktop(stream, output, serial, boot == 2))
        for name in ('serial.log', 'qemu.log', 'display.png'):
            shutil.copyfile(output / name, output / f'boot-{boot}-{name}')
        contents = path.read_bytes()
        database_image.prepare(path)
        assert contents == path.read_bytes(), 'relaunch must not reformat storage'
    # An independent host model checks the retained records on a copy: two
    # worlds of two notes sharing content, nothing in the ungranted root.
    snapshot = output / 'host-reopen.raw'
    snapshot.write_bytes(contents)
    result = subprocess.run([str(database_image.oracle()), 'engines', 'volume', 'verify-desktop', str(snapshot),
                             str(database_image.OFFSET), str(database_image.VOLUME_BYTES)],
                            cwd=ROOT / 'tools/storage-lab', check=True, capture_output=True, text=True)
    (output / 'host.txt').write_text(result.stdout, encoding='utf-8')
    print(result.stdout, end='')
    print(f'PASS: default F5 desktop, duplicated notes, cloned world and saved notes after cold reboot; artifacts: {output}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--timeout', type=float, default=180)
    args = parser.parse_args()
    if args.timeout <= 0:
        parser.error('timeout must be positive')
    run(args.timeout)
