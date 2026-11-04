"""Boot the Omega entry/IPC witness and check successful and failed exits."""
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
    parser.add_argument('--matrix', action='store_true',
                        help='also check the stock matrix refuses an empty environment with its full exit status')
    args = parser.parse_args()
    image = prepare(ROOT / 'source/distribution/lab/user-entry/native', args.compiler)
    elf_limits.check(image)
    output = ROOT / 'build/omega-user/tests' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    for clock in (False, True):
        boot(image, output / ('clock' if clock else 'no-clock'), clock, args.timeout)
    # Use real refusals in the unchanged witness. With no quota its first open
    # fails; with one channel it closes that channel before returning failure.
    for channels, status in ((0, 102), (1, 109)):
        boot(image, output / f'channels-{channels}', False, args.timeout,
             composition={'CATHEDRAL_CHANNEL_LIMITS': str(channels)}, expected_exit=status)
    if args.matrix:
        matrix = prepare(ROOT / 'source/distribution/matrix/native', args.compiler)
        elf_limits.check(matrix)
        matrix_exit(matrix, output / 'matrix', args.timeout)
    print(f'Standalone Omega entry/IPC/parking and failure exits PASS: {output}', flush=True)


def matrix_exit(image, output, timeout):
    # Supply enough initial funding for the normal matrix image/stack, but no
    # service endpoints. It must reject that shape and finish cleanup with
    # INVALID_ARGUMENT. This high-bit status also detects exit-code truncation.
    status = (1 << 64) - 22

    def verify(stream, directory, text):
        import re
        reasons = re.findall(r'Matrix control: [0-9a-f]{16} 7 5 ([0-9a-f]{16})', text)
        if reasons != [f'{status:016x}']:
            raise RuntimeError(f'Matrix cleanup did not report its refusal; see {directory / "serial.log"}')

    boot(image, output, True, timeout, composition={'CATHEDRAL_INIT_PRIVATE_PAGES': '512'},
         expected_exit=status, verifier=verify)


def boot(image, output, clock, timeout, *, composition=None, verifier=None, expected_exit=0):
    output.mkdir(parents=True)
    # Own this test's composition; never inherit another profile's grants.
    environment = {key: value for key, value in os.environ.items() if not key.startswith('CATHEDRAL_')}
    environment.update(CATHEDRAL_INIT_ELF=str(image), CATHEDRAL_CHANNEL_LIMITS='2',
                       CATHEDRAL_CPU_LIMITS='0', CATHEDRAL_LAUNCH_COUNT='0',
                       CATHEDRAL_IMAGE_COUNT='0', CATHEDRAL_DYNAMIC_TASKS='0',
                       CATHEDRAL_INIT_CLOCK=str(int(clock)), CATHEDRAL_INIT_PRIVATE_PAGES='1',
                       CATHEDRAL_INIT_SHARED_PAGES='1')
    if composition is not None:
        environment.update(composition)
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

    stopped = f'initial program stopped: Some(Returned({expected_exit}))\n'

    def verify(stream, directory, text):
        if ('Omega IPC ready\n' in text) != (expected_exit == 0):
            raise RuntimeError(f'Omega entry/IPC witness failed; see {directory / "serial.log"}')
        if ('Omega wait ready\n' in text) != (clock and expected_exit == 0):
            raise RuntimeError(f'Clock authority/parking witness failed; see {directory / "serial.log"}')

    # A prefix can be visible before the kernel finishes printing the outcome.
    # Verify only after the complete expected exit line has arrived.
    capture.run(command, output, timeout, getattr(subprocess, 'CREATE_NO_WINDOW', 0),
                stopped, verifier or verify, allow_initial_exit=True)


if __name__ == '__main__':
    main()
