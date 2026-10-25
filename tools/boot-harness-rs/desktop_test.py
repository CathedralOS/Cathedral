#!/usr/bin/env python3
"""Exercise the ordinary F5 composition: make a user, launch its programs, edit, switch and cold-open."""
import argparse
import json
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
import desktop_timing
import pointer_test
import run as harness

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def desktop(stream, output, serial, reboot, expected_absolute=True, fill_users=False, service_imports=None, nested=False, nested_saved=False, deep=False, nested_desktop=False, user_check=None, allowed_panics=(), latency=False, pointer_stress=False, interaction_delay=False, nested_depth=3, soak=0, collection_pressure=False):
    assert 'Cathedral: object storage ready' in serial
    assert 'Cathedral input: shared source connection ready' in serial
    assert 'Cathedral matrix: shared display connection ready' in serial
    assert 'Cathedral matrix: shared storage connection ready' in serial
    position = [40, 40]
    motion_step = 0
    timings = desktop_timing.Timings(output, reboot)
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

    def point(x, y):
        # The pointer events that reach (x, y) on the current transport: one absolute
        # position, or paced relative steps; no settling pause.
        if absolute:
            send([{'type': 'abs', 'data': {'axis': 'x', 'value': round(x * 32767 / (width-1))}},
                  {'type': 'abs', 'data': {'axis': 'y', 'value': round(y * 32767 / (height-1))}}])
            position[:] = [x,y]
            return
        while position != [x, y]:
            dx = max(-70, min(70, x - position[0]))
            dy = max(-70, min(70, y - position[1]))
            send([{'type': 'rel', 'data': {'axis': 'x', 'value': dx}},
                  {'type': 'rel', 'data': {'axis': 'y', 'value': dy}}])
            position[0] += dx
            position[1] += dy
            time.sleep(0.04)

    def move(x, y):
        point(x, y)
        if absolute:
            time.sleep(0.05)

    def click(x, y):
        move(x, y)
        for down in (True, False):
            send([{'type': 'btn', 'data': {'button': 'left', 'down': down}}])
            time.sleep(0.08)

    def scan(check, description):
        nonlocal motion_step
        deadline = time.monotonic() + (120 if nested_desktop or latency or soak else 45)
        while True:
            if pointer_stress:
                motion_step += 1
                send([{'type': 'abs', 'data': {'axis': axis, 'value': value}}
                      for axis, value in [('x', 12000 + (motion_step * 941) % 18000),
                                          ('y', 4000 + (motion_step * 677) % 24000)]])
            serial = (output / 'serial.log').read_text(errors='replace')
            assert 'CATHEDRAL_RS_PANIC' not in serial and 'initial program stopped' not in serial, serial[-3000:]
            if pointer_stress:
                assert 'matrix-service-exit' not in serial, serial[-5000:]
            for report in serial.replace('Cathedral Rust lab: ', '').split('Cathedral user panic:')[1:]:
                if report.count('\n') >= 2:
                    assert report.splitlines()[1] in allowed_panics, serial[-3000:]
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
    APP = (8, 8)

    def note(text, dirty=False, temporary=False, error=False):
        expected = text_image(text, cursor=True)
        def check(width, data):
            actual = b''.join(data[((APP[1] + 20 + y) * width + APP[0] + 10) * 3:((APP[1] + 20 + y) * width + APP[0] + 190) * 3] for y in range(64))
            return actual == expected and rgb(width, data, APP[0] + 5, APP[1] + 10) == bytes.fromhex('993f46' if error else '9b7028' if dirty else '55447a' if temporary else '27664a')
        scan(check, f'note text/status {text!r}, dirty={dirty}')

    def measure_input(text):
        # End-to-end upper bounds: QMP key injection through screenshot observation.
        # No deliberate key delay; scan polling/capture overhead remains included.
        samples = []
        for char in 'xyz':
            with timings.step('type-character'):
                start = time.perf_counter()
                send([{'type': 'key', 'data': {'down': down, 'key': {'type': 'qcode', 'data': char}}}
                      for down in (True, False)])
                text += char
                note(text, dirty=True)
                samples.append(round((time.perf_counter() - start) * 1000, 1))
        import json
        (output / 'input-latency.json').write_text(json.dumps({'qmp_to_observed_text_ms': samples}, indent=2))
        print(f'Input-to-observed-text upper bounds (QMP + capture included): {samples} ms')
        for _ in 'xyz': key('backspace')
        note(text[:-3], dirty=True)

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
        click(40+(index-1)*56,height-48)

    def focus_app():
        click(APP[0] + 5, APP[1] + 120)

    def app_close():
        click(width - 22, APP[1] + 10)

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
        with timings.step('create-user'):
            click_user_tile(0,1)
            tile_label(*user_tile(0,2),'USER 1')
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
    with timings.step('enter-user'):
        click_user_tile(0,total)
        scan(lambda w,d: rgb(w,d,32,height-56)==bytes.fromhex('d6e5ef'), 'bottom taskbar Notes icon')
    move(2, 2)
    # Check the entire old tile band, not a background pixel between avatars.
    # No picker process/control connection survives into the user workspace.
    stopped = 'Cathedral desktop: picker stopped; loading user launcher'
    ready = 'Cathedral desktop: user matrix furnished'
    log = (output / 'serial.log').read_text(errors='replace')
    assert stopped in log and ready in log and log.index(stopped) < log.index(ready), log[-2000:]
    assert 'Cathedral matrix: shared parent interaction ready' in log, log[-2000:]
    assert 'Cathedral matrix: shared parent storage ready' in log, log[-2000:]
    assert 'Cathedral Omega host: shared storage ready' in log, log[-2000:]
    assert 'Cathedral Omega host: shared interaction broker ready' in log, log[-2000:]
    scan(lambda _width, _pixels: 'Cathedral background: shared interaction ready; storage denied'
         in (output/'serial.log').read_text(errors='replace'), 'shared background with storage denied')
    def picker_gone(w,d):
        return all(rgb(w,d,x,y)==bytes.fromhex('101827')
                   for y in range((height-96)//2,(height+96)//2)
                   for x in range(256,width-24))
    scan(picker_gone, 'every old user tile retired')
    click_user_tile(total-1,total)  # Former New User/User tile must be inert.
    move(8,8)
    scan(picker_gone, 'old picker cannot redraw after a click')
    shutil.copyfile(output / 'display.png', output / 'user-workspace.png')
    if latency:
        import desktop_latency
        desktop_latency.exercise(stream, output, click, point, scan, width, height, reboot, timings, nested_depth)
        return
    if user_check is not None:
        user_check(output, click, key, scan, width, height, reboot)
        return
    if soak:
        import desktop_soak
        desktop_soak.exercise(output, click, key, scan, width, height, reboot, soak, pointer_stress, nested_depth, collection_pressure)
        return
    if nested_desktop:
        import nested_desktop_test
        nested_desktop_test.exercise(output, click, key, scan, width, height, reboot, pointer_stress, interaction_delay, nested_depth)
        return
    if nested:
        import split_test
        split_test.exercise(output, click, key, scan, width, height, nested_saved, deep)
        return
    if not reboot and not service_imports:
        import multitask_test
        multitask_test.exercise(click_chrome, click, key, note, focus_app, app_close, dialog, choose_close, color, measure_input, timings)
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
        assert log.count('Cathedral Notes: temporary; no storage endpoint') >= 2
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
    # A second app coexists; closing it reveals the untouched live draft.
    click_chrome(2)
    color('315ca8')
    app_close()
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
    with timings.step('save-note'):
        key('f1')
        note('hello cathedral two')
    key('f4')
    note('hello cathedral')
    app_close()
    color('101827')
    with timings.step('launch-colors'):
        click_chrome(2)
        color('315ca8')
    focus_app()
    with timings.step('save-color'):
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
    click(232,height-48)
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


def run(timeout, trace_storage=False, relative_pointer=False, service_imports=None, nested=False, deep=False, nested_desktop=False, trace_input=False, latency=False, pointer_stress=False, interaction_delay=False, nested_depth=3, soak=0, collection_pressure=False, accel='tcg', input_producer=None, participants=False, participant_fault=None, render_demand=False, masked_capture=False, capture_case=None, cursor_preferences=False, cursor_producer=False, launcher_async=None, launch_async=None, binding_async=None, scene_async=None, selection_async=None):
    output = harness.BUILD / 'desktop' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    # Exact default composition, with a separate build output so a regression
    # run does not replace the ESP of an already-running interactive guest.
    profile = output / f'desktop-check-{output.name}.json'
    composition = json.loads((ROOT / 'source-rs/distribution/profile.json').read_text())
    if launcher_async:
        import launcher_async_test
        launcher_async_test.configure(composition, launcher_async)
    if launch_async:
        import stored_launch_test
        stored_launch_test.configure(composition, launch_async)
    if selection_async:
        import selection_test
        selection_test.configure(composition, selection_async)
    if scene_async:
        import stored_launch_test
        stored_launch_test.configure(composition, scene_async, stage='scene')
    if binding_async:
        import stored_launch_test
        stored_launch_test.configure(composition, binding_async, stage='bind')
    if render_demand or masked_capture:
        for entry in composition['startup']['realm_programs']:
            if entry.get('path') in ('Split', 'Split/Notes'):
                entry.setdefault('features', []).append('render-demand-probe')
            if masked_capture and entry.get('path') == 'Split/Capture':
                entry.setdefault('features', []).append('capture-probe')
    if capture_case and capture_case != 'ordinary':
        for entry in composition['startup']['realm_programs']:
            if entry.get('path') == 'Split/Notes':
                entry.setdefault('features', []).append('capture-source-exit' if capture_case == 'source' else 'capture-source-stall')
            if entry.get('path') == 'Split/Capture' and capture_case != 'source':
                entry.setdefault('features', []).append('capture-recipient-exit' if capture_case == 'recipient' else 'capture-timeout')
    if cursor_producer:
        for entry in composition['startup']['realm_programs']:
            if entry.get('path') == 'Split/Producer':
                entry.setdefault('features', []).append('cursor-probe')
    if participant_fault:
        for entry in composition['startup']['realm_programs']:
            if entry.get('package') == 'cathedral-input-demo':
                entry.setdefault('features', []).append(participant_fault + '-probe')
            if entry.get('path') == 'Split/Notes':
                entry.setdefault('features', []).append('input-receiver-probe')
    if input_producer:
        def producer_variant(value):
            if isinstance(value, dict):
                if value.get('package') == 'cathedral-notes':
                    value.setdefault('features', []).append('input-producer-partial' if input_producer == 'partial' else 'input-producer-probe')
                if value.get('package') == 'cathedral-stock-matrix' and input_producer != 'denied':
                    value.setdefault('features', []).append('input-producer-probe')
                for child in value.values(): producer_variant(child)
            elif isinstance(value, list):
                for child in value: producer_variant(child)
        producer_variant(composition)
    if interaction_delay:
        def enable(value):
            if isinstance(value, dict):
                if value.get('package') == 'cathedral-stock-matrix':
                    value.setdefault('features', []).append('interaction-delay')
                for child in value.values(): enable(child)
            elif isinstance(value, list):
                for child in value: enable(child)
        enable(composition)
    profile.write_text(json.dumps(composition, indent=2), encoding='utf-8', newline='\n')
    build = [sys.executable, str(HERE / 'run.py'), '--release', '--build-only', '--profile', str(profile)]
    if trace_storage:
        build.append('--trace-storage')
    if trace_input:
        build.append('--trace-input')
    if collection_pressure:
        build.append('--collection-pressure')
    subprocess.run(build, cwd=ROOT, check=True)
    shutil.copytree(harness.BUILD / f'{profile.stem}-interactive/esp', output / 'esp')
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
                assert data.count(b'CTHIMP03') == 1
                at = data.index(b'CTHIMP03')
                assert int.from_bytes(data[at+48:at+56], 'little') == 2
                if service_imports == 'undeclared':
                    # Count shares the header with channel/descendant requests.
                    # Removing storage must not erase the broker's own funding.
                    header = int.from_bytes(data[at+8:at+16], 'little')
                    data[at+8:at+16] = ((header & ~0xff) | 1).to_bytes(8, 'little')
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
    command = [qemu, '-machine', 'q35,vmport=off' if relative_pointer else 'q35', '-cpu', 'qemu64', '-accel', accel, '-smp', '1', '-m', '128']
    command += harness.firmware_args(qemu, output)
    command += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none', '-no-reboot', '-display', 'none']
    command += disk_image.arguments(path)
    flags = subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0
    user_check = None
    if render_demand or masked_capture:
        import render_demand_test
        from functools import partial
        user_check = partial(render_demand_test.exercise, capture=masked_capture)
    if cursor_preferences or cursor_producer:
        import cursor_preferences_test
        from functools import partial
        user_check = partial(cursor_preferences_test.exercise, synthetic=cursor_producer)
    if participants:
        import participants_test
        user_check = participants_test.exercise
    if participant_fault:
        import participant_fault_test
        from functools import partial
        user_check = partial(participant_fault_test.exercise, mode=participant_fault)
    if capture_case:
        import capture_lifecycle_test
        from functools import partial
        user_check = partial(capture_lifecycle_test.exercise, mode=capture_case)
    if launcher_async:
        import launcher_async_test
        from functools import partial
        user_check = partial(launcher_async_test.exercise, mode=launcher_async)
    if launch_async:
        import stored_launch_test
        from functools import partial
        user_check = partial(stored_launch_test.exercise, mode=launch_async)
    if selection_async:
        import selection_test
        from functools import partial
        user_check = partial(selection_test.exercise, mode=selection_async)
    if scene_async:
        import stored_launch_test
        from functools import partial
        user_check = partial(stored_launch_test.exercise, mode=scene_async, stage='scene')
    if binding_async:
        import stored_launch_test
        from functools import partial
        user_check = partial(stored_launch_test.exercise, mode=binding_async, stage='bind')
    for boot in (1, 2):
        capture.run(command, output, timeout, flags, ready_marker='Cathedral desktop: ready; choose a user',
                    verify=lambda stream, output, serial: desktop(stream, output, serial, boot == 2, not relative_pointer, service_imports=service_imports, nested=nested, nested_saved=boot == 2, deep=deep, nested_desktop=nested_desktop, latency=latency, pointer_stress=pointer_stress, interaction_delay=interaction_delay, nested_depth=nested_depth, soak=soak, collection_pressure=collection_pressure, user_check=user_check, allowed_panics=("deliberate launcher failure",) if launcher_async else ()))
        if input_producer:
            expected = {'complete': 'acquired isolated connection and closed',
                        'partial': 'partial acquisition retained',
                        'denied': 'undelegated acquisition refused'}[input_producer]
            producer_log = (output / 'serial.log').read_text(errors='replace')
            # Both boots close and relaunch Notes. A lone successful startup
            # cannot prove that interrupted handoffs return their allocation.
            assert producer_log.count(f'Cathedral producer: {expected}') >= 2
            if input_producer == 'partial' and boot == 1:
                assert 'Cathedral producer: capacity refused; broker retained' in producer_log
        for name in ('serial.log', 'qemu.log', 'display.png', 'boot-timing.json'):
            shutil.copyfile(output / name, output / f'boot-{boot}-{name}')
        contents = path.read_bytes()
        database_image.prepare(path, programs, migration, app_close=True, realm_programs=harness.realm_programs(profile, True))
        assert contents == path.read_bytes(), 'relaunch must not reformat storage or reinstall unchanged programs'
    if selection_async:
        print(f'PASS: yielding existing-app selection {selection_async}, input and cold reopen; artifacts: {output}')
        return
    if scene_async:
        print(f'PASS: yielding launch scene {scene_async}, input and cold reopen; artifacts: {output}')
        return
    if binding_async:
        print(f'PASS: nested asynchronous binding {binding_async}, sibling input and cold reopen; artifacts: {output}')
        return
    if launch_async:
        print(f'PASS: nested ordinary launch {launch_async}, sibling input and cold reopen; artifacts: {output}')
        return
    if launcher_async:
        print(f'PASS: nested launcher asynchronous {launcher_async}, live sibling input and cold reopen; artifacts: {output}')
        return
    if cursor_preferences or cursor_producer:
        label = 'independent producer cursor suppression' if cursor_producer else 'recursive cursor preferences and target retirement'
        print(f'PASS: {label} and cold reopen; artifacts: {output}')
        return
    if capture_case:
        print(f'PASS: capture {capture_case}, repeated teardown and cold reopen; artifacts: {output}')
        return
    if render_demand or masked_capture:
        label = 'masked capture, recipient byte inspection and hidden input' if masked_capture else 'hidden input and deferred painting'
        print(f'PASS: {label}, restored content and cold reopen; artifacts: {output}')
        return
    if participant_fault:
        print(f'PASS: participant {participant_fault} interruption, replacement and cold reopen; artifacts: {output}')
        return
    if participants:
        print(f'PASS: independent participants, cursor retirement, repeated grants and cold reopen; artifacts: {output}')
        return
    if soak:
        import desktop_soak
        desktop_soak.report(output)
        print(f'PASS: {soak} soak cycles through {nested_depth} nested desktops, exact saved text, bounded provider heap and cold reopen; artifacts: {output}')
        return
    if latency:
        desktop_timing.report(output)
        print(f'PASS: outer and {nested_depth} nested desktop latency, saved text and cold reopen; artifacts: {output}')
        return
    if nested_desktop:
        print(f'PASS: nested desktop login, recursive close, confined persistence and cold reopen; artifacts: {output}')
        return
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
    parser.add_argument('--selection-async', choices=['complete', 'failure', 'stop', 'late-stop'], help='hold an existing-app switch; prove input and cancellation')
    parser.add_argument('--scene-async', choices=['complete', 'revoke', 'stop', 'failure', 'late-revoke', 'late-stop-target'], help='hold a staged launch scene before commit; prove input and abort/rollback')
    parser.add_argument('--binding-async', choices=('complete', 'revoke', 'stop', 'timeout'), help='nested asynchronous launch binding, input and rollback fixture')
    parser.add_argument('--launch-async', choices=('complete', 'revoke', 'stop', 'bind', 'place'), help='ordinary nested launch delay, cancellation and rollback fixture')
    parser.add_argument('--launcher-async', choices=('recover', 'revoke', 'stop'), help='nested ticketed launcher read, input and cancellation fixture')
    parser.add_argument('--timeout', type=float, default=180)
    parser.add_argument('--accel', choices=('tcg', 'whpx'), default='tcg', help='QEMU accelerator: tcg emulates the CPU; whpx runs the guest under the Windows hypervisor, with hardware paging and timing (default: tcg)')
    parser.add_argument('--relative-pointer', action='store_true', help='disable VM port and verify PS/2 fallback')
    parser.add_argument('--input-producer', choices=('complete', 'partial', 'denied'), help='disposable Notes producer acquisition/custody variant')
    parser.add_argument('--service-imports', choices=('undeclared', 'incompatible'), help='exercise absent optional storage in the ordinary desktop using a disposable manifest variant')
    parser.add_argument('--nested-desktop', action='store_true', help='exercise the installed nested login and desktop')
    parser.add_argument('--nested-depth', type=int, choices=(1, 2, 3, 4), default=3, help='nested desktops below the outer user; 3 or 4 for --nested-desktop/--latency, any for --soak')
    parser.add_argument('--soak', type=int, default=0, metavar='CYCLES', help='repeat enter/edit/save/teardown through the nested depth this many times in one session, then two more after a cold boot')
    parser.add_argument('--collection-pressure', action='store_true', help='with --soak and --trace-storage: build the provider to collect before every write inside a tiny workspace, and require live collection passes in every cycle')
    parser.add_argument('--latency', action='store_true', help='measure launch, typing and save through the selected nested depth on fresh/cold boots')
    parser.add_argument('--participant-fault', choices=('hold', 'disconnect', 'pressure'), help='disposable producer/receiver faults through Split AUTO')
    parser.add_argument('--capture-case', choices=('ordinary', 'source', 'recipient', 'timeout'), help='uninstrumented capture or disposable source/recipient/expiry lifecycle checks')
    parser.add_argument('--cursor-producer', action='store_true', help='disposable AUTO participant hides its own pointer through ordinary Notes F6')
    parser.add_argument('--cursor-preferences', action='store_true', help='exercise per-pointer cursor suppression, independent producer and nested target retirement')
    parser.add_argument('--capture', action='store_true', help='capture hidden Notes through an explicit service, inspect masked backing and present it in an independent consumer')
    parser.add_argument('--render-demand', action='store_true', help='hide a live Notes presentation while AUTO edits it, then restore and cold reopen')
    parser.add_argument('--participants', action='store_true', help='exercise Split AUTO with independent physical and synthetic input')
    parser.add_argument('--nested', action='store_true', help='exercise the installed alternative Split matrix')
    parser.add_argument('--deep', action='store_true', help='also enter Split inside Split and check bounded admission')
    parser.add_argument('--trace-storage', action='store_true', help='build with storage phase traces in the serial log')
    parser.add_argument('--trace-input', action='store_true', help='include input and compositor CPU traces')
    parser.add_argument('--pointer-stress', action='store_true', help='move the pointer continuously while waiting for nested startup and scenes')
    parser.add_argument('--interaction-delay', action='store_true', help='hold upstream polls, images and commits past the old timeout; requires --nested-desktop')
    args = parser.parse_args()
    if args.timeout <= 0:
        parser.error('timeout must be positive')
    if args.latency and (args.service_imports or args.nested or args.deep or args.nested_desktop):
        parser.error('--latency uses the ordinary desktop without other workload variants')
    if args.nested_depth != 3 and not (args.nested_desktop or args.latency or args.soak):
        parser.error('--nested-depth requires --nested-desktop, --latency or --soak')
    if (args.nested_desktop or args.latency) and args.nested_depth not in (3, 4):
        parser.error('--nested-desktop and --latency run at depth 3 or 4')
    if args.soak and (args.soak < 1 or args.latency or args.nested_desktop or args.nested or args.deep or args.service_imports or args.interaction_delay):
        parser.error('--soak takes a positive cycle count and runs alone, optionally with --pointer-stress, --relative-pointer and traces')
    if args.interaction_delay and not args.nested_desktop:
        parser.error('--interaction-delay requires --nested-desktop')
    if args.collection_pressure and not (args.soak and args.trace_storage):
        parser.error('--collection-pressure requires --soak with --trace-storage')
    if args.pointer_stress and (not (args.nested_desktop or args.soak) or args.relative_pointer):
        parser.error('--pointer-stress requires --nested-desktop or --soak with absolute input')
    if args.input_producer and (args.service_imports or args.nested or args.deep or args.nested_desktop or args.latency or args.soak or args.interaction_delay):
        parser.error('--input-producer requires the ordinary desktop workflow')
    if (args.participants or args.participant_fault or args.render_demand or args.capture or args.capture_case or args.cursor_preferences or args.cursor_producer or args.launcher_async or args.launch_async or args.binding_async or args.scene_async or args.selection_async) and (args.service_imports or args.nested or args.deep or args.nested_desktop or args.latency or args.soak or args.input_producer or args.interaction_delay):
        parser.error('participant and rendering-demand checks run their own default-desktop workflows')
    if sum(bool(v) for v in (args.participant_fault, args.participants, args.render_demand, args.capture, args.capture_case, args.cursor_preferences, args.cursor_producer, args.launcher_async, args.launch_async, args.binding_async, args.scene_async, args.selection_async)) > 1:
        parser.error('choose one participant, rendering-demand or capture workflow')
    run(args.timeout, args.trace_storage, args.relative_pointer, args.service_imports, args.nested or args.deep, args.deep, args.nested_desktop, args.trace_input, args.latency, args.pointer_stress, args.interaction_delay, args.nested_depth, args.soak, args.collection_pressure, accel=args.accel, input_producer=args.input_producer, participants=args.participants, participant_fault=args.participant_fault, render_demand=args.render_demand, masked_capture=args.capture, capture_case=args.capture_case, cursor_preferences=args.cursor_preferences, cursor_producer=args.cursor_producer, launcher_async=args.launcher_async, launch_async=args.launch_async, binding_async=args.binding_async, scene_async=args.scene_async, selection_async=args.selection_async)
