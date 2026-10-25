#!/usr/bin/env python3
"""Build and boot the Rust lab; --smoke checks a bounded, real QEMU boot."""

import argparse
import artifacts
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import capture
import disk_image
import database_image
import elf_trim
import compositor_test
import pointer_test
import session_test
import protocol_test
import matrix_desktop_test
import resource_budgets
import timing_report

REPO = Path(__file__).resolve().parents[2]
WORKSPACE = REPO / "source-rs"
BUILD = REPO / "build" / "boot-harness-rs"
# The kernel contract's per-task caps: private pages fund frames; reader pages
# pin other tasks' objects, and a matrix hub needs several objects' worth.
PAGE_LIMITS = (("private_pages", 1024), ("shared_pages", 4096))
# The store's largest object (its MAX_BYTES): an installed program must fit one.
# The installer refuses a larger one; reporting the margin here shows how close
# each build sits before that happens.
OBJECT_BYTES = 1024 * 1024


def report_program(trimmed):
    """Print a trimmed program's size against the object limit; refuse one over it."""
    size = Path(trimmed).stat().st_size
    print(f"program {Path(trimmed).name}: {size} bytes, {100 * size // OBJECT_BYTES}% of the object limit", flush=True)
    if size > OBJECT_BYTES:
        raise SystemExit(f"{trimmed} is {size} bytes; a program must fit one object of {OBJECT_BYTES} bytes")
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
    "Cathedral Rust lab: device capabilities, masked IRQ delivery and driver replacement reclaimed all state",
    "Cathedral Rust lab: shared objects forwarded without mappings; lifetime and reclamation passed",
    "Cathedral Rust lab: ELF instances exited [17, 29]",
    "Cathedral Rust lab: ELF instances exited [18, 30]",
    "Cathedral Rust lab: ELF rejection and",
    "Cathedral Rust lab: IPC echo round=0 256 exchanges; bound rights and stale handles passed",
    "Cathedral Rust lab: IPC echo round=1 256 exchanges; bound rights and stale handles passed",
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
    "Cathedral Rust lab: hierarchical CPU turns preserved outside progress under subtree growth; loan return and full reclamation passed",
    "Cathedral Rust lab: measured CPU ceilings, descendant charging, yield pressure, restart debt and idle refill passed",
    "Cathedral Rust lab: periodic timing under busy/yielding peers and CPU exhaustion reclaimed all state",
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


def program_images(profile_path, composition, release):
    """The profile's programs as (parent object, trimmed ELF) pairs, in order.
    Built program ELFs are trimmed to their loadable bytes beside the harness
    output so the host lab can install them as objects of the first root."""
    installed = composition.get("startup", {}).get("programs", [])
    mode = "release" if release else "debug"
    images = []
    for index, program in enumerate(installed):
        elf_trim.title_record(program.get("display_title"))
        inside = program.get("inside")
        if inside is not None and not (type(inside) is int and 0 <= inside < index):
            raise ValueError("a program may only sit inside an earlier program")
        if any(program.get(key) for key in ("argument", "framebuffer", "keyboard", "disk", "clock", "private_pages", "shared_pages", "ipc_channels", "cpu_budget_us")):
            raise ValueError("store programs carry code only; roles and resources are chosen at launch")
        parent = 0 if inside is None else inside + 1
        if program.get("folder"):
            # An empty object: a container other programs sit inside.
            images.append((parent, None, BUILD / "programs" / Path(profile_path).stem / f"{index}-folder"))
            continue
        built = BUILD / "cargo" / program["target"] / mode / program["package"]
        trimmed = BUILD / "programs" / Path(profile_path).stem / f"{index}-{program['package']}.elf"
        images.append((parent, built, trimmed))
    return images


def seeded_programs(profile_path, release):
    """What a fixture passes to database_image.prepare after a --build-only run."""
    composition = json.loads(Path(profile_path).read_text(encoding="utf-8"))
    return [(parent, trimmed) for parent, _, trimmed in program_images(profile_path, composition, release)]


def realm_source(program):
    """A realm entry's source: a program package, or a directive the installer applies."""
    directives = [key for key in ('folder', 'matrix_config', 'copy') if program.get(key)]
    if 'package' in program:
        if directives:
            raise ValueError('a realm entry is either a program or one directive')
        return None
    if len(directives) != 1:
        raise ValueError('a realm entry without a package needs exactly one directive')
    if directives[0] == 'copy':
        return '@copy=' + program['copy']
    return '@' + directives[0].replace('_', '-')


def realm_programs(profile_path, release):
    """Named entries under each recognized stock user, independent of object IDs.

    Each item pairs the entry's path with its source: a built program file, or
    a directive string for an empty folder, a stock matrix configuration or a
    subtree copy of an earlier entry.
    """
    composition = json.loads(Path(profile_path).read_text(encoding="utf-8"))
    result = []
    seen = set()
    for i, program in enumerate(composition.get('startup', {}).get('realm_programs', [])):
        name = program['path']
        parts = name.split('/')
        if any(not part or part in ('.', '..') or len(part.encode('utf-8')) > 32 for part in parts) or name in seen:
            raise ValueError('realm program paths must be unique relative names')
        if len(parts) > 1 and '/'.join(parts[:-1]) not in seen:
            raise ValueError('a realm program must follow its installed parent')
        if any(program.get(key) for key in ('argument', 'framebuffer', 'keyboard', 'disk', 'clock', 'private_pages', 'shared_pages', 'ipc_channels', 'cpu_budget_us')):
            raise ValueError('realm programs carry code only; the owner grants resources')
        source = realm_source(program)
        if source is None:
            elf_trim.title_record(program.get('display_title'))
            source = BUILD / 'programs' / Path(profile_path).stem / f"realm-{i}-{program['package']}.elf"
        elif source.startswith('@copy='):
            origin = source[len('@copy='):]
            if origin not in seen or name == origin:
                raise ValueError('a realm copy must name an earlier, different entry')
            # The installer enumerates the origin before writing, so a copy may
            # land inside its origin's own subtree. The copy then carries every
            # entry beneath the origin, so later entries may be placed inside it.
            seen.update(name + path[len(origin):] for path in list(seen) if path.startswith(origin + '/'))
        result.append((name, source))
        seen.add(name)
    return result


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
    mode.add_argument("--matrix-desktop-test", action="store_true", help="Check the visible matrix-owned tree, clipping and physical input")
    mode.add_argument("--admission-test", action="store_true", help="Check executable-authorized dynamic process pools")
    mode.add_argument("--ipc-test", action="store_true", help="Check ordinary app channel creation, endpoint moves and worker replies")
    mode.add_argument("--matrix-storage-test", action="store_true", help="Check recursive private-catalog delegation and revocation")
    mode.add_argument("--matrix-test", action="store_true", help="Check recursive matrix locality and input over isolated processes")
    mode.add_argument("--session-test", action="store_true", help="Exercise persistent apps, attenuated keyboard focus and process-owned child scenes")
    mode.add_argument("--pointer-test", action="store_true", help="Exercise physical and scoped synthetic pointers, capture and recovery")
    mode.add_argument("--compositor-test", action="store_true", help="Run the isolated retained-compositor profile and verify scanout")
    mode.add_argument("--build-only", action="store_true")
    mode.add_argument("--smoke", action="store_true")
    parser.add_argument("--fault", choices=("guard", "invalid-opcode", "double-fault", "text-write", "rodata-write", "heap-execute", "stack-execute", "heap-alias-execute", "stack-alias-execute"), help="Expected-fault smoke test (requires --smoke)")
    parser.add_argument("--release", action="store_true")
    parser.add_argument("--startup-probes", action="store_true", help="Include destructive startup probes in the session lab")
    parser.add_argument("--trace-storage", action="store_true", help="Log app/matrix/database/ATA stages; diagnostic timing perturbs execution")
    parser.add_argument("--trace-input", action="store_true", help="Log local matrix input stages (diagnostic timing perturbs execution)")
    parser.add_argument("--collection-pressure", action="store_true", help="Lab only: the storage provider collects before every write inside a tiny workspace")
    parser.add_argument("--pressure", action="store_true", help="Add a pressure peer to the session or matrix desktop")
    parser.add_argument("--kernel-only", action="store_true", help="Build/boot without any platform or distribution executable")
    parser.add_argument("--profile", type=Path, default=WORKSPACE / "distribution/profile.json", help="Host composition profile")
    parser.add_argument("--window", action="store_true", help="Show QEMU's display during an ordinary interactive boot")
    parser.add_argument("--fresh", action="store_true", help="Back up the desktop disk and install a freshly seeded lab disk")
    parser.add_argument("--timeout", type=float, default=120, help="Smoke boot timeout in seconds (default: 120; the full suite takes about 40 s on the reference host)")
    parser.add_argument("--memory", type=int, default=128, help="Guest RAM in MiB (default: 128)")
    parser.add_argument("--accel", choices=("tcg", "whpx"), default="tcg", help="QEMU accelerator: tcg emulates the CPU; whpx runs the guest under the Windows hypervisor, with hardware paging and timing (default: tcg)")
    args = parser.parse_args()
    fixture_profiles = [Path(__file__).parent / f'profiles/{name}.json' for name in ('volume', 'database', 'objects', 'objects-retention', 'matrix-database-recovery', 'notes-recovery')]
    if args.profile.resolve() in [p.resolve() for p in fixture_profiles] and not args.build_only:
        parser.error('storage fixtures require volume_test.py, database_test.py, objects_test.py or matrix_database_test.py; these profiles are build-only here')
    if args.compositor_test or args.pointer_test or args.session_test or args.matrix_test or args.matrix_desktop_test or args.matrix_storage_test or args.admission_test or args.ipc_test:
        if args.kernel_only:
            parser.error("protocol/display tests require userspace")
        args.profile = Path(__file__).parent / ("profiles/matrix-storage.json" if args.matrix_storage_test else "profiles/admission.json" if args.admission_test else "profiles/matrix-desktop.json" if args.matrix_desktop_test else "profiles/channels.json" if args.ipc_test else "profiles/matrix.json" if args.matrix_test else "profiles/session.json" if args.session_test else "profiles/pointer.json" if args.pointer_test else "profiles/compositor.json")
    default_profile = (WORKSPACE / "distribution/profile.json").resolve()
    desktop = args.profile.resolve() == default_profile and not (args.smoke or args.kernel_only)
    if args.fresh and (not desktop or args.build_only):
        parser.error('--fresh requires an ordinary default desktop boot')
    database_desktop = desktop or args.profile.resolve() in [(Path(__file__).parent / f'profiles/{name}.json').resolve() for name in ('matrix-desktop', 'matrix-database-recovery', 'matrix-storage', 'notes', 'notes-recovery')]
    if database_desktop and not args.release:
        parser.error('the database desktop requires --release under TCG')
    if args.timeout <= 0 or args.memory < 64:
        parser.error("timeout must be positive and memory must be at least 64 MiB")
    if args.fault and not args.smoke:
        parser.error("--fault requires --smoke")
    if args.window and (args.smoke or args.compositor_test or args.pointer_test or args.session_test or args.matrix_test or args.matrix_desktop_test or args.matrix_storage_test or args.admission_test or args.ipc_test or args.build_only):
        parser.error("--window requires an ordinary interactive boot")

    if args.kernel_only and args.matrix_storage_test:
        parser.error("the storage fixture requires the distribution")
    custom_profile = args.profile.resolve() != (WORKSPACE / "distribution/profile.json").resolve()
    if custom_profile and args.kernel_only:
        parser.error("--kernel-only does not use a distribution profile")
    if custom_profile and args.smoke:
        parser.error("custom profiles select ordinary startup; smoke uses the standard lab profile")
    composition = ({"target": "x86_64-unknown-uefi", "boot_package": "cathedral-boot-uefi"}
                   if args.kernel_only else json.loads(args.profile.read_text(encoding="utf-8")))
    if desktop and not args.build_only:
        database_image.require_available(BUILD / "cathedral-storage.raw")
    if args.pressure:
        startup = composition.get("startup", {})
        profile = args.profile.resolve()
        if not args.kernel_only and profile == (Path(__file__).parent / "profiles/matrix-desktop.json").resolve():
            # Nine accounts exercise persistence and the capped spinner together.
            startup["dynamic_tasks"] = 4
            startup["executables"].append({
                "package": "cathedral-desktop", "entry": "distribution/desktop/main.rs",
                "target": "x86_64-unknown-none", "argument": 3,
                "features": ["desktop"],
            })
        elif not args.kernel_only and profile == (Path(__file__).parent / "profiles/session.json").resolve():
            startup["launches"].append({
                "package": "cathedral-compositor-lab", "entry": "distribution/lab/compositor/main.rs",
                "target": "x86_64-unknown-none", "clock": True, "argument": 4, "private_pages": 16,
            })
            startup["links"].append([6, 1])
        else:
            parser.error("--pressure requires a session or matrix-desktop profile")
    target = composition["target"]
    boot_package = composition["boot_package"]
    environment = {key: value for key, value in os.environ.items()
                   if not (key.startswith("CATHEDRAL_") and (key.endswith("_ELF") or key in ("CATHEDRAL_INIT_CLOCK", "CATHEDRAL_LINKS", "CATHEDRAL_CHANNEL_LIMITS", "CATHEDRAL_CPU_LIMITS", "CATHEDRAL_DYNAMIC_TASKS") or key.startswith("CATHEDRAL_LAUNCH_") or key.startswith("CATHEDRAL_INIT_") or key.startswith("CATHEDRAL_IMAGE_")))}
    programs = {}
    if not args.kernel_only:
        if args.smoke:
            programs = composition["user_programs"]
        else:
            startup = composition["startup"]
            try:
                limits = resource_budgets.channel_limits(startup)
                cpu_limits = resource_budgets.cpu_limits(startup)
            except ValueError as error:
                parser.error(str(error))
            environment["CATHEDRAL_CHANNEL_LIMITS"] = ",".join(map(str, limits))
            environment["CATHEDRAL_CPU_LIMITS"] = ",".join(map(str, cpu_limits))
            environment["CATHEDRAL_DYNAMIC_TASKS"] = str(startup.get("dynamic_tasks", 0))
            for resource, limit in PAGE_LIMITS:
                pages = startup["initial"].get(resource, 0)
                if type(pages) is not int or not 0 <= pages <= limit:
                    parser.error(f"initial {resource} must be an integer in 0..{limit}")
                environment[f"CATHEDRAL_INIT_{resource.upper()}"] = str(pages)
            programs["init"] = startup["initial"]
            environment["CATHEDRAL_INIT_CLOCK"] = "1" if startup["initial"].get("clock", False) else "0"
            launches = startup.get("launches", [startup["launch"]] if startup.get("launch") else [])
            environment["CATHEDRAL_LINKS"] = ",".join(f"{edge[0]}:{edge[1]}" for edge in startup.get("links", []))
            images = startup.get("executables", [])
            if not isinstance(images, list) or len(images) + len(launches) > 7:
                parser.error("executable catalogue plus static launches must fit seven grants")
            try:
                installed = program_images(args.profile, composition, args.release)
            except ValueError as error:
                parser.error(str(error))
            for index, program in enumerate(startup.get("realm_programs", [])):
                if "package" in program:
                    programs[f"realm_{index}"] = program
            for index, program in enumerate(startup.get("programs", [])):
                if "package" in program:
                    programs[f"program_{index}"] = program
            environment["CATHEDRAL_IMAGE_COUNT"] = str(len(images))
            for index, image in enumerate(images):
                if any(image.get(key) for key in ("framebuffer", "keyboard", "disk", "clock", "private_pages", "shared_pages", "ipc_channels", "cpu_budget_us")):
                    parser.error("catalogue executables carry code only; resources belong to process reservations")
                argument = image.get("argument", 0)
                if type(argument) is not int or not 0 <= argument < 2**64:
                    parser.error("executable argument must be a u64")
                programs[f"image_{index}"] = image
                environment[f"CATHEDRAL_IMAGE_{index}_ARGUMENT"] = str(argument)
            environment["CATHEDRAL_LAUNCH_COUNT"] = str(len(launches))
            for index, child in enumerate(launches):
                programs[f"launch_{index}"] = child
                for resource in ("framebuffer", "keyboard", "disk", "clock"):
                    environment[f"CATHEDRAL_LAUNCH_{index}_{resource.upper()}"] = "1" if child.get(resource, False) else "0"
                for resource, limit in PAGE_LIMITS:
                    pages = child.get(resource, 0)
                    if type(pages) is not int or not 0 <= pages <= limit:
                        parser.error(f"{resource} must be an integer in 0..{limit}")
                    environment[f"CATHEDRAL_LAUNCH_{index}_{resource.upper()}"] = str(pages)
                argument = child.get("argument", 0)
                if type(argument) is not int or not 0 <= argument < 2**64:
                    parser.error("launch argument must be a u64")
                environment[f"CATHEDRAL_LAUNCH_{index}_ARGUMENT"] = str(argument)
    profile = "release" if args.release else "debug"
    captured = {}
    for name, user in programs.items():
        user_cargo = ["cargo", "build", "--locked", "--package", user['package'],
                      "--target", user['target'], "--target-dir", str(BUILD / "cargo")]
        user_features = list(user.get("features", []))
        if args.startup_probes and user['package'] == 'cathedral-compositor-lab':
            user_features.append('session-probes')
        if args.matrix_desktop_test and user["package"] == "cathedral-desktop":
            user_features.append("desktop-probes")
        if args.matrix_desktop_test and user["package"] == "cathedral-storage-service":
            user_features.append("recovery-lab")
        if args.trace_input and user["package"] in ("cathedral-stock-matrix", "cathedral-input-service", "cathedral-display-service", "cathedral-desktop", "cathedral-colors", "cathedral-notes", "cathedral-chooser", "cathedral-omega-ui-demo"):
            user_features.append("trace-input")
        if args.trace_storage and user["package"] in ("cathedral-stock-matrix", "cathedral-desktop", "cathedral-storage-service", "cathedral-notes", "cathedral-colors"):
            user_features.append("trace-storage")
        if args.collection_pressure and user["package"] == "cathedral-storage-service":
            user_features.append("collection-pressure")
        if user_features:
            user_cargo.extend(["--features", ",".join(user_features)])
        if args.release:
            user_cargo.append("--release")
        subprocess.run(user_cargo, cwd=WORKSPACE, check=True)
        user_elf = BUILD / "cargo" / user['target'] / profile / user['package']
        captured[name] = artifacts.capture(user_elf, BUILD / 'artifacts')
        environment[f'CATHEDRAL_{name.upper()}_ELF'] = str(captured[name])

    seeded = []
    if not args.kernel_only and not args.smoke:
        for index, (parent, built, trimmed) in enumerate(installed):
            if built is None:
                trimmed.parent.mkdir(parents=True, exist_ok=True)
                trimmed.write_bytes(b"")
            else:
                elf_trim.trim(captured[f'program_{index}'], trimmed, startup["programs"][index].get("display_title"))
                report_program(trimmed)
            seeded.append((parent, trimmed))
    if not args.kernel_only and not args.smoke:
        for index, (_, trimmed) in enumerate(realm_programs(args.profile, args.release)):
            if not isinstance(trimmed, str):
                elf_trim.trim(captured[f'realm_{index}'], trimmed, startup['realm_programs'][index].get('display_title'))
                report_program(trimmed)
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
    output = BUILD / (f"fault-{args.fault}" if args.fault else "smoke" if args.smoke else "test" if (args.compositor_test or args.pointer_test or args.session_test or args.matrix_test or args.matrix_desktop_test or args.matrix_storage_test or args.admission_test or args.ipc_test) else "interactive")
    if args.kernel_only:
        output = output.with_name("kernel-" + output.name)
    if custom_profile:
        output = output.with_name(args.profile.stem + "-" + output.name)
    if args.pressure:
        output = output.with_name("pressure-" + output.name)
    esp = output / "esp/EFI/BOOT"
    esp.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(BUILD / "cargo" / target / profile / f"{boot_package}.efi", esp / "BOOTX64.EFI")
    print(f"EFI image: {esp / 'BOOTX64.EFI'}", flush=True)
    if args.build_only:
        return 0

    qemu = find_qemu()
    command = [qemu, "-machine", "q35", "-cpu", "qemu64", "-accel", args.accel, "-smp", "1", "-m", str(args.memory)]
    command += firmware_args(qemu, output)
    command += ["-drive", "format=raw,file=fat:rw:esp", "-nic", "none",
                "-monitor", "none", "-no-reboot"]
    if not args.kernel_only and not args.smoke and any(child.get("disk", False) for child in launches):
        disposable = args.matrix_storage_test or args.matrix_desktop_test
        # Matrix app schemas must never reinterpret the default distribution's roots.
        persistent = BUILD / ("cathedral-storage.raw" if desktop else "notes-storage.raw" if args.profile.stem == "notes" else "matrix-database-storage.raw")
        data_path = output / "test-storage.raw" if disposable else persistent
        # A disposable proof starts from a freshly formatted object volume;
        # persistent media is created once and never reformatted.
        if disposable and data_path.exists():
            data_path.unlink()
        migration = composition.get("startup", {}).get("label_migration")
        if args.fresh:
            database_image.reset(data_path)
        database_image.prepare(data_path, seeded, WORKSPACE / migration if migration else None,
                               composition.get("startup", {}).get("app_close_migration", False),
                               realm_programs(args.profile, args.release))
        command += disk_image.arguments(data_path)
    if not args.window:
        command += ["-display", "none"]
    # No console helper on Windows; --window explicitly opts into QEMU's GUI.
    creationflags = subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0
    if args.matrix_desktop_test:
        capture.run(command, output, args.timeout, creationflags, ready_marker="Cathedral matrix desktop: ready", verify=lambda stream, output, serial: matrix_desktop_test.verify(stream, output, serial, pressure=args.pressure))
        shutil.copyfile(output / "serial.log", output / "first-boot-serial.log")
        capture.run(command, output, args.timeout, creationflags,
                    ready_marker="Cathedral matrix desktop: ready", verify=matrix_desktop_test.verify_reboot)
        return 0
    if args.admission_test:
        protocol_test.run(command, output, args.timeout, creationflags, "Cathedral admission: executable capabilities, recursive funded pools, load rollback, stale grants and sibling survival passed")
        return 0
    if args.matrix_test or args.matrix_storage_test or args.ipc_test:
        marker = ("Cathedral matrix storage: scoped roots, recursive delegation, revocation, cheap copies, attenuated realms, programs from the realm, launched siblings and replacement passed" if args.matrix_storage_test else "Cathedral IPC: channels, startup bundles, recursive admission and reclamation passed" if args.ipc_test
                  else "Cathedral matrix: two identical worlds, local-only views, attenuated input and recovery passed")
        protocol_test.run(command, output, args.timeout, creationflags, marker)
        return 0
    if args.session_test:
        capture.run(command, output, args.timeout, creationflags, ready_marker="Cathedral session: ready",
                    verify=lambda stream, output, serial: session_test.verify(stream, output, serial, pressure=args.pressure))
        return 0
    if args.pointer_test:
        capture.run(command, output, args.timeout, creationflags, ready_marker="Cathedral pointer: ready", verify=pointer_test.verify)
        return 0
    if args.compositor_test:
        capture.run(command, output, args.timeout, creationflags, ready_marker="Cathedral compositor: ready", verify=compositor_test.verify)
        return 0
    if not args.smoke:
        command += ["-chardev", "stdio,id=serial0,logfile=serial.log,logappend=off",
                    "-serial", "chardev:serial0"]
        print(f"Booting; serial log: {output / 'serial.log'}; Ctrl+C stops QEMU.", flush=True)
        process = subprocess.Popen(command, cwd=output, creationflags=creationflags)
        try:
            return process.wait()
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=5)

    serial_path = output / "serial.log"
    serial_path.write_text("", encoding="utf-8")
    command += ["-serial", "file:serial.log", "-serial", "null", "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]
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
    if not args.kernel_only and not args.fault:
        try:
            cases = timing_report.verify_periodic_waits(serial)
        except ValueError as error:
            raise RuntimeError(f"Wait/selection observations failed: {error}; logs: {output}") from error
        if cases != 5:
            raise RuntimeError(f"Expected five periodic wait observations, got {cases}; logs: {output}")
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
