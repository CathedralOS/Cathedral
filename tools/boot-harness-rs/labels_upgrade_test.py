"""Replay an existing pre-label desktop fixture through the ordinary installer.

Input is a retained desktop_test.py guest.raw with its saved 'hello cathedral'
notes. Only a copy is opened. Host unit tests cover custom-code preservation,
record integrity and idempotence; this checks the old user's actual boot/UI.
"""
import argparse
from datetime import datetime, timezone
import os
from pathlib import Path
import shutil
import subprocess
import sys
import capture
import database_image
import desktop_test
import disk_image
import run as harness


def run(source):
    root = desktop_test.ROOT
    here = Path(__file__).resolve().parent
    subprocess.run([sys.executable, str(here / 'run.py'), '--release', '--build-only'], cwd=root, check=True)
    output = harness.BUILD / 'labels-upgrade' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    shutil.copytree(harness.BUILD / 'interactive/esp', output / 'esp')
    path = output / 'guest.raw'
    shutil.copyfile(source, path)
    programs = harness.seeded_programs(root / 'source-rs/distribution/profile.json', True)
    migration = root / 'source-rs/distribution/migrations/readable-labels.txt'
    database_image.prepare(path, programs, migration)
    assert path.with_name(path.name + '.before-readable-labels.raw').exists(), 'fixture must exercise an upgrade'
    migrated = path.read_bytes()
    database_image.prepare(path, programs, migration)
    assert migrated == path.read_bytes(), 'unchanged second installation must not rewrite the image'
    qemu = harness.find_qemu()
    command = [qemu, '-machine', 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1', '-m', '128']
    command += harness.firmware_args(qemu, output)
    command += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none', '-no-reboot', '-display', 'none']
    command += disk_image.arguments(path)
    capture.run(command, output, 180, subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0,
                ready_marker='Cathedral desktop: ready; choose a user',
                verify=lambda stream, output, serial: desktop_test.desktop(stream, output, serial, True))
    print(f'PASS: existing user, readable program labels and both original saved notes; artifacts: {output}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('image', type=Path, help='retained pre-label desktop test image; never modified')
    run(parser.parse_args().image)
