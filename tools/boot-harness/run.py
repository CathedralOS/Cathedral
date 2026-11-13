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
        return [str(Path(binary).resolve())], ROOT
    # A checkout is deliberately preferred to a possibly stale installed release.
    if (repository / 'Cargo.toml').is_file():
        # Rustup and Cargo discover toolchain/configuration from cwd, not
        # --manifest-path. Build with the producer checkout's own policy.
        return (['cargo', 'run', '--locked', '--manifest-path',
                 str(repository / 'Cargo.toml'), '-p', 'omega', '--'], repository)
    installed = shutil.which('omega')
    if installed:
        return [installed], ROOT
    raise ValueError('No Omega compiler: set OMEGA_BIN or OMEGA_REPO.')


def run_checked(command, *, cwd=ROOT):
    print('+ ' + subprocess.list2cmdline([str(word) for word in command]), flush=True)
    subprocess.run(command, cwd=cwd, stdin=subprocess.DEVNULL, check=True)


def create_run_directory(parent):
    parent.mkdir(parents=True, exist_ok=True)
    # Retain diagnostics, but never share mutable ESP, firmware or serial
    # files between concurrent invocations of the harness.
    return Path(tempfile.mkdtemp(prefix='run-', dir=parent))


def build_image(command, entry, target, directory, *, compiler_directory=ROOT):
    # A fresh directory prevents a successful check-only invocation or a failed
    # build from accidentally booting an image left by an earlier compiler.
    directory.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='compile-', dir=directory) as temporary:
        output = Path(temporary)
        run_checked(command + ['--build-dir', str(output), '--target', target, str(entry)],
                    cwd=compiler_directory)
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
    after_cpu = after_tables[cpu.end():]
    root = re.search(rb'(?:^|\n)Root located\r?\n', after_cpu)
    if root is None:
        raise ValueError('Expected firmware root location in the final map after CPU checked.')
    after_root = after_cpu[root.end():]
    observed = re.search(rb'(?:^|\n)Root read\r?\n', after_root)
    if observed is None:
        raise ValueError('Expected admitted firmware root snapshot after root location.')
    after_observed = after_root[observed.end():]
    retained = re.search(rb'(?:^|\n)Tables retained\r?\n', after_observed)
    if retained is None:
        raise ValueError('Expected complete reachable firmware-table retention after root read.')
    after_retained = after_observed[retained.end():]
    retired = re.search(rb'(?:^|\n)Mapping retired\r?\n', after_retained)
    if retired is None:
        raise ValueError('Expected completed mapping retirement and original root readback after table retention.')
    after_retired = after_retained[retired.end():]
    mapping = re.search(rb'(?:^|\n)Mapping checked\r?\n', after_retired)
    if mapping is None:
        raise ValueError('Expected mapped writes and physical readback after mapping retirement.')
    after_mapping = after_retired[mapping.end():]
    segments = re.search(rb'(?:^|\n)GDT checked\r?\n', after_mapping)
    if segments is None:
        raise ValueError('Expected descriptor-table load, readback and restoration after mapping checked.')
    after_segments = after_mapping[segments.end():]
    task = re.search(rb'(?:^|\n)TSS checked\r?\n', after_segments)
    if task is None:
        raise ValueError('Expected task-register selection and complete busy descriptor readback after GDT checked.')
    if re.search(rb'(?:^|\n)Segments ready\r?\n', after_segments[task.end():]) is None:
        raise ValueError('Expected kernel code/data selector readback and retained task-state checks after TSS checked.')


def check_boot_output(output):
    # Keep the startup-only checker usable at the pre-Work observation cut.
    # Normal smoke acceptance additionally requires the authored timer result.
    check_boot_milestones(output)
    if b'Timer failed' in output:
        raise ValueError('The kernel reported timer failure.')
    ready = list(re.finditer(rb'(?:^|\n)Segments ready\r?\n', output))
    after_ready = output[ready[-1].end():]
    if re.search(rb'(?:^|\n)\.{8}Timer wakes 8\r?\n', after_ready) is None:
        raise ValueError('Expected exactly eight timer wake dots and a complete Timer wakes 8 line after Segments ready.')


def main(arguments=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', default=os.environ.get('OMEGA_BIN'))
    parser.add_argument('--omega-repo', type=Path,
                        default=Path(os.environ.get('OMEGA_REPO', ROOT.parent / 'Omega')))
    parser.add_argument('--build-only', action='store_true')
    parser.add_argument('--prepare', action='store_true',
                        help='run ordinary omega update for dependency review before building')
    parser.add_argument('--bootstrap-assumptions', type=Path,
                        default=os.environ.get('OMEGA_BOOTSTRAP_ASSUMPTIONS'),
                        help='pass a reviewed bootstrap condition record to Omega')
    parser.add_argument('--interrupt-delivery-assumptions', type=Path,
                        default=os.environ.get('OMEGA_INTERRUPT_DELIVERY_ASSUMPTIONS'),
                        help='pass a reviewed installed-delivery condition record to Omega')
    parser.add_argument('--build-dir', type=Path, default=ROOT / 'build/boot-harness')
    parser.add_argument('--smoke', action='store_true',
                        help='require all ordered boot milestones and exactly eight timer wakes')
    parser.add_argument('--timings', action='store_true',
                        help='print compiler stage durations during the fresh build')
    parser.add_argument('--timeout', type=float, default=30)
    options = parser.parse_args(arguments)
    if options.timeout <= 0:
        parser.error('--timeout must be positive')
    try:
        profile = json.loads((ROOT / 'source/distribution/profile.json').read_text())
        entry = (ROOT / 'source' / profile['boot_entry']).resolve()
        target = profile['target']
        command, compiler_directory = compiler_command(options.compiler, options.omega_repo.resolve())
        if options.prepare:
            run_checked(command + ['update', '--project', str(entry.parent), '--target', target],
                        cwd=compiler_directory)
        if options.timings:
            command = command + ['--timings']
        # Keep platform assumptions explicit. Omega checks the selected records
        # against this compilation; the harness does not infer them from QEMU.
        for flag, record in [
            ('--bootstrap-assumptions', options.bootstrap_assumptions),
            ('--interrupt-delivery-assumptions', options.interrupt_delivery_assumptions),
        ]:
            if record is not None:
                command = command + [flag, str(record.resolve())]
        directory = create_run_directory(options.build_dir.resolve())
        print(f'Run directory: {directory}', flush=True)
        image = build_image(command, entry, target, directory,
                            compiler_directory=compiler_directory)
        print(f'Fresh image: {image}', flush=True)
        if options.build_only:
            return 0
        qemu_name = shutil.which('qemu-system-x86_64')
        qemu = Path(qemu_name or 'C:/Program Files/qemu/qemu-system-x86_64.exe')
        if not qemu.is_file():
            raise ValueError('qemu-system-x86_64 unavailable; use --build-only or install QEMU.')
        serial = directory / 'serial.log'
        invocation = [str(qemu), '-machine', 'q35', '-smp', '1',
                      *firmware_arguments(qemu, directory),
                      '-drive', f'format=raw,file=fat:rw:{directory / "esp"}',
                      '-net', 'none', '-no-reboot', '-monitor', 'none',
                      '-serial', f'file:{serial}' if options.smoke else 'stdio',
                      '-debugcon', f'file:{directory / "ovmf-debug.log"}',
                      '-global', 'isa-debugcon.iobase=0x402']
        if options.smoke:
            invocation += ['-display', 'none']
        # Record the exact VM configuration, including the single virtual CPU,
        # before launch. This does not establish source-level CPU authority.
        (directory / 'qemu-command.json').write_text(
            json.dumps(invocation, indent=2) + '\n', encoding='utf-8')
        if not options.smoke:
            run_checked(invocation)
            return 0
        serial.write_bytes(b'')
        # The guest keeps its installation live, so timeout is not success by itself.
        # This run must produce every startup marker and the eight-wake result.
        try:
            subprocess.run(invocation, stdin=subprocess.DEVNULL, check=True,
                           timeout=options.timeout)
        except subprocess.TimeoutExpired:
            pass
        output = serial.read_bytes()
        try:
            check_boot_output(output)
        except ValueError as error:
            raise ValueError(f'{error} Inspect {serial}') from error
        print(f'Boot milestones and eight timer wakes observed: {serial}')
        return 0
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f'Cathedral Omega boot failed: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
