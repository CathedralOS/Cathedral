#!/usr/bin/env python3
"""Build and boot the Rust lab; --smoke checks a bounded, real QEMU boot."""

import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys

REPO = Path(__file__).resolve().parents[2]
WORKSPACE = REPO / "source-rs"
BUILD = REPO / "build" / "boot-harness-rs"
TARGET = "x86_64-unknown-uefi"
MARKERS = (
    "Cathedral Rust lab: UEFI entry",
    "Cathedral Rust lab: ExitBootServices complete",
    "Cathedral Rust lab: memory regions=",
    "Cathedral Rust lab: arch=",
    "Cathedral Rust lab: owned page tables and stack",
    "Cathedral Rust lab: GDT TSS IDT installed; breakpoint returned",
    "Cathedral Rust lab: timer ticks=",
    "CATHEDRAL_RS_BOOT_OK",
)


def find_qemu():
    candidate = os.environ.get("QEMU") or shutil.which("qemu-system-x86_64")
    if candidate:
        return str(Path(candidate).resolve())
    candidate = Path("C:/Program Files/qemu/qemu-system-x86_64.exe")
    if candidate.is_file():
        return str(candidate)
    raise RuntimeError("QEMU not found. Install qemu-system-x86_64 or set QEMU to its executable path.")


def firmware_args(qemu, output):
    if os.environ.get("OVMF"):
        source = Path(os.environ["OVMF"]).resolve(strict=True)
        shutil.copyfile(source, output / "ovmf.fd")
        return ["-bios", "ovmf.fd"]

    code = os.environ.get("OVMF_CODE")
    variables = os.environ.get("OVMF_VARS")
    if bool(code) != bool(variables):
        raise RuntimeError("Set both OVMF_CODE and OVMF_VARS to a matching firmware pair.")
    pairs = [(Path(code), Path(variables))] if code else []
    executable_dir = Path(qemu).parent
    for folder in (executable_dir / "share", executable_dir / "share/qemu",
                   executable_dir.parent / "share/qemu", Path("/usr/share/qemu")):
        pairs.append((folder / "edk2-x86_64-code.fd", folder / "edk2-i386-vars.fd"))
    for folder in (Path("/usr/share/OVMF"), Path("/usr/share/edk2/ovmf")):
        for suffix in ("_4M", ""):
            pairs.append((folder / f"OVMF_CODE{suffix}.fd", folder / f"OVMF_VARS{suffix}.fd"))
    for code_path, vars_path in pairs:
        if code_path.is_file() and vars_path.is_file():
            shutil.copyfile(code_path, output / "ovmf-code.fd")
            # Reset a private variable store on every run; never mutate installed firmware.
            shutil.copyfile(vars_path, output / "ovmf-vars.fd")
            return ["-drive", "if=pflash,format=raw,readonly=on,file=ovmf-code.fd",
                    "-drive", "if=pflash,format=raw,file=ovmf-vars.fd"]
    raise RuntimeError("OVMF not found. Set OVMF to a combined image, or OVMF_CODE and OVMF_VARS.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--build-only", action="store_true")
    mode.add_argument("--smoke", action="store_true")
    parser.add_argument("--fault", choices=("guard", "invalid-opcode", "double-fault"), help="Expected-fault smoke test (requires --smoke)")
    parser.add_argument("--release", action="store_true")
    parser.add_argument("--timeout", type=float, default=30, help="Smoke boot timeout in seconds (default: 30)")
    parser.add_argument("--memory", type=int, default=128, help="Guest RAM in MiB (default: 128)")
    args = parser.parse_args()
    if args.timeout <= 0 or args.memory < 64:
        parser.error("timeout must be positive and memory must be at least 64 MiB")
    if args.fault and not args.smoke:
        parser.error("--fault requires --smoke")

    cargo = ["cargo", "build", "--locked", "--package", "cathedral-boot-uefi",
             "--target", TARGET, "--target-dir", str(BUILD / "cargo")]
    if args.release:
        cargo.append("--release")
    if args.smoke:
        cargo.extend(["--features", f"fault-{args.fault}" if args.fault else "smoke-test"])
    subprocess.run(cargo, cwd=WORKSPACE, check=True)
    # Keep smoke images and their terminating feature separate from normal boots.
    output = BUILD / (f"fault-{args.fault}" if args.fault else "smoke" if args.smoke else "interactive")
    esp = output / "esp/EFI/BOOT"
    esp.mkdir(parents=True, exist_ok=True)
    profile = "release" if args.release else "debug"
    shutil.copyfile(BUILD / "cargo" / TARGET / profile / "cathedral-boot-uefi.efi", esp / "BOOTX64.EFI")
    print(f"EFI image: {esp / 'BOOTX64.EFI'}", flush=True)
    if args.build_only:
        return 0

    qemu = find_qemu()
    command = [qemu, "-machine", "q35", "-cpu", "qemu64", "-accel", "tcg", "-smp", "1", "-m", str(args.memory)]
    command += firmware_args(qemu, output)
    command += ["-drive", "format=raw,file=fat:rw:esp", "-nic", "none",
                "-display", "none", "-monitor", "none", "-no-reboot"]
    # No visible helper window on Windows; interactive output stays in this terminal.
    creationflags = subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0
    if not args.smoke:
        command += ["-serial", "stdio"]
        print("Booting; Ctrl+C stops QEMU.", flush=True)
        process = subprocess.Popen(command, cwd=output, creationflags=creationflags)
        try:
            return process.wait()
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=5)

    serial_path = output / "serial.log"
    serial_path.write_text("", encoding="utf-8")
    command += ["-serial", "file:serial.log", "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]
    with (output / "qemu.log").open("w", encoding="utf-8") as log:
        try:
            result = subprocess.run(command, cwd=output, stdout=log, stderr=log,
                                    timeout=args.timeout, creationflags=creationflags)
        except subprocess.TimeoutExpired as error:
            print(serial_path.read_text(encoding="utf-8", errors="replace"))
            raise RuntimeError(f"Boot timed out after {args.timeout}s; logs: {output}") from error
    serial = serial_path.read_text(encoding="utf-8", errors="replace")
    print(serial)
    position = 0
    markers = MARKERS if not args.fault else MARKERS[:-2] + ("CATHEDRAL_RS_FAULT:", "CATHEDRAL_RS_EXPECTED_FAULT")
    for marker in markers:
        found = serial.find(marker, position)
        if found < 0:
            raise RuntimeError(f"Missing or out-of-order boot marker: {marker}; logs: {output}")
        position = found + len(marker)
    # isa-debug-exit returns (guest_value << 1) | 1, so guest 0x10 means 33.
    if result.returncode != 33 or "CATHEDRAL_RS_PANIC" in serial:
        raise RuntimeError(f"Boot failed (QEMU exit {result.returncode}); logs: {output}")
    print(f"PASS: expected {args.fault} exception" if args.fault else "PASS: owned memory, exception entry/return, and timer IRQs")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except KeyboardInterrupt:
        sys.exit(130)
    except (OSError, RuntimeError, subprocess.SubprocessError) as error:
        print(f"error: {error}", file=sys.stderr)
        sys.exit(1)
