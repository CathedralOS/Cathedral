"""Bootstrap rollback, bounded retry and terminal diagnostics on disposable F5 media."""
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
import colors_test
import database_image
import disk_image
import pointer_test
import run as harness

STAGES = {'display': 0, 'input': 1, 'root': 2}
MODES = {'death': 1, 'stall': 2, 'malformed': 3, 'refusal': 4, 'bad-ready': 5}


def exercise(stream, folder, stage, mode, repeat, dynamic_tasks):
    def log():
        text = (folder / 'serial.log').read_text(errors='replace').replace('Cathedral Rust lab: ', '')
        assert 'user panic:' not in text and 'CATHEDRAL_RS_PANIC' not in text, text[-4000:]
        return text

    deadline = time.monotonic() + 60
    marker = ('bootstrap exhausted; restart Cathedral' if stage == 'display' else 'restart screen visible') if repeat else 'ready; choose a user'
    while marker not in log():
        assert time.monotonic() < deadline, f'missing {marker}: {folder}; {log()[-4000:]}'
        time.sleep(0.05)
    text = log()
    failures = re.findall(r'bootstrap-failed (\d+) (\d+) (\d+)', text)
    count = 2 if repeat else 1
    assert len(failures) == count and text.count('bootstrap attempt') == 2, text
    for number, found_stage, error in failures:
        assert int(found_stage) == STAGES[stage]
        assert int(number) < count
        expected = {'death': -32, 'stall': -110, 'malformed': -5, 'refusal': -12, 'bad-ready': -5}[mode]
        assert int(error) == expected % (1 << 64), (mode, error, text)
    reclaimed = re.findall(r'bootstrap-reclaimed ([\d ]+)', text)
    assert len(reclaimed) == count and len(set(reclaimed)) == 1, text
    assert all(int(line.split()[0]) == dynamic_tasks for line in reclaimed), 'process capacity not restored'
    transport = re.findall(r'bootstrap-transport-reclaimed (\d+) (\d+) (\d+) (\d+)', text)
    assert len(transport) == count and all(tuple(map(int, row)) == (0, 0, 0, 0) for row in transport), transport
    # The guest compares every rollback to the measured pre-attempt limits,
    # frame charge, page charge and channel charge; panics fail this fixture.
    if mode == 'bad-ready':
        assert text.count('bootstrap readiness request received') == count, text
    if repeat:
        assert text.count('shared root control ready') == (2 if stage == 'input' else 0), text
        assert 'storage startup attempt' not in text and 'bootstrap accepted' not in text
        if stage != 'display':
            font = bytes.fromhex((harness.WORKSPACE / 'platform/libraries/bitmap-font/font.hex').read_text())
            width, height, data = pointer_test.read_scanout(stream, folder)
            for title, y in [('COULD NOT START', height // 2 - 24), ('PLEASE RESTART', height // 2 + 12)]:
                x = (width - len(title) * 12) // 2
                for i, ch in enumerate(title):
                    for row, bits in enumerate(font[(ord(ch) - 32) * 7:(ord(ch) - 31) * 7]):
                        for col in range(5):
                            offset = ((y + row * 2) * width + x + i * 12 + col * 2) * 3
                            assert (data[offset:offset + 3] == b'\xff\xff\xff') == bool(bits & (1 << (4 - col)))
            capture.save_png(folder, width, height, data)
        time.sleep(1)
        assert log().count('bootstrap attempt') == 2, 'unbounded bootstrap restart'
    else:
        assert text.count('bootstrap accepted') == 1 and text.count('storage startup attempt') == 1
        assert text.count('shared root control ready') == (2 if stage == 'input' else 1), text
        colors_test.exercise(stream, folder, text, 1, 1, None)
    # Retirement is asynchronous; check after actual input delivery on success.
    text = log()
    for marker in ('shared input control ready', 'shared source authority checked', 'startup control retired'):
        assert text.count(marker) == (0 if repeat else 1), (marker, text)
    print(f'PASS: {stage} {mode} {"persistent" if repeat else "transient"}; {folder}', flush=True)


def run(source, selected):
    original = source.read_bytes()
    output = harness.BUILD / 'bootstrap' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    completed = []
    for stage, index in STAGES.items():
        for mode, value in MODES.items():
            for repeat in (False, True):
                case = f'{stage}-{mode}-{"repeat" if repeat else "once"}'
                if selected and case not in selected:
                    continue
                folder = output / case
                folder.mkdir()
                composition = json.loads((harness.WORKSPACE / 'distribution/profile.json').read_text())
                initial = composition['startup']['initial']
                initial['features'].append('bootstrap-' + stage)
                if mode != 'death':
                    initial['features'].append('bootstrap-' + mode)
                if repeat:
                    initial['features'].append('bootstrap-repeat')
                composition['startup']['launches'][index].setdefault('features', []).append('bootstrap-probes')
                profile = folder / 'bootstrap-check.json'
                profile.write_text(json.dumps(composition, indent=2), encoding='utf-8')
                subprocess.run([sys.executable, str(Path(__file__).with_name('run.py')), '--release', '--build-only',
                                '--profile', str(profile)], cwd=harness.REPO, check=True)
                shutil.copytree(harness.BUILD / 'bootstrap-check-interactive/esp', folder / 'esp')
                image = folder / 'guest.raw'
                image.write_bytes(original)
                database_image.prepare(image, harness.seeded_programs(profile, True),
                                       harness.WORKSPACE / 'distribution/migrations/readable-labels.txt', True)
                before = image.read_bytes()
                qemu = harness.find_qemu()
                command = [qemu, '-machine', 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1', '-m', '128']
                command += harness.firmware_args(qemu, folder)
                command += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none', '-no-reboot', '-display', 'none']
                command += disk_image.arguments(image)
                capture.run(command, folder, 180, subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0,
                            'Cathedral desktop: bootstrap attempt',
                            lambda stream, folder, serial: exercise(stream, folder, stage, mode, repeat, composition['startup']['dynamic_tasks']),
                            allow_initial_exit=repeat and stage == 'display')
                if repeat:
                    assert image.read_bytes() == before, 'failed bootstrap changed storage'
                completed.append(case)
    assert completed, 'no matching cases'
    assert source.read_bytes() == original, 'source fixture changed'
    (output / 'result.json').write_text(json.dumps({'passed': True, 'cases': completed}), encoding='utf-8')
    print(f'PASS: bootstrap suite; {output}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('image', type=Path, help='one-user fixture; only copies are modified')
    parser.add_argument('--case', action='append', help='e.g. input-stall-once; default: all 30 cases')
    args = parser.parse_args()
    run(args.image, args.case)
