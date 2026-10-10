#!/usr/bin/env python3
"""Build and boot the Rust lab; --smoke checks a bounded, real QEMU boot."""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import capture
import disk_image
import storage_test

REPO = Path(__file__).resolve().parents[2]
WORKSPACE = REPO / "source-rs"
BUILD = REPO / "build" / "boot-harness-rs"
MARKERS = (
    "Cathedral Rust lab: UEFI entry",
    "Cathedral Rust lab: ExitBootServices complete",
    "Cathedral Rust lab: memory regions=",
    "Cathedral Rust lab: arch=",
    "Cathedral Rust lab: owned page tables and stack",
    "Cathedral Rust lab: GDT TSS IDT installed; breakpoint returned",
    "Cathedral Rust lab: heap initialized",
    "Cathedral Rust lab: heap alignment exhaustion and reclamation passed",
    "Cathedral Rust lab: timer ticks=",
    "Cathedral Rust lab: cooperative tasks yielded slept woke and reclaimed",
    "Cathedral Rust lab: preempted non-yielding tasks counts=",
    "Cathedral Rust lab: task heap reclaimed and stack slots reusable",
    "Cathedral Rust lab: stack allocation rollback passed all frame boundaries",
    "Cathedral Rust lab: dynamic spawn limit stale IDs and live-peer progress passed",
    "Cathedral Rust lab: task stacks unmapped; heap and physical frames returned to baseline",
    *(f"Cathedral Rust lab: user fault contained mode={mode} " for mode in range(1, 11)),
    "Cathedral Rust lab: ring3 private memory preemption and checked syscalls passed",
    "Cathedral Rust lab: user admission rollback passed",
    "Cathedral Rust lab: ELF instances exited [17, 29]",
    "Cathedral Rust lab: ELF instances exited [18, 30]",
    "Cathedral Rust lab: ELF rejection and",
    "Cathedral Rust lab: IPC echo round=0 32 exchanges; bound rights and stale handles passed",
    "Cathedral Rust lab: IPC echo round=1 32 exchanges; bound rights and stale handles passed",
    "Cathedral Rust lab: IPC peer exit woke blocked receiver and reclaimed all memory",
    "Cathedral Rust lab: IPC peer fault woke blocked receiver and reclaimed all memory",
    "Cathedral Rust lab: IPC revocation woke blocked receiver and reclaimed all memory",
    "Cathedral Rust lab: IPC backpressure and checked copyout preserved queued message; all memory reclaimed",
    "Cathedral Rust lab: IPC deadlines woke idle sessions; checked copies, late replies, terminal precedence and denied clock access passed",
    "Cathedral Rust lab: userspace supervisor restarted 32 faulted services; live client reconnected with fresh grants",
    "Cathedral Rust lab: supervision parent-exit cancellation and collected return status reclaimed all memory",
    "Cathedral Rust lab: supervision failed-spawn retries preserved live peers and memory baselines",
    "Cathedral Rust lab: multiple launch grants preserved sibling IPC across 16 restarts; failed admission and keyboard-wait cancellation reclaimed all memory",
    "Cathedral Rust lab: peer graph survived 16 app/provider replacements; all endpoints, frames and heap reclaimed",
    "Cathedral Rust lab: deadlines recovered 4 silent and 4 spinning services; independent observer progressed; all memory reclaimed",
    "Cathedral Rust lab: clock grants copy checks and deadline completion/cancellation precedence passed",
    "Cathedral Rust lab: deadline woke an idle session with every user task blocked; all memory reclaimed",
    "Cathedral Rust lab: runtime page allocation rolled back at every frame boundary",
    "Cathedral Rust lab: private budgets zeroing shared leases NX write faults stale handles and peer death passed; all memory reclaimed",
    "Cathedral Rust lab: rendering equivalence same-page escape page fault mapping counts and reclamation passed",
    "Cathedral Rust lab: GOP ",
    "Cathedral Rust lab: display service faulted and restarted; pattern redrawn; observer progressed; ungranted mapping fault contained; all task memory reclaimed",
    "Cathedral Rust lab: framebuffer NX guards and checked copies passed; ",
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
    # Binary user diagnostics may decode to replacement characters. Windows
    # terminals with a legacy encoding must still report and validate the boot.
    sys.stdout.reconfigure(errors="backslashreplace")
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--build-only", action="store_true")
    mode.add_argument("--smoke", action="store_true")
    mode.add_argument("--screenshot", action="store_true", help="Boot normally, verify scanout through QMP, save display.png and stop")
    mode.add_argument("--input-test", action="store_true", help="Verify keyboard navigation and independent provider restarts through QMP")
    mode.add_argument("--recovery-test", action="store_true", help="Inject provider failures and verify automatic deadline recovery")
    mode.add_argument("--storage-test", action="store_true", help="Verify durable state across reboot and abrupt QEMU power cuts")
    parser.add_argument("--fault", choices=("guard", "invalid-opcode", "double-fault"), help="Expected-fault smoke test (requires --smoke)")
    parser.add_argument("--release", action="store_true")
    parser.add_argument("--kernel-only", action="store_true", help="Build/boot without any platform or distribution executable")
    parser.add_argument("--profile", type=Path, default=WORKSPACE / "distribution/profile.json", help="Host composition profile")
    parser.add_argument("--window", action="store_true", help="Show QEMU's display during an ordinary interactive boot")
    parser.add_argument("--timeout", type=float, default=30, help="Smoke boot timeout in seconds (default: 30)")
    parser.add_argument("--memory", type=int, default=128, help="Guest RAM in MiB (default: 128)")
    args = parser.parse_args()
    if args.timeout <= 0 or args.memory < 64:
        parser.error("timeout must be positive and memory must be at least 64 MiB")
    if args.fault and not args.smoke:
        parser.error("--fault requires --smoke")
    if args.window and (args.smoke or args.screenshot or args.input_test or args.recovery_test or args.storage_test or args.build_only):
        parser.error("--window requires an ordinary interactive boot")

    if args.kernel_only and (args.screenshot or args.input_test or args.recovery_test or args.storage_test):
        parser.error("capture modes require the distribution scene")
    custom_profile = args.profile.resolve() != (WORKSPACE / "distribution/profile.json").resolve()
    if custom_profile and args.kernel_only:
        parser.error("--kernel-only does not use a distribution profile")
    if custom_profile and (args.recovery_test or args.storage_test):
        parser.error("recovery/storage tests use the standard distribution profile")
    if custom_profile and args.smoke:
        parser.error("custom profiles select ordinary startup; smoke uses the standard lab profile")
    composition = ({"target": "x86_64-unknown-uefi", "boot_package": "cathedral-boot-uefi"}
                   if args.kernel_only else json.loads(args.profile.read_text(encoding="utf-8")))
    target = composition["target"]
    boot_package = composition["boot_package"]
    environment = {key: value for key, value in os.environ.items()
                   if not (key.startswith("CATHEDRAL_") and (key.endswith("_ELF") or key in ("CATHEDRAL_INIT_CLOCK", "CATHEDRAL_LINKS") or key.startswith("CATHEDRAL_LAUNCH_")))}
    programs = {}
    if not args.kernel_only:
        if args.smoke:
            programs = composition["user_programs"]
        else:
            startup = composition["startup"]
            programs["init"] = startup["initial"]
            environment["CATHEDRAL_INIT_CLOCK"] = "1" if startup["initial"].get("clock", False) else "0"
            launches = startup.get("launches", [startup["launch"]] if startup.get("launch") else [])
            environment["CATHEDRAL_LINKS"] = ",".join(f"{edge[0]}:{edge[1]}" for edge in startup.get("links", []))
            if args.recovery_test or args.storage_test:
                startup["initial"]["features"] = ["recovery-lab"]
                for index, child in enumerate(launches):
                    child["features"] = ["recovery-lab"]
                    child["argument"] = 2 if args.recovery_test and index < 2 else 0 # Provider startup wedges; application stays live.
            environment["CATHEDRAL_LAUNCH_COUNT"] = str(len(launches))
            for index, child in enumerate(launches):
                programs[f"launch_{index}"] = child
                for resource in ("framebuffer", "keyboard", "disk", "clock"):
                    environment[f"CATHEDRAL_LAUNCH_{index}_{resource.upper()}"] = "1" if child.get(resource, False) else "0"
                for resource in ("private_pages", "shared_pages"):
                    pages = child.get(resource, 0)
                    if type(pages) is not int or not 0 <= pages <= 4:
                        parser.error("memory page budgets must be integers between 0 and 4")
                    environment[f"CATHEDRAL_LAUNCH_{index}_{resource.upper()}"] = str(pages)
                argument = child.get("argument", 0)
                if type(argument) is not int or not 0 <= argument < 2**64:
                    parser.error("launch argument must be a u64")
                environment[f"CATHEDRAL_LAUNCH_{index}_ARGUMENT"] = str(argument)
    profile = "release" if args.release else "debug"
    for name, user in programs.items():
        user_cargo = ["cargo", "build", "--locked", "--package", user['package'],
                      "--target", user['target'], "--target-dir", str(BUILD / "cargo")]
        if user.get("features"):
            user_cargo.extend(["--features", ",".join(user["features"])])
        if args.release:
            user_cargo.append("--release")
        subprocess.run(user_cargo, cwd=WORKSPACE, check=True)
        user_elf = BUILD / "cargo" / user['target'] / profile / user['package']
        environment[f'CATHEDRAL_{name.upper()}_ELF'] = str(user_elf.resolve(strict=True))

    cargo = ["cargo", "build", "--locked", "--package", boot_package,
             "--target", target, "--target-dir", str(BUILD / "cargo")]
    if args.release:
        cargo.append("--release")
    features = [] if args.kernel_only else ['bundled-user']
    if args.smoke:
        features.append(f"fault-{args.fault}" if args.fault else "smoke-test")
    if features:
        cargo.extend(["--features", ','.join(features)])
    subprocess.run(cargo, cwd=WORKSPACE, check=True, env=environment)
    # Keep smoke images and their terminating feature separate from normal boots.
    output = BUILD / (f"fault-{args.fault}" if args.fault else "smoke" if args.smoke else "storage-test" if args.storage_test else "recovery-test" if args.recovery_test else "input-test" if args.input_test else "capture" if args.screenshot else "interactive")
    if args.kernel_only:
        output = output.with_name("kernel-" + output.name)
    if custom_profile:
        output = output.with_name(args.profile.stem + "-" + output.name)
    esp = output / "esp/EFI/BOOT"
    esp.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(BUILD / "cargo" / target / profile / f"{boot_package}.efi", esp / "BOOTX64.EFI")
    print(f"EFI image: {esp / 'BOOTX64.EFI'}", flush=True)
    if args.build_only:
        return 0

    qemu = find_qemu()
    command = [qemu, "-machine", "q35", "-cpu", "qemu64", "-accel", "tcg", "-smp", "1", "-m", str(args.memory)]
    command += firmware_args(qemu, output)
    command += ["-drive", "format=raw,file=fat:rw:esp", "-nic", "none",
                "-monitor", "none", "-no-reboot"]
    if not args.kernel_only and not args.smoke and any(child.get("disk", False) for child in launches):
        data_path = output / "test-storage.raw" if (args.screenshot or args.input_test or args.recovery_test or args.storage_test) else BUILD / "storage.raw"
        disk_image.prepare(data_path, reset=args.screenshot or args.input_test or args.recovery_test or args.storage_test)
        command += disk_image.arguments(data_path)
    if not args.window:
        command += ["-display", "none"]
    # No console helper on Windows; --window explicitly opts into QEMU's GUI.
    creationflags = subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0
    if args.storage_test:
        storage_test.run(command, output, args.timeout, creationflags)
        return 0
    if args.screenshot or args.input_test or args.recovery_test:
        capture.run(command, output, args.timeout, creationflags, args.input_test, args.recovery_test)
        return 0
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
    markers = MARKERS
    if args.kernel_only:
        markers = MARKERS[:MARKERS.index("Cathedral Rust lab: ELF instances exited [17, 29]")] + ("CATHEDRAL_RS_BOOT_OK",)
    if args.fault:
        markers = MARKERS[:MARKERS.index("Cathedral Rust lab: timer ticks=")] + ("CATHEDRAL_RS_FAULT:", "CATHEDRAL_RS_EXPECTED_FAULT")
    for marker in markers:
        found = serial.find(marker, position)
        if found < 0:
            raise RuntimeError(f"Missing or out-of-order boot marker: {marker}; logs: {output}")
        position = found + len(marker)
    # isa-debug-exit returns (guest_value << 1) | 1, so guest 0x10 means 33.
    if result.returncode != 33 or "CATHEDRAL_RS_PANIC" in serial:
        raise RuntimeError(f"Boot failed (QEMU exit {result.returncode}); logs: {output}")
    print("PASS: standalone kernel boot and raw user isolation" if args.kernel_only and not args.fault else f"PASS: expected {args.fault} exception" if args.fault else "PASS: boot, preemption, fault containment, static ELF programs, capability IPC, userspace supervision, deadline/display recovery and memory reclamation")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except KeyboardInterrupt:
        sys.exit(130)
    except (OSError, RuntimeError, subprocess.SubprocessError) as error:
        print(f"error: {error}", file=sys.stderr)
        sys.exit(1)
