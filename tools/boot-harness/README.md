# Cathedral Omega boot harness

This host-only tool compiles the Omega boot source and boots that invocation's
image under QEMU/OVMF. See the [boot frontier](../../source/kernel/boot/uefi/README.md)
for the required compiler support and remaining kernel work. The Rust lab has its
own harness and is not evidence that the Omega source works.

## F5 in VS Code

Open the Cathedral repository folder, select **Cathedral (Omega): Build & Boot
in QEMU** in Run and Debug, then press **F5**. The existing Rust desktop launch
entries remain available. This entry uses the same fresh-image harness below;
it does not attach an Omega source debugger.

Install Python 3.10+, QEMU and OVMF as described below. The launch uses `python`
on Windows and `python3` on macOS/Linux. By default, it builds the compiler from
the sibling `../Omega` checkout (requiring its Rust toolchain). Alternatively,
set `OMEGA_BIN` to an optimized compiler binary, or `OMEGA_REPO` to the Omega
checkout, in the environment inherited by VS Code. Isolated Cathedral worktrees
need one of these overrides when Omega is not their sibling.

For a new checkout, run `python tools/boot-harness/run.py --prepare --build-only`
first (use `python3` on macOS/Linux), completing any package review requested by
the compiler. F5 does not bypass dependency review. Each launch recompiles the
Omega kernel before opening QEMU; a failed build never boots an old image.

QEMU opens a firmware display window. Kernel progress is currently reported
over serial in the VS Code terminal, from `Hello from Omega` through the
memory and mapping checks described below. The kernel then idles; there is no
Omega desktop yet. Close the QEMU window to end the run. Use `--smoke` from a
terminal for the automated, headless milestone check.

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
retired `omega-cli` name. Cargo runs from the Omega checkout so Rustup selects
its pinned toolchain and Cargo reads its local configuration; Cathedral's host
toolchain is not used to build Omega. An explicit optimized binary is preferable for
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
`Memory paired`, `Page ready`, `Tables ready`, `CPU checked`, `Root located`, `Root read`,
`Tables retained`, `Mapping retired`, and `Mapping checked`.
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
On that same known backing, an External-only supply role must refuse Stable
view formation despite the stronger ordinary-RAM role also being selected.
This refusal is another prerequisite for `Page ready`.
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
The subsequent `Root read` requires an explicitly admitted root extent,
an External-only profile and an actual 512-entry snapshot containing a present
entry. The grant remains retained. This tests root observation, not full
reachable-table retention or activation; the backing and identity-mapping
premises remain explicit external admissions.
`Tables retained` additionally requires completing the reachable four-level
table inventory: every distinct frame has one retained grant, every frame/level
pair is visited once, every new child passes final-map/exclusion checks, and
large leaves end traversal. The 2048-frame/8192-item budgets must not be exhausted.
Every failure retains the incomplete inventory without emitting that marker.
The backing, identity mapping and quiescent-topology premises are explicit
boundary admissions, not facts established by descriptor bytes. This marker
does not mean the replacement hierarchy is active.
`Mapping retired` follows a closed temporary CR3 installation, mapped-region probe
and exact restoration, with an independent root readback by the caller.
`Mapping checked` additionally requires successful mapped writes of distinct
endpoint pairs (73,97), (74,98) and (75,99) across three contiguous pages,
followed by every page's physical resident readback after retirement.
Firmware translations and every
storage descendant remain retained. The live CPU profile is rechecked with
interrupts disabled. This is a synchronous uniprocessor region transaction
within one leaf table, not an escaping mapping API or remote-TLB completion
protocol. Virtual-namespace issuance,
effective cache compatibility and backing remain explicit provider admissions.
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
The scan carries its validated map/stride bounds through every loop state and
uses Exact arithmetic. An arbitrary-input caller in the harness forbids Trap
independently of the assertion helper, matching the kernel's crash restriction.

```text
omega update --project tools/boot-harness/firmware-root
omega run --both tools/boot-harness/firmware-root/main.omg
```

Both engines must exit 0. These synthetic bytes establish numeric policy
behavior, never physical backing or access authority. The firmware smoke test
separately observes the real CR3 and retained final map. Neither test establishes
successful mapping installation.

## Firmware table-entry policy

`table-walk/main.omg` checks the kernel's four-level entry classifier on sixteen
numeric cases, including absent entries, child links, both large-page sizes,
large-page PAT, reserved alignment bits, PT PAT and invalid levels.

```text
omega update --project tools/boot-harness/table-walk
omega run --both tools/boot-harness/table-walk/main.omg
```

Both engines must exit 0. These decisions grant no backing or table authority;
the UEFI boot separately acquires and retains each reached physical frame.

## Path-copy policy and retirement control

`table-clone/main.omg` exercises the actual `core::page_table_clone` operation:
absent paths, 4-KiB/2-MiB/1-GiB paths, large-leaf PAT conversion, read-only
parent ceilings, NX and supervisor/user preservation, unchanged off-path words,
and malformed geometry or supplied-child links. The shared range operation also
checks every output leaf of a three-page request, a complete 512-page table,
the final table suffix and final representable physical frame. Zero, unaligned,
oversized, cross-table and physical-frame-overflow requests refuse without
changing the borrowed snapshot. This is numeric policy, not a
root grant or hardware activation.

```text
omega update --project tools/boot-harness/table-clone
omega run --both tools/boot-harness/table-clone/main.omg
```

Both engines must exit 0. For the source-level retirement control, copy the
authored source tree to an isolated scratch project, change only the final
mapped-probe expectation in `mapping_activation.omg` from
`last == 97 + next` to `last == 97 + next + next / 2`, complete ordinary package
review, then build and boot a fresh image. For the configured three pages,
the first two expectations stay 97 and 98; only the last changes from 99 to
100. The writes and retirement code stay unchanged. The guest must reach
`Mapping retired` but never `Mapping checked`; the normal smoke oracle must
reject that stream for the missing mapped-write/physical-readback marker.
An old image, an early boot failure or a timeout without retirement does not
satisfy this control. Do not change the production source to keep the negative
expectation, or weaken the smoke oracle to accept it as a successful boot.
