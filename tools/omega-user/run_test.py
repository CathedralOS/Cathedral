"""Boot the standalone Omega entry/IPC witness under the ordinary Rust kernel."""
import argparse
from datetime import datetime, timezone
import os
from pathlib import Path
import shutil
import subprocess
import sys

from build import ROOT, prepare

sys.path.insert(0, str(ROOT / 'tools/boot-harness-rs'))
import capture
import elf_limits
import run as harness


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path)
    parser.add_argument('--timeout', type=int, default=60)
    args = parser.parse_args()
    image = prepare(ROOT / 'source/distribution/lab/user-entry/native', args.compiler)
    elf_limits.check(image)
    output = ROOT / 'build/omega-user/tests' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    for clock in (False, True):
        boot(image, output / ('clock' if clock else 'no-clock'), clock, args.timeout)
    print(f'Standalone Omega entry/IPC/parking PASS: {output}', flush=True)


def boot(image, output, clock, timeout):
    output.mkdir(parents=True)
    # Own this test's composition; never inherit another profile's grants.
    environment = {key: value for key, value in os.environ.items() if not key.startswith('CATHEDRAL_')}
    environment.update(CATHEDRAL_INIT_ELF=str(image), CATHEDRAL_CHANNEL_LIMITS='2',
                       CATHEDRAL_CPU_LIMITS='0', CATHEDRAL_LAUNCH_COUNT='0',
                       CATHEDRAL_IMAGE_COUNT='0', CATHEDRAL_DYNAMIC_TASKS='0',
                       CATHEDRAL_INIT_CLOCK=str(int(clock)), CATHEDRAL_INIT_PRIVATE_PAGES='1',
                       CATHEDRAL_INIT_SHARED_PAGES='1')
    cargo = ROOT / 'build/boot-harness-rs/cargo'
    with (output / 'kernel-build.log').open('w', encoding='utf-8') as log:
        subprocess.run(['cargo', 'build', '--locked', '--release', '--package', 'cathedral-boot-uefi',
                        '--features', 'bundled-user',
                        '--target', 'x86_64-unknown-uefi', '--target-dir', str(cargo)],
                       cwd=ROOT / 'source-rs', env=environment, stdout=log, stderr=subprocess.STDOUT, check=True)
    esp = output / 'esp/EFI/BOOT'
    esp.mkdir(parents=True)
    shutil.copyfile(cargo / 'x86_64-unknown-uefi/release/cathedral-boot-uefi.efi', esp / 'BOOTX64.EFI')
    qemu = harness.find_qemu()
    command = [qemu, '-machine', 'q35', '-cpu', 'qemu64', '-accel', 'tcg', '-smp', '1',
               '-m', '256', '-display', 'none', '-nic', 'none', '-monitor', 'none', '-no-reboot']
    command += harness.firmware_args(qemu, output)
    command += ['-drive', 'format=raw,file=fat:rw:esp']

    def verify(stream, directory, text):
        if 'Omega IPC ready\n' not in text or 'initial program stopped: Some(Returned(0))' not in text:
            raise RuntimeError(f'Omega entry/IPC witness failed; see {directory / "serial.log"}')
        if ('Omega wait ready\n' in text) != clock:
            raise RuntimeError(f'Clock authority/parking witness failed; see {directory / "serial.log"}')

    capture.run(command, output, timeout, getattr(subprocess, 'CREATE_NO_WINDOW', 0),
                'initial program stopped:', verify, allow_initial_exit=True)


if __name__ == '__main__':
    main()
