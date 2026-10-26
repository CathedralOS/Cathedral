#!/usr/bin/env python3
"""Exercise private-root IPC content publication, refusals and cold recovery."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
import disk_image
import database_image
import protocol_test
import run as harness
import volume_test

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]

def run(args):
    name = 'objects-retention' if args.retention else 'objects'
    profile = HERE / f'profiles/{name}.json'
    build = [sys.executable, str(HERE / 'run.py'), '--profile', str(profile), '--build-only']
    if args.release: build.append('--release')
    subprocess.run(build, cwd=ROOT, check=True)
    lab = ROOT / 'tools/storage-lab'
    subprocess.run(['cargo', 'build', '--locked', '--release', '--features', 'engines'], cwd=lab, check=True)
    executable = lab / 'target/release' / ('storage-layout.exe' if os.name == 'nt' else 'storage-layout')
    output = harness.BUILD / 'objects' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    shutil.copytree(harness.BUILD / f'{name}-interactive/esp', output / 'esp')
    # The object provider owns the database extent, not the smaller volume-IO
    # lab's extent. Keep neighbor guards while matching the provider's 8 MiB.
    size = database_image.VOLUME_BYTES
    path, original = volume_test.seed(executable, output, size)
    qemu = harness.find_qemu()
    command = [qemu, '-machine', 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1', '-m', '128']
    command += harness.firmware_args(qemu, output)
    command += ['-drive', 'format=raw,file=fat:rw:esp', '-nic', 'none', '-monitor', 'none', '-no-reboot', '-display', 'none']
    command += disk_image.arguments(path)
    if args.ata_trace:
        (output / 'ata-events').write_text('ide_bus_exec_cmd\nide_sector_read\nide_sector_write\nblk_co_preadv\nblk_co_pwritev\n', encoding='utf-8')
        command += ['-trace', 'events=ata-events,file=ata-trace.log']
    flags = subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0
    stages = [(9, 'pinned revision 9'), (25, 'collected revision 25'), (25, 'recovered content and private roots passed')] if args.retention else [(1, 'acknowledged revision 1'), (2, 'recovered content and private roots passed')]
    for boot, (revision, stage) in enumerate(stages, 1):
        marker = f'Cathedral objects: {stage}'
        protocol_test.run(command, output, args.timeout, flags,
                          marker, abrupt=True)
        serial = (output / 'serial.log').read_text(encoding='utf-8', errors='replace').replace('Cathedral Rust lab: ', '')
        assert serial.count('Cathedral objects: ' + ('retention and bulk checks passed' if args.retention else 'client content and refusal checks passed')) == 2
        assert serial.count('Cathedral objects: shared root connection ready') == 2, serial
        assert serial.count('Cathedral objects: shared owner control ready') == 1, serial
        if boot == len(stages):
            assert 'Cathedral objects heap: closed live=0' in serial
            assert serial.count('Cathedral objects: shared owner resources reclaimed') == 1, serial
        metrics = [line for line in serial.splitlines() if line.startswith('Cathedral objects heap:')]
        assert any(': recovered ' in line for line in metrics)
        if boot < 3: assert any(': prepared ' in line for line in metrics)
        assert sum(': committed ' in line for line in metrics) == ([18, 32, 0][boot-1] if args.retention else 2)
        assert all('failures=0' in line for line in metrics)
        (output / f'heap-{boot}.txt').write_text('\n'.join(metrics) + '\n', encoding='utf-8')
        for name in (('serial.log', 'qemu.log', 'ata-trace.log') if args.ata_trace else ('serial.log', 'qemu.log')):
            shutil.copyfile(output / name, output / f'boot-{boot}-{name}')
        actual = path.read_bytes()
        assert len(actual) == len(original)
        assert actual[:4096] == original[:4096]
        assert actual[4096 + size:] == original[4096 + size:]
        # Verify a copy so host recovery/close cannot clean up the guest image
        # before the next cold guest opens it.
        snapshot = output / f'host-reopen-{boot}.raw'
        with snapshot.open('xb') as file: file.write(actual)
        result = subprocess.run([str(executable), 'engines', 'volume', ('verify-retention' if args.retention else 'verify-objects'), str(snapshot),
                                 '4096', str(size), str(revision)], cwd=lab,
                                check=True, capture_output=True, text=True)
        (output / f'host-{boot}.txt').write_text(result.stdout, encoding='utf-8')
        print(result.stdout, end='')
    sources = [profile, Path(__file__), HERE / 'volume_test.py', HERE / 'database_image.py',
               ROOT / 'source-rs/Cargo.toml', ROOT / 'source-rs/Cargo.lock',
               ROOT / 'source-rs/platform/libraries/block-device/lib.rs',
               * (ROOT / 'source-rs/distribution/lab/objects').rglob('*.rs'),
               * (ROOT / 'source-rs/platform/libraries/user-heap').glob('*.rs'),
               * (ROOT / 'source-rs/platform/libraries/storage-volume').rglob('*.rs'),
               * (ROOT / 'source-rs/platform/libraries/content-layout').rglob('*.rs'),
               * (ROOT / 'source-rs/platform/libraries/database-objects').rglob('*.rs'),
               * (ROOT / 'source-rs/platform/services/storage').glob('*.rs'),
               ROOT / 'source-rs/platform/contracts/storage.rs',
               ROOT / 'tools/storage-lab/engines/volume/objects.rs']
    context = {'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
               'release': args.release, 'retention': args.retention,
               'rustc': subprocess.check_output(['rustc', '--version'], cwd=ROOT / 'source-rs', text=True).strip(),
               'qemu': subprocess.check_output([qemu, '--version'], text=True).splitlines()[0],
               'efi_sha256': hashlib.sha256((output / 'esp/EFI/BOOT/BOOTX64.EFI').read_bytes()).hexdigest(), 'checks': ['private root isolation', 'real content publication', 'stale revision refusal',
               'oversized request refusal', 'attachment size validation', 'retained capacity refusal',
               'cold guest content recovery', 'independent exact host records', 'heap measurements', 'neighbour preservation'] if not args.retention else [
               'durable revision pin', 'bounded collection', 'stale release refusal', 'collected revision unavailable',
               'sealed bulk reads', 'zero padding', 'reader pressure and reuse', 'cold guest content recovery',
               'independent exact host records', 'heap measurements', 'neighbour preservation'],
               'source_sha256': {p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in sources},
               'limits': 'Fixed two- or three-boot trace. QEMU kill does not emulate host cache loss or physical power failure.'}
    (output / 'context.json').write_text(json.dumps(context, indent=2) + '\n', encoding='utf-8')
    print(f'PASS: private content publication and cold recovery; artifacts: {output}')

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--release', action='store_true')
    parser.add_argument('--retention', action='store_true', help='three-boot pin, reclamation and bulk read proof')
    parser.add_argument('--ata-trace', action='store_true', help='record QEMU ATA command and block IO events')
    parser.add_argument('--timeout', type=float, default=120)
    args = parser.parse_args()
    if args.timeout <= 0: parser.error('timeout must be positive')
    run(args)
