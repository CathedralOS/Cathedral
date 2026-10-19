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


def desktop(stream, output, serial, reboot, expected_absolute=True, fill_users=False, service_imports=None, nested=False):
    assert 'Cathedral: object storage ready' in serial
    position = [40, 40]
    font = bytes.fromhex((ROOT / 'source-rs/platform/libraries/bitmap-font/font.hex').read_text())

    def send(events):
        capture.command(stream, 'input-send-event', {'events': events})

    def key(code):
        for down in (True, False):
            send([{'type': 'key', 'data': {'down': down, 'key': {'type': 'qcode', 'data': code}}}])
            time.sleep(0.07)

    absolute = 'Cathedral input: absolute pointer ready' in serial
    assert absolute == expected_absolute, 'unexpected input transport: ' + serial[-1500:]
    mice = capture.command(stream, 'query-mice')
    assert any(m['current'] and m['absolute'] == absolute for m in mice), mice

    def move(x, y):
        if absolute:
            send([{'type': 'abs', 'data': {'axis': 'x', 'value': round(x * 32767 / (width-1))}},
                  {'type': 'abs', 'data': {'axis': 'y', 'value': round(y * 32767 / (height-1))}}])
            position[:] = [x,y]
            time.sleep(0.05)
            return
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

    def tile_label(left, top, text):
        move(8,8)
        def check(width,data):
            start=left+(128-len(text)*12)//2
            for i,ch in enumerate(text):
                for y,bits in enumerate(font[(ord(ch)-32)*7:(ord(ch)-31)*7]):
                    for x in range(5):
                        actual=rgb(width,data,start+i*12+x*2,top+76+y*2)==b'\xff\xff\xff'
                        if actual != bool(bits & (1 << (4-x))): return False
            return True
        scan(check,f'tile {text}')

    def user_tile(index,count):
        return ((width-(count*152-24))//2+index*152,(height-96)//2)

    def click_user_tile(index,count):
        x,y=user_tile(index,count)
        click(x+64,y+48)

    def click_chrome(index):
        click(120,96+(index-1)*120)

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

    # The template is hidden. Only user tiles and an add-user tile are visible.
    if not reboot:
        tile_label(*user_tile(0,1),'NEW USER')
        click_user_tile(0,1)
    total = 2
    if fill_users:
        assert reboot
        for count in (1,2):
            tile_label(*user_tile(count,count+1),'NEW USER')
            click_user_tile(count,count+1)
            total=3
            tile_label(*user_tile(count,3),f'USER {count+1}')
    tile_label(*user_tile(0,total),'USER 1')
    if not fill_users:
        tile_label(*user_tile(1,total),'NEW USER')
    else:
        tile_label(*user_tile(1,3),'USER 2')
        tile_label(*user_tile(2,3),'USER 3')
        # At capacity, keyboard movement repaints three retained avatar tiles.
        key('left')
        key('right')
        tile_label(*user_tile(2,3),'USER 3')
    shutil.copyfile(output / 'display.png', output / 'user-picker.png')
    click_user_tile(0,total)
    tile_label(56,48,'NOTES')
    tile_label(56,168,'COLORS')
    tile_label(56,288,'SPLIT')
    # Check the entire old tile band, not a background pixel between avatars.
    # No picker process/control connection survives into the user workspace.
    stopped = 'Cathedral desktop: picker stopped; loading user launcher'
    ready = 'Cathedral desktop: user matrix furnished'
    log = (output / 'serial.log').read_text(errors='replace')
    assert stopped in log and ready in log and log.index(stopped) < log.index(ready), log[-2000:]
    def picker_gone(w,d):
        return all(rgb(w,d,x,y)==bytes.fromhex('101827')
                   for y in range((height-96)//2,(height+96)//2)
                   for x in range(256,width-24))
    scan(picker_gone, 'every old user tile retired')
    click_user_tile(total-1,total)  # Former New User/User tile must be inert.
    move(8,8)
    scan(picker_gone, 'old picker cannot redraw after a click')
    shutil.copyfile(output / 'display.png', output / 'user-workspace.png')
    if nested:
        import split_test
        split_test.exercise(output, click, key, scan, width, height)
        return
    click_chrome(1)
    if service_imports:
        note('', temporary=True)
        focus_app()
        key('a')
        note('a', dirty=True, temporary=True)
        app_close()
        dialog(0, 'UNSAVED NOTE')
        choose_close(2)
        color('101827')
        click_chrome(1)
        note('', temporary=True)
        app_close()
        color('101827')
        # The unavailable Notes binding must not affect its sibling's storage.
        click_chrome(2)
        color('d18b35' if reboot else '315ca8')
        if not reboot:
            focus_app()
            key('ret')
            color('d18b35')
        log = (output / 'serial.log').read_text(errors='replace')
        assert log.count('Cathedral Notes: temporary; no storage endpoint; interaction storage denied') >= 2
        print(f'PASS: {service_imports} Notes storage stays absent, dirty-close works, Colors retains its own binding')
        return
    note('hello cathedral' if reboot else '')
    tabs(2 if reboot else 0,0)
    if reboot:
        focus_app()
        key('f4')
        note('hello cathedral two')
        tabs(2,1)
        app_close()
        color('101827')
        click_chrome(1)
        note('hello cathedral')
        app_close()
        color('101827')
        return
    focus_app()
    for char in 'hello cathedral': key('spc' if char==' ' else char)
    note('hello cathedral',dirty=True)
    # A second app cannot replace a live draft.
    click_chrome(2)
    note('hello cathedral',dirty=True)
    app_close()
    dialog(0,'UNSAVED NOTE')
    dialog(1,'SAVE')
    dialog(2,'DISCARD')
    dialog(3,'CANCEL')
    key('x')
    dialog(0,'UNSAVED NOTE')
    choose_close(3)
    note('hello cathedral',dirty=True)
    app_close()
    choose_close(1)
    color('101827')
    click_chrome(1)
    note('hello cathedral')
    tabs(1,0)
    focus_app()
    key('f3')
    tabs(2,1)
    for char in ' two': key('spc' if char==' ' else char)
    note('hello cathedral two',dirty=True)
    key('f3')
    tabs(2,1)
    key('f1')
    note('hello cathedral two')
    key('f4')
    note('hello cathedral')
    app_close()
    color('101827')
    click_chrome(2)
    color('315ca8')
    focus_app()
    key('ret')
    color('d18b35')
    app_close()
    color('101827')
    for _ in range(3):
        click_chrome(1)
        note('hello cathedral')
        app_close()
        color('101827')
        click_chrome(2)
        color('d18b35')
        app_close()
        color('101827')
    # Focus the tile strip, then keyboard-select Notes from Colors.
    click(40,300)
    key('up')
    key('ret')
    note('hello cathedral')
    focus_app()
    key('x')
    note('hello cathedralx',dirty=True)
    app_close()
    dialog(2,'DISCARD')
    shutil.copyfile(output / 'display.png', output / 'notes-close.png')
    choose_close(2)
    color('101827')
    click_chrome(1)
    note('hello cathedral')
    tabs(2,0)
    app_close()
    color('101827')
    click_chrome(2)
    color('d18b35')
    print('PASS: user tiles, absolute/relative clicks, app tiles, save/discard/cancel and persistent replacement')


def run(timeout, trace_storage=False, relative_pointer=False, service_imports=None, nested=False):
    output = harness.BUILD / 'desktop' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    # Exact default composition, with a separate build output so a regression
    # run does not replace the ESP of an already-running interactive guest.
    profile = output / 'desktop-check.json'
    shutil.copyfile(ROOT / 'source-rs/distribution/profile.json', profile)
    build = [sys.executable, str(HERE / 'run.py'), '--release', '--build-only', '--profile', str(profile)]
    if trace_storage:
        build.append('--trace-storage')
    subprocess.run(build, cwd=ROOT, check=True)
    shutil.copytree(harness.BUILD / 'desktop-check-interactive/esp', output / 'esp')
    path = output / 'guest.raw'
    programs = harness.seeded_programs(profile, True)
    if service_imports:
        # Alter only a disposable sealed-image input to exercise the real launcher
        # with an undeclared or incompatible optional import. No grant/UI override.
        changed = []
        replacements = 0
        for parent, file in programs:
            if 'cathedral-notes.elf' in file.name:
                data = bytearray(file.read_bytes())
                assert data.count(b'CTHIMP02') == 1
                at = data.index(b'CTHIMP02')
                assert int.from_bytes(data[at+48:at+56], 'little') == 2
                if service_imports == 'undeclared':
                    data[at+8:at+16] = (1).to_bytes(8, 'little')
                    data[at+48:at+144] = bytes(96)
                else:
                    data[at+56:at+64] = (2).to_bytes(8, 'little')
                    data[at+64:at+72] = (2).to_bytes(8, 'little')
                file = output / file.name
                file.write_bytes(data)
                replacements += 1
            changed.append((parent, file))
        assert replacements == 1
        programs = changed
    migration = ROOT / 'source-rs/distribution/migrations/readable-labels.txt'
    database_image.prepare(path, programs, migration, app_close=True, realm_programs=harness.realm_programs(profile, True))
    qemu = harness.find_qemu()
    command = [qemu, '-machine', 'q35,vmport=off' if relative_pointer else 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1', '-m', '128']
    command += harness.firmware_args(qemu, output)
    command += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none', '-no-reboot', '-display', 'none']
    command += disk_image.arguments(path)
    flags = subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0
    for boot in (1, 2):
        capture.run(command, output, timeout, flags, ready_marker='Cathedral desktop: ready; choose a user',
                    verify=lambda stream, output, serial: desktop(stream, output, serial, boot == 2, not relative_pointer, service_imports=service_imports, nested=nested))
        for name in ('serial.log', 'qemu.log', 'display.png'):
            shutil.copyfile(output / name, output / f'boot-{boot}-{name}')
        contents = path.read_bytes()
        database_image.prepare(path, programs, migration, app_close=True, realm_programs=harness.realm_programs(profile, True))
        assert contents == path.read_bytes(), 'relaunch must not reformat storage or reinstall unchanged programs'
    if nested:
        print(f'PASS: F5 nested Split matrix across cold reboot; artifacts: {output}')
        return
    if service_imports:
        print(f'PASS: scoped service acquisition with {service_imports} storage across cold reboot; artifacts: {output}')
        return
    # An independent host model checks the retained records on a copy: the
    # installed programs, the user copied from the template, two notes sharing
    # one manifest under the user's Notes, the color under its Colors, nothing
    # in the ungranted root.
    snapshot = output / 'host-reopen.raw'
    snapshot.write_bytes(contents)
    result = subprocess.run([str(database_image.oracle()), 'engines', 'volume', 'verify-desktop', str(snapshot),
                             str(database_image.OFFSET), str(database_image.VOLUME_BYTES),
                             *[str(item) for parent, file in programs for item in (parent, file)],
                             *[str(item) for pair in harness.realm_programs(profile, True) for item in pair]],
                            cwd=ROOT / 'tools/storage-lab', check=True, capture_output=True, text=True)
    (output / 'host.txt').write_text(result.stdout, encoding='utf-8')
    print(result.stdout, end='')
    print(f'PASS: default F5 desktop, a copied user, duplicated notes and saved state after cold reboot; artifacts: {output}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--timeout', type=float, default=180)
    parser.add_argument('--relative-pointer', action='store_true', help='disable VM port and verify PS/2 fallback')
    parser.add_argument('--service-imports', choices=('undeclared', 'incompatible'), help='exercise absent optional storage in the ordinary desktop using a disposable manifest variant')
    parser.add_argument('--nested', action='store_true', help='exercise the installed alternative Split matrix')
    parser.add_argument('--trace-storage', action='store_true', help='build with storage phase traces in the serial log')
    args = parser.parse_args()
    if args.timeout <= 0:
        parser.error('timeout must be positive')
    run(args.timeout, args.trace_storage, args.relative_pointer, args.service_imports, args.nested)
