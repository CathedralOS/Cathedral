#!/usr/bin/env python3
"""Build Cathedral with Omega, then boot only that invocation's UEFI image."""

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]


def compiler_command(binary, repository):
    if binary:
        return [str(Path(binary).resolve())]
    # A checkout is deliberately preferred to a possibly stale installed release.
    if (repository / 'Cargo.toml').is_file():
        return ['cargo', 'run', '--locked', '--manifest-path',
                str(repository / 'Cargo.toml'), '-p', 'omega', '--']
    installed = shutil.which('omega')
    if installed:
        return [installed]
    raise ValueError('No Omega compiler: set OMEGA_BIN or OMEGA_REPO.')


def run_checked(command, *, cwd=ROOT):
    print('+ ' + subprocess.list2cmdline([str(word) for word in command]), flush=True)
    subprocess.run(command, cwd=cwd, stdin=subprocess.DEVNULL, check=True)


def create_run_directory(parent):
    parent.mkdir(parents=True, exist_ok=True)
    # Retain diagnostics, but never share mutable ESP, firmware or serial
    # files between concurrent invocations of the harness.
    return Path(tempfile.mkdtemp(prefix='run-', dir=parent))


def build_image(command, entry, target, directory):
    # A fresh directory prevents a successful check-only invocation or a failed
    # build from accidentally booting an image left by an earlier compiler.
    directory.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='compile-', dir=directory) as temporary:
        output = Path(temporary)
        run_checked(command + ['--build-dir', str(output), '--target', target, str(entry)])
        image = output / 'omega-program.exe'
        if not image.is_file():
            raise ValueError('Omega returned success without publishing omega-program.exe.')
        destination = directory / 'esp/EFI/BOOT/BOOTX64.EFI'
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(image, destination)
        return destination


def firmware_arguments(qemu, directory):
    combined = os.environ.get('OVMF')
    if combined:
        firmware = Path(combined).resolve()
        if not firmware.is_file():
            raise ValueError(f'OVMF does not name a firmware file: {firmware}')
        return ['-bios', str(firmware)]
    candidates = [
        (qemu.parent / 'share/edk2-x86_64-code.fd', qemu.parent / 'share/edk2-i386-vars.fd'),
        (qemu.parent / '../share/qemu/edk2-x86_64-code.fd',
         qemu.parent / '../share/qemu/edk2-i386-vars.fd'),
        (Path('/usr/share/OVMF/OVMF_CODE.fd'), Path('/usr/share/OVMF/OVMF_VARS.fd')),
        (Path('/usr/share/edk2/x64/OVMF_CODE.fd'), Path('/usr/share/edk2/x64/OVMF_VARS.fd')),
        (ROOT / 'reference_code/rust-osdev/ovmf-prebuilt/OVMF_CODE.fd',
         ROOT / 'reference_code/rust-osdev/ovmf-prebuilt/OVMF_VARS.fd'),
    ]
    for code, variables in candidates:
        if code.is_file() and variables.is_file():
            code_copy = directory / 'ovmf_code.fd'
            variable_copy = directory / 'ovmf_vars.fd'
            shutil.copyfile(code, code_copy)
            shutil.copyfile(variables, variable_copy)
            return ['-drive', f'if=pflash,format=raw,readonly=on,file={code_copy}',
                    '-drive', f'if=pflash,format=raw,file={variable_copy}']
    raise ValueError('OVMF firmware unavailable; set OVMF or install QEMU firmware.')


def check_boot_milestones(output):
    greeting = output.find(b'Hello from Omega')
    owned = re.search(rb'(?:^|[\r\n])Owned [0-9]+\+? MiB\r?\n', output)
    if greeting < 0 or owned is None or owned.start() <= greeting:
        raise ValueError('Expected greeting followed by a complete owned-memory report.')
    after_owned = output[owned.end():]
    handoff = re.search(rb'(?:^|\n)Boot map ready\r?\n', after_owned)
    if handoff is None:
        raise ValueError('Expected complete boot-map handoff after the owned-memory report.')
    kernel = output.find(b'Kernel entered\r\n', owned.end() + handoff.end())
    if kernel < 0:
        raise ValueError('Expected selected kernel entry after the boot-map handoff.')
    memory = output.find(b'Memory split\r\n', kernel + len(b'Kernel entered\r\n'))
    if memory < 0:
        raise ValueError('Expected checked memory partition after kernel entry.')
    after_partition = output[memory + len(b'Memory split\r\n'):]
    checked = re.search(rb'(?:^|\n)Memory checked\r?\n', after_partition)
    if checked is None:
        raise ValueError('Expected resident initialization, read/write and extraction after memory split.')
    after_checked = after_partition[checked.end():]
    paired = re.search(rb'(?:^|\n)Memory paired\r?\n', after_checked)
    if paired is None:
        raise ValueError('Expected two coexisting resident allocations after memory checked.')
    after_paired = after_checked[paired.end():]
    page = re.search(rb'(?:^|\n)Page ready\r?\n', after_paired)
    if page is None:
        raise ValueError('Expected aligned page initialization and zero readback after memory paired.')
    after_page = after_paired[page.end():]
    tables = re.search(rb'(?:^|\n)Tables ready\r?\n', after_page)
    if tables is None:
        raise ValueError('Expected resident page-table links and unused-slot readback after page ready.')
    after_tables = after_page[tables.end():]
    cpu = re.search(rb'(?:^|\n)CPU checked\r?\n', after_tables)
    if cpu is None:
        raise ValueError('Expected supported CPU paging profile after tables ready.')
    if re.search(rb'(?:^|\n)Root located\r?\n', after_tables[cpu.end():]) is None:
        raise ValueError('Expected firmware root location in the final map after CPU checked.')


def main(arguments=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', default=os.environ.get('OMEGA_BIN'))
    parser.add_argument('--omega-repo', type=Path,
                        default=Path(os.environ.get('OMEGA_REPO', ROOT.parent / 'Omega')))
    parser.add_argument('--build-only', action='store_true')
    parser.add_argument('--prepare', action='store_true',
                        help='run ordinary omega update for dependency review before building')
    parser.add_argument('--build-dir', type=Path, default=ROOT / 'build/boot-harness')
    parser.add_argument('--smoke', action='store_true',
                        help='require ordered boot milestones through CPU checked and Root located')
    parser.add_argument('--timeout', type=float, default=30)
    options = parser.parse_args(arguments)
    if options.timeout <= 0:
        parser.error('--timeout must be positive')
    try:
        profile = json.loads((ROOT / 'source/distribution/profile.json').read_text())
        entry = (ROOT / 'source' / profile['boot_entry']).resolve()
        target = profile['target']
        command = compiler_command(options.compiler, options.omega_repo.resolve())
        if options.prepare:
            run_checked(command + ['update', '--project', str(entry.parent), '--target', target])
        directory = create_run_directory(options.build_dir.resolve())
        print(f'Run directory: {directory}', flush=True)
        image = build_image(command, entry, target, directory)
        print(f'Fresh image: {image}', flush=True)
        if options.build_only:
            return 0
        qemu_name = shutil.which('qemu-system-x86_64')
        qemu = Path(qemu_name or 'C:/Program Files/qemu/qemu-system-x86_64.exe')
        if not qemu.is_file():
            raise ValueError('qemu-system-x86_64 unavailable; use --build-only or install QEMU.')
        serial = directory / 'serial.log'
        invocation = [str(qemu), '-machine', 'q35', *firmware_arguments(qemu, directory),
                      '-drive', f'format=raw,file=fat:rw:{directory / "esp"}',
                      '-net', 'none', '-no-reboot', '-monitor', 'none',
                      '-serial', f'file:{serial}' if options.smoke else 'stdio',
                      '-debugcon', f'file:{directory / "ovmf-debug.log"}',
                      '-global', 'isa-debugcon.iobase=0x402']
        if not options.smoke:
            run_checked(invocation)
            return 0
        invocation += ['-display', 'none']
        serial.write_bytes(b'')
        # Idle is the expected boot endpoint, so timeout is not success by itself.
        # The guest must first produce every required milestone in this run.
        try:
            subprocess.run(invocation, stdin=subprocess.DEVNULL, check=True,
                           timeout=options.timeout)
        except subprocess.TimeoutExpired:
            pass
        output = serial.read_bytes()
        try:
            check_boot_milestones(output)
        except ValueError as error:
            raise ValueError(f'{error} Inspect {serial}') from error
        print(f'Boot milestones observed: {serial}')
        return 0
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f'Cathedral Omega boot failed: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
