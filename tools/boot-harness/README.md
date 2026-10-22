# Cathedral Omega boot harness

This host-only tool compiles the Omega boot source and boots that invocation's
image under QEMU/OVMF. See the [boot frontier](../../source/kernel/boot/uefi/README.md)
for the required compiler support and remaining kernel work. The Rust lab has its
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
`Owned N MiB` line, then `Boot map ready`, `Kernel entered`, `Memory split`, `Memory checked`,
`Memory paired`, `Page ready`, `Tables ready`, `CPU checked`, and `Root located`.
The boot-map marker requires the kernel to copy the final memory map from the
explicit initial-storage packet and check its format, byte count, descriptor
stride, checksum, original storage base and image-prefix correspondence. It
does not authenticate firmware or grant table-root authority. The partition marker
requires the kernel to establish vacancy for its initial-storage root, split
4096 bytes with the checked core operation, and observe both returned lengths.
Both children remain owned. `Memory checked` additionally requires successful
resident initialization with 17, readback of 17, mutation and readback of 29,
extraction of 29, and return of the original extent base and length.
`Memory paired` requires two owned eight-byte allocations from the checked
`alloc::bump` package through brief strategy borrows, rejection of an oversized
request with unchanged tail base, length and capacity, two simultaneous residents
initialized with 17 and 41, independent
writes of 29 and 73, rereading the first after the second write, and extraction
of both updated values. `Page ready` additionally requires an aligned 4096-byte
page prepared after an eight-byte reservation, indexed resident reads of 17 and
73, helper writes of 29 and 97 to the first and last words, rereads, a borrowed
External snapshot of all 512 words, and independent whole-page extraction of
those updated values,
zero readback of all 512 words, and retention of the initialized page, padding
and remaining tail. Changing the expected extracted last word from 97 to 98
suppresses the page marker while the preceding markers still appear. Requiring
98 instead of 97 in `PageFrame::check_snapshot` also suppresses that marker:
the snapshot's copied bytes participate in acceptance. The snapshot borrows
the original grant without moving resident custody or claiming Stable access.
This known-page control does not yet observe the active firmware root.
`Tables ready` additionally requires four aligned, resident 4096-byte table
pages carved from the retained tail. The kernel writes the three links and one
RW/supervisor/NX leaf to the retained target page, reads the actual packed words
back, validates their complete fields and walk endpoints, and reads every unused
slot as zero. Padding, target, tables and remaining tail stay owned. This is
inactive hierarchy construction, not a CR3 switch or a successful virtual access.
`CPU checked` requires the supported four-level paging regime. `Root located`
requires the actual CR3 frame to pass the full final-map eligibility scan and
complete image/private-stack and original allocator-root exclusions. It grants
no table access and does not retain the reachable firmware-table closure.
All storage descendants remain retained. A split-only, single-resident, or
paired-allocation-only or page-only boot does not satisfy this acceptance. Missing, truncated or
out-of-order markers fail.
Timeout alone is not success. The check observes those
milestones; it does not prove firmware correctness or long-term liveness.
The serial oracle observes the kernel's checks; it does not independently grant
Stable access, prove compiler correctness, or demonstrate a general allocator.
The exhaustion result remains owned through the retained kernel state. Replacing
only the oversized request with zero bytes creates a success result with unchanged
geometry; the final marker must disappear. This is a source-level boot control,
not an alternative harness success condition.

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

## Firmware-root map policy

`firmware-root/main.omg` runs the same `FirmwareRootMap` checker used after
the CPU observation. It covers a valid root in an unsorted runtime-stride map,
CR3 flags/reserved bits, malformed map and descriptor sizes, missing/duplicate
coverage, image/storage exclusions, disallowed ownership and attributes, and
page-count/end overflow. An incorrect decision crashes; success exits 0.

```text
omega update --project tools/boot-harness/firmware-root
omega run --both tools/boot-harness/firmware-root/main.omg
```

Both engines must exit 0. These synthetic bytes establish numeric policy
behavior, never physical backing or access authority. The firmware smoke test
separately observes the real CR3 and retained final map. Neither test establishes
successful mapping installation.
