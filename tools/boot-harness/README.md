# Cathedral Omega boot harness

This host-only tool compiles the Omega boot source and boots that invocation's
image under QEMU/OVMF. See the [boot frontier](../../source/kernel/boot/uefi/README.md)
for the required compiler repair and remaining kernel-root installation. The Rust lab has its
own harness and is not evidence that the Omega source works.

## Run

Python 3.10+ is required on every host. Bash and PowerShell wrappers call the
same implementation:

```text
python tools/boot-harness/run.py --compiler PATH_TO_OMEGA --prepare --build-only
python tools/boot-harness/run.py --compiler PATH_TO_OMEGA --smoke
```

Alternatively use `run.sh` or `run.ps1` with the same arguments.
`--prepare` runs ordinary `omega update` for dependency review; pending
decisions stop the command. Review them and resume using the compiler's
instructions. Nothing is automatically accepted.

Compiler selection is explicit `--compiler` / `OMEGA_BIN`, then a checkout
at `--omega-repo` / `OMEGA_REPO` (default sibling `../Omega`), then an
installed `omega`. The checkout route uses Cargo package `omega`, not the
retired `omega-cli` name. An explicit optimized binary is preferable for
repeated iterations. In isolated worktrees, supply the checkout or binary
explicitly.

The distribution's `source/distribution/profile.json` selects the entry and
`uefi_x86_64` target. Each compilation uses an empty temporary output
directory. Only a newly emitted `omega-program.exe` is copied to
`build/boot-harness/run-*/esp/EFI/BOOT/BOOTX64.EFI`; failed compilation or success
without an image stops before QEMU. An old ESP image is never a fallback.

## Firmware and execution

Install `qemu-system-x86_64` on PATH. Windows also checks the standard
`C:/Program Files/qemu` installation. Firmware lookup accepts `OVMF` as an
explicit combined image, then checks QEMU's bundled split firmware (including
Homebrew), Linux OVMF/edk2 paths and the local reference checkout. Missing
firmware is an error. Split firmware gets a fresh writable variable store.

Without `--smoke`, QEMU's serial console is interactive. With it, the guest
runs headlessly for at most `--timeout` seconds (default 30), and this run's
serial log must contain the greeting followed by a complete numeric
`Owned N MiB` line. Timeout alone is not success. The check observes those
milestones; it does not prove firmware correctness or long-term liveness.

Logs, firmware copies and the ESP stay in a unique retained `run-*` directory
under `--build-dir` (default `build/boot-harness`). Concurrent invocations never
share images or logs. No network device is attached and reboot is disabled.

Host-only stale-image and milestone controls:

```text
python tools/boot-harness/test_harness.py
```

## Root geometry without firmware

`root-geometry/main.omg` runs the kernel's `BootGeometry::check` on twelve
cases: both range orders, adjacency in both orders, overlap, containment,
identical ranges, either empty root, either overflowing endpoint, and the
largest accepted one-past endpoint. A wrong result crashes; success exits 0.
Eight additional cases exercise `exclude_image_suffix`: valid interior cuts,
either storage ordering, boundaries outside the image or leaving an empty
residual, and initial storage overlapping the retained tail. All twenty cases
must succeed in both native and Terminal execution.

```text
omega update --project tools/boot-harness/root-geometry
omega run --both tools/boot-harness/root-geometry/main.omg
```

Complete ordinary package review if requested. The checkout-local lock is
regenerated, not shared. Use an Omega compiler containing `711be509b7d2` or
later for the payload reread proof repair this helper exercises. The native and
Terminal runs validate numeric geometry only. The helper grants no storage,
performs no firmware calls, and cannot establish stack survival or successful
UEFI handoff. The boot transaction separately invokes this checker before exit.
