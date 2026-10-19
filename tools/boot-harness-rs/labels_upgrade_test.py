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


def run(source, fill_users=False, nested=False):
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
    database_image.prepare(path, programs, migration, app_close=True, realm_programs=harness.realm_programs(root / 'source-rs/distribution/profile.json', True))
    assert any(output.glob('guest.raw.before-compact-*.raw')), 'installer must preserve the pre-upgrade disk'
    migrated = path.read_bytes()
    database_image.prepare(path, programs, migration, app_close=True, realm_programs=harness.realm_programs(root / 'source-rs/distribution/profile.json', True))
    assert migrated == path.read_bytes(), 'unchanged second installation must not rewrite the image'
    qemu = harness.find_qemu()
    command = [qemu, '-machine', 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1', '-m', '128']
    command += harness.firmware_args(qemu, output)
    command += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none', '-no-reboot', '-display', 'none']
    command += disk_image.arguments(path)
    capture.run(command, output, 180, subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0,
                ready_marker='Cathedral desktop: ready; choose a user',
                verify=lambda stream, output, serial: desktop_test.desktop(stream, output, serial, True, fill_users=fill_users))
    if nested:
        capture.run(command, output, 180, subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0,
                    ready_marker='Cathedral desktop: ready; choose a user',
                    verify=lambda stream, output, serial: desktop_test.desktop(stream, output, serial, True, nested=True))
    print(f'PASS: existing user, tile UI, app-owned close and both original saved notes; artifacts: {output}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--nested', action='store_true', help='also launch the installed Split subtree in the migrated user')
    parser.add_argument('--fill-users', action='store_true', help='also create users up to the three-tile limit')
    parser.add_argument('image', type=Path, help='retained pre-label desktop test image; never modified')
    args = parser.parse_args()
    run(args.image, args.fill_users, args.nested)
