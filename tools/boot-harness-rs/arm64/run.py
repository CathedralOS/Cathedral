#!/usr/bin/env python3
"""Build and boot the single-CPU ARM64 portability proof on QEMU virt."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fault', choices=('guard', 'text-write', 'rodata-write', 'heap-execute', 'stack-execute', 'heap-alias-execute', 'stack-alias-execute'))
    parser.add_argument('--release', action='store_true')
    parser.add_argument('--kernel-only', action='store_true')
    parser.add_argument('--build-only', action='store_true')
    parser.add_argument('--timeout', type=int, default=90)
    parser.add_argument('--qemu', default=shutil.which('qemu-system-aarch64') or
                        'C:/Program Files/qemu/qemu-system-aarch64.exe')
    parser.add_argument('--firmware', type=Path)
    args = parser.parse_args()
    source = ROOT / 'source-rs'
    out = ROOT / 'build/arm64'
    boot = out / 'esp/EFI/BOOT'
    boot.mkdir(parents=True, exist_ok=True)
    mode = 'release' if args.release else 'debug'
    profile = ['--release'] if args.release else []
    env = os.environ.copy()
    features = f'fault-{args.fault}' if args.fault else 'smoke-test'
    if not args.kernel_only:
        subprocess.run(['cargo', 'build', '--locked', '-p', 'cathedral-portability-lab',
                        '--target', 'aarch64-unknown-none', *profile], cwd=source, check=True)
        env['CATHEDRAL_ARM64_ELF'] = str(source / 'target/aarch64-unknown-none' /
                                       mode / 'cathedral-portability-lab')
        features += ',bundled-user'
    subprocess.run(['cargo', 'build', '--locked', '-p', 'cathedral-boot-uefi',
                    '--target', 'aarch64-unknown-uefi', '--features', features, *profile],
                   cwd=source, env=env, check=True)
    shutil.copy2(source / 'target/aarch64-unknown-uefi' / mode / 'cathedral-boot-uefi.efi',
                 boot / 'BOOTAA64.EFI')
    if args.build_only:
        return
    candidates = [Path(args.qemu).resolve().parent / 'share', Path('/usr/share/qemu')]
    firmware = args.firmware or next((p for p in candidates if (p / 'edk2-aarch64-code.fd').is_file()), candidates[0])
    if not all((firmware / name).is_file() for name in ('edk2-aarch64-code.fd', 'edk2-arm-vars.fd')):
        raise SystemExit('Supply --firmware with the directory containing edk2-aarch64-code.fd and edk2-arm-vars.fd')
    shutil.copy2(firmware / 'edk2-arm-vars.fd', out / 'vars.fd')
    command = [args.qemu, '-machine', 'virt,gic-version=2', '-cpu', 'cortex-a57',
               '-m', '128M', '-smp', '1',
               '-drive', f'if=pflash,format=raw,readonly=on,file={firmware}/edk2-aarch64-code.fd',
               '-drive', f'if=pflash,format=raw,file={out}/vars.fd',
               '-drive', f'if=none,id=esp,format=raw,file=fat:rw:{out}/esp',
               '-device', 'virtio-blk-pci,drive=esp', '-display', 'none',
               '-serial', f'file:{out}/serial.log', '-monitor', 'none', '-net', 'none',
               '-semihosting-config', 'enable=on,target=native', '-no-reboot']
    (out / 'serial.log').unlink(missing_ok=True)
    try:
        result = subprocess.run(command, timeout=args.timeout, capture_output=True)
    except subprocess.TimeoutExpired:
        raise SystemExit(f'ARM64 boot timed out; see {out}/serial.log')
    log = (out / 'serial.log').read_text(errors='replace')
    print(log[max(0, log.find('Cathedral Rust lab: UEFI entry')):])
    if args.fault:
        if result.returncode != 0 or 'CATHEDRAL_RS_EXPECTED_FAULT' not in log or 'CATHEDRAL_RS_PANIC' in log:
            raise SystemExit(f'ARM64 expected fault failed ({result.returncode})')
        print(f'PASS: expected ARM64 {args.fault} exception')
        return
    if result.returncode != 0 or 'CATHEDRAL_RS_BOOT_OK' not in log:
        raise SystemExit(f'ARM64 boot failed ({result.returncode}): {result.stderr.decode(errors="replace")}')
    milestones = ['UEFI entry', 'ExitBootServices complete', 'arch=aarch64',
                  'owned page tables and stack', 'AArch64 vectors installed; breakpoint returned',
                  'heap alignment exhaustion and reclamation passed', 'timer ticks=',
                  'cooperative tasks yielded slept woke and reclaimed',
                  'GPR FP/SIMD FPCR FPSR preserved',
                  'stack allocation rollback passed all frame boundaries',
                  'dynamic spawn limit stale IDs and live-peer progress passed',
                  'task stacks unmapped; heap and physical frames returned to baseline']
    if not args.kernel_only:
        milestones += ['ARM64 IPC round=0', 'ARM64 IPC round=1',
                       'ARM64 timer preempted both private EL0 peers',
                       'ARM64 sealed pages rejected writes', 'ARM64 admission rollback passed',
                       'ARM64 portability proof passed']
    milestones.append('CATHEDRAL_RS_BOOT_OK')
    cursor = 0
    for marker in milestones:
        found = log.find(marker, cursor)
        if found < 0:
            raise SystemExit(f'ARM64 boot missed ordered milestone: {marker}')
        cursor = found + len(marker)
    if 'CATHEDRAL_RS_PANIC' in log or 'CATHEDRAL_RS_FAULT' in log:
        raise SystemExit('ARM64 boot reported a kernel failure')
    print('PASS: ARM64 UEFI boot' if args.kernel_only else 'PASS: ARM64 portability proof')


if __name__ == '__main__':
    main()
