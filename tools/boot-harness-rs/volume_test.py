#!/usr/bin/env python3
"""Prove shared volume decoding and byte IO over the guest's granted ATA disk."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

import disk_image
import protocol_test
import run as harness

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
LAB = ROOT / 'tools/storage-lab'
BLOCK = 4096
SIZE = 4 * 1024 * 1024
SCRATCH = BLOCK + SIZE + BLOCK


def command(*args, cwd=ROOT):
    return subprocess.run(args, cwd=cwd, check=True, text=True, capture_output=True)


def seed(executable, output):
    source = output / 'formatted.raw'
    command(str(executable), 'engines', 'volume', 'format', str(source), str(SIZE), '1')
    volume = source.read_bytes()
    scratch = bytearray([0xc3] * BLOCK)
    scratch[:256] = volume[8192:8192 + 256]
    scratch[256:264] = (0).to_bytes(8, 'little')
    image = bytearray(b'\x5a' * BLOCK + volume + b'\xa5' * BLOCK + scratch + b'\x69' * BLOCK)
    path = output / 'guest.raw'
    with path.open('xb') as file:
        file.write(image)
        file.flush()
        os.fsync(file.fileno())
    return path, image


def run(args):
    profile = HERE / 'profiles/volume.json'
    build = [sys.executable, str(HERE / 'run.py'), '--profile', str(profile), '--build-only']
    if args.release:
        build.append('--release')
    subprocess.run(build, cwd=ROOT, check=True)
    subprocess.run(['cargo', 'build', '--locked', '--release', '--features', 'engines'], cwd=LAB, check=True)
    executable = LAB / 'target/release' / ('storage-layout.exe' if os.name == 'nt' else 'storage-layout')
    output = harness.BUILD / 'volume' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)  # Every run owns a new image; never reset a session disk.
    shutil.copytree(harness.BUILD / 'volume-interactive/esp', output / 'esp')
    path, expected = seed(executable, output)
    qemu = harness.find_qemu()
    boot = [qemu, '-machine', 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1', '-m', '128']
    boot += harness.firmware_args(qemu, output)
    boot += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none', '-no-reboot', '-display', 'none']
    boot += disk_image.arguments(path)
    flags = subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0

    def observe(name, marker):
        protocol_test.run(boot, output, args.timeout, flags, f'Cathedral volume: {marker}', abrupt=True)
        for log in ('serial.log', 'qemu.log'):
            shutil.copyfile(output / log, output / f'{name}-{log}')
        actual = path.read_bytes()
        if actual != expected:
            mismatch = next((i for i, (a, b) in enumerate(zip(actual, expected)) if a != b), min(len(actual), len(expected)))
            raise RuntimeError(f'{name}: unexpected disk mutation at {mismatch}; lengths {len(actual)}/{len(expected)}')

    expected[SCRATCH + 509:SCRATCH + 1540] = b'\x37' * 1031
    expected[SCRATCH + 4095] = 0x58
    expected[SCRATCH + 256:SCRATCH + 264] = (1).to_bytes(8, 'little')
    observe('write', 'flushed')
    observe('cold-boot', 'reopened')
    # Damage only disposable fixtures. A single damaged envelope is tolerated;
    # losing both copies is refused before any writes, with the whole image checked.
    expected[BLOCK] ^= 1
    path.write_bytes(expected)
    observe('single-copy', 'reopened')
    expected[BLOCK * 2] ^= 1
    expected[SCRATCH + 256:SCRATCH + 264] = (2).to_bytes(8, 'little')
    path.write_bytes(expected)
    observe('both-copies', 'refused')
    # Reopen an intact post-guest volume in the real host database engine.
    verified = output / 'host-reopen.raw'
    intact = bytearray(expected)
    intact[BLOCK] ^= 1
    intact[BLOCK * 2] ^= 1
    with verified.open('xb') as file:
        file.write(intact)
    result = command(str(executable), 'engines', 'volume', 'open', str(verified), str(BLOCK), str(SIZE))
    (output / 'host-open.txt').write_text(result.stdout, encoding='utf-8')
    sources = [profile, Path(__file__), HERE / 'disk_image.py', HERE / 'protocol_test.py', HERE / 'run.py',
               * (ROOT / 'source-rs/platform/libraries/storage-volume').glob('*.rs'),
               ROOT / 'source-rs/platform/libraries/storage-volume/Cargo.toml',
               * (ROOT / 'source-rs/distribution/lab/volume').glob('*.rs'),
               ROOT / 'source-rs/distribution/lab/volume/Cargo.toml',
               ROOT / 'source-rs/platform/drivers/ata-pio/lib.rs', ROOT / 'source-rs/platform/libraries/block-device/lib.rs',
               ROOT / 'source-rs/Cargo.toml', ROOT / 'source-rs/Cargo.lock', ROOT / 'source-rs/rust-toolchain.toml',
               ROOT / 'source-rs/.cargo/config.toml']
    context = {'revision': command('git', 'rev-parse', 'HEAD').stdout.strip(), 'release': args.release,
               'rustc': command('rustc', '-Vv', cwd=ROOT / 'source-rs').stdout.strip(),
               'qemu': command(qemu, '--version').stdout.strip(),
               'boot_sha256': hashlib.sha256((output / 'esp/EFI/BOOT/BOOTX64.EFI').read_bytes()).hexdigest(),
               'checks': ['guest flush', 'cold reboot', 'single-copy recovery', 'both-copy refusal', 'exact image comparison', 'host engine reopen'],
               'source_sha256': {p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in sources},
               'limits': 'Guest exercises envelope and block transport, not redb transactions. Killing QEMU does not simulate host cache or physical power loss.'}
    (output / 'context.json').write_text(json.dumps(context, indent=2) + '\n', encoding='utf-8')
    print(f'PASS: guest volume transport and cold reboot; artifacts: {output}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--release', action='store_true')
    parser.add_argument('--timeout', type=float, default=90, help='seconds per boot')
    args = parser.parse_args()
    if args.timeout <= 0:
        parser.error('timeout must be positive')
    run(args)
