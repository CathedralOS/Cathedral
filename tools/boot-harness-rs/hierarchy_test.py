"""On copies of a one-user desktop fixture, rename its launcher and test missing paths."""
import argparse
from datetime import datetime, timezone
import os
from pathlib import Path
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


def run(source):
    root = harness.REPO
    here = Path(__file__).resolve().parent
    subprocess.run([sys.executable, str(here / 'run.py'), '--release', '--build-only'], cwd=root, check=True)
    output = harness.BUILD / 'hierarchy' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    program_list = harness.seeded_programs(root / 'source-rs/distribution/profile.json', True)
    executable = database_image.oracle()
    for mode in ('renamed', 'missing'):
        folder = output / mode
        folder.mkdir()
        shutil.copytree(harness.BUILD / 'interactive/esp', folder / 'esp')
        image = folder / 'guest.raw'
        shutil.copyfile(source, image)
        database_image.prepare(image, program_list, root / 'source-rs/distribution/migrations/readable-labels.txt', True)
        subprocess.run([str(executable), 'engines', 'volume', 'configure-fixture', str(image),
                        str(database_image.OFFSET), str(database_image.VOLUME_BYTES), mode], check=True)
        # An installer rerun must preserve the owner's edited hierarchy/configuration.
        subprocess.run([str(executable), 'engines', 'volume', 'upgrade-hierarchy', str(image),
                        str(database_image.OFFSET), str(database_image.VOLUME_BYTES)], check=True)
        qemu = harness.find_qemu()
        command = [qemu, '-machine', 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1', '-m', '128']
        command += harness.firmware_args(qemu, folder)
        command += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none', '-no-reboot', '-display', 'none']
        command += disk_image.arguments(image)

        def verify(stream, folder, serial):
            if mode == 'renamed':
                colors_test.exercise(stream, folder, serial, 1, 1, None, False)
                # The text object named Fake.exe must not occupy a third program tile.
                width, height, data = pointer_test.read_scanout(stream, folder)
                assert data[(336 * width + 120) * 3:(336 * width + 120) * 3 + 3] == bytes.fromhex('101827')
                return
            width, height, _ = pointer_test.read_scanout(stream, folder)
            x = (width - 280) // 2 + 64
            capture.command(stream, 'input-send-event', {'events': [
                {'type': 'abs', 'data': {'axis': 'x', 'value': round(x * 32767 / (width - 1))}},
                {'type': 'abs', 'data': {'axis': 'y', 'value': round((height // 2) * 32767 / (height - 1))}}]})
            for down in (True, False):
                capture.command(stream, 'input-send-event', {'events': [{'type': 'btn', 'data': {'button': 'left', 'down': down}}]})
                time.sleep(0.1)
            deadline = time.monotonic() + 30
            while True:
                log = (folder / 'serial.log').read_text(encoding='utf-8', errors='replace')
                assert 'user panic:' not in log and 'CATHEDRAL_RS_PANIC' not in log, log[-3000:]
                if 'user-start-refused' in log:
                    break
                assert time.monotonic() < deadline, log[-3000:]
                time.sleep(0.1)
            assert 'configured from realm paths' not in log
            # Wait for the restored picker rather than accepting a serial refusal alone.
            while True:
                width, height, data = pointer_test.read_scanout(stream, folder)
                if data[((height // 2 - 20) * width + x) * 3:((height // 2 - 20) * width + x) * 3 + 3] != bytes.fromhex('101827'):
                    break
                assert time.monotonic() < deadline
                time.sleep(0.1)
            capture.save_png(folder, width, height, data)

        capture.run(command, folder, 180, subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0,
                    'Cathedral desktop: ready; choose a user', verify)
    print(f'PASS: saved launcher path survives rename, extension does not grant type, missing path returns to picker; artifacts: {output}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('image', type=Path, help='one-user desktop fixture; opened only through copies')
    run(parser.parse_args().image)
