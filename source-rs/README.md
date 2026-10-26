# Cathedral Rust lab

A bootable Rust implementation for working out Cathedral's OS mechanisms while
Omega develops. `source/` remains the Omega implementation; this workspace
mirrors its ownership layers and names as behavior is implemented. It is an
experimental kernel, with explicit unsafe hardware operations and no claim to
Omega's proof or authority guarantees.

See the [implementation map](STATUS.md) for the integrated matrix desktop,
mechanism-to-test coverage, restrictions and next kernel boundaries.
The harness reports each compiled user ELF's load-page demand before bundling.
[The check](../tools/boot-harness-rs/elf_limits.py) includes zero-filled memory and
reads the kernel's current image-page cap; it does not replace kernel admission
validation. An oversized instrumented provider fails the build with its file name
and page demand instead of reaching QEMU as a generic BAD_EXECUTABLE refusal.

The [ARM64 portability proof](kernel/arch/isa/aarch64/README.md) exercises the shared
core on a second CPU architecture; the integrated desktop remains x86-64.

The desktop builds its UI from checked-in
[`source/distribution/libraries/omega-ui`](../source/distribution/libraries/omega-ui/README.md).
Build the supported Omega compiler in `build/omega-ui/compiler`, set `OMEGA_BIN`
to it elsewhere, or put it on PATH. The
[UI build guide](../tools/omega-ui/README.md#build) includes the producer and build commands.
The installed [Colors](../source/distribution/applications/colors/README.md),
[Notes](../source/distribution/applications/notes/README.md) and
[chooser](../source/distribution/applications/chooser/README.md) apps own their
sessions, input, storage behavior and presentation in Omega. A reusable
[platform host](platform/libraries/omega-app/README.md) supplies its existing
scoped service transport. The kernel and remaining desktop applications are
still Rust; this is an incremental application migration.

Press **F5** at the repository root, or run
`python tools/boot-harness-rs/run.py --release --window --fresh`. One desktop boots to a
chooser over the folder of users: NEW copies the template user, clicking a user
runs it, and its bottom taskbar launches Notes, Colors, Split, Nested or Omega UI instances;
use the running-app buttons to switch, click a program to focus it
and its upper-right X to close it with draft protection. Notes uses F1 to save/check and
F2 to reload; Colors uses Enter.
See [controls and executable routes](distribution/README.md).
Normal F5 starts a fresh prototype each time. It sets the old disk and its
receipt aside under `build/boot-harness-rs/before-fresh-*`, then seeds a new disk
with current programs. Create users again. Choose **Cathedral: Resume Saved Lab
in QEMU**, or omit `--fresh`, to keep users, notes and colors across launches.
Legacy lab users/templates are not a compatibility target.
`python tools/boot-harness-rs/desktop_test.py` tests this same default composition,
including three independent Notes drafts, capacity refusal, switching and close.
It records QMP-to-observed-text latency upper bounds in `input-latency.json`.
Add `--trace-storage --trace-input` for provider I/O, storage phase and compositor
CPU samples alongside host observations of boot, login, launches, typing,
switching and saves. See [desktop timing interpretation](../tools/boot-harness-rs/README.md#ordinary-desktop-timings)
before comparing these measurements. `--soak CYCLES --nested-depth D` keeps one
session busy instead: it enters the nested desktops, edits and saves at the
deepest level and tears the subtree down that many times, checking exact saved
text, provider heap headroom and the absence of provider replacements.
Add `--latency` to measure picker/user entry, Notes, pointer, typing and saves
at each of three Nested levels, including a cold reopen of their saved state.
Use `--nested-depth 4` with `--latency` or `--nested-desktop` to exercise four
levels beneath the outer desktop. `--pointer-stress --trace-storage` adds motion
throughout the checks and recursive close diagnostics.

**Split** runs two independently saved Notes. Its **DEEPER** button enters another
copy of that matrix in a separate saved realm; unsaved work prevents replacement.
An unfunded further level reports **NO ROOM** and retains the live notes.
`python tools/boot-harness-rs/desktop_test.py --deep --trace-storage` exercises
this through F5's composition, including repeated close and cold reboot.

`python tools/boot-harness-rs/desktop_test.py --nested-desktop` exercises three
Nested desktops inside the outer desktop, deepest Notes persistence, dirty-close
protection and repeated subtree reclamation across reboot. Use Fresh Lab when
discarding stale template snapshots; new users then get current code at every depth.
`python tools/boot-harness-rs/nested_existing_test.py PATH_TO_SAVED_DISK --depth 4`
instead replays an untouched copy with its existing program snapshots and current
kernel/providers, without an installer. It expects one outer User 1 and stock
compact chrome; old executables may still time out. See the
[harness guide](../tools/boot-harness-rs/README.md#ordinary-desktop-timings).
The stock bottom bar is 80 pixels high; Settings changes bar height and spacing.

`python tools/boot-harness-rs/nested_failures_test.py PATH_TO_THREE_LEVEL_IMAGE`
faults the local picker, selected user and background on disposable copies, checking
bounded retries and a surviving outer draft. Supply the image from the nested test.
`python tools/boot-harness-rs/storage_capacity_test.py PATH_TO_ONE_USER_IMAGE`
fills real retained quota on a copy of the completed ordinary desktop fixture,
checks refused saves and close-save, sibling reads, reload and unchanged records
after cold reboot. Physical extent refusal/reopen is also covered by the real-engine
host test in `platform/libraries/database-objects/capacity_tests.rs`.

`python tools/boot-harness-rs/lifecycle_test.py PATH_TO_ONE_USER_IMAGE` copies an
existing one-user desktop fixture and injects invalid code, an early app fault,
a fault after admission, and a user-matrix CONFIGURE fault. It checks repeated
failures, visible recovery, healthy app saves and normal close; the source image
is unchanged. Fault-only features are built into those copied programs and are
absent from normal F5. The headless `--matrix-storage-test` also checks scoped
launch receipts, placement rollback, broken control channels and close/exit races.

`python tools/boot-harness-rs/launcher_recovery_test.py PATH_TO_ONE_USER_IMAGE`
checks one local launcher restart while Notes keeps a draft, retry exhaustion,
explicit pointer/keyboard retries, repeated startup failure, fallback failure/exit,
clean exit, owner stop/revocation, fresh layout authority and cold persistence.
Its injected programs and disks are disposable copies; ordinary F5 has no fault keys.

`python tools/boot-harness-rs/management_test.py PATH_TO_ONE_USER_IMAGE` exercises
the launcher's **END APP** confirmation with a non-yielding Notes process, both
Cancel actions, saved-state retention, a recovered launcher and cold reboot.
It checks that ungranted app/storage endpoints, stale task tickets and revoked
manager authority cannot stop an app. Only disposable copies receive fault keys.

Recent integrated proofs: `python tools/boot-harness-rs/run.py --matrix-desktop-test --release`
checks recursive atomic presentation; `--matrix-storage-test --release` checks
[private root delegation](platform/libraries/matrix/world/storage/README.md).
The smoke suite covers [combined IPC/IRQ/deadline waits](kernel/core/user_tasks/devices/README.md).
Run these commands from the repository root.

`python tools/boot-harness-rs/matrix_composition_test.py` checks an
[independent three-peer matrix](distribution/lab/matrix-composition/README.md)
over real IPC, with no stock matrix/desktop contract or storage disk. The ordinary
F5 matrix shares its local grant and input command implementation. Add `--omega`
to run the independent Omega implementation, or `--upstream` to run it under a
real parent with nested input/view attenuation and live-subtree reclamation.
These are headless boundary proofs, not additional F5 desktops. The normal stock
matrix now executes its [close/loss decisions](../source/distribution/matrix/lifecycle/README.md),
[ticketed asynchronous launcher recovery](../source/distribution/matrix/recovery/README.md),
[ordinary launch orchestration](../source/distribution/matrix/launch/README.md) and
[local startup recovery](../source/distribution/matrix/startup/README.md)
in Omega, with authenticated asynchronous IPC and task cleanup retained in Rust.

The installed Split matrix also has a bounded masked capture demonstration:
F5 -> user -> Split -> CAPTURE. HIDE RIGHT can remove the source presentation
without preventing authorized capture. See [Split's guide](distribution/applications/split/README.md)
for the fixed mask and current limits. `python tools/boot-harness-rs/desktop_test.py
--capture` verifies actual delivered pixels; `--capture-case ordinary|source|recipient|timeout`
selects normal delivery or disposable lifecycle faults, each on fresh/cold boots.

Notes **F6** toggles the cursor preference for pointers in that keyboard's group.
`python tools/boot-harness-rs/desktop_test.py --cursor-preferences --trace-storage`
checks input while hidden, independent AUTO, restoration on target exit and the
same behavior through Split-in-Split. `--cursor-producer --trace-storage` installs
a disposable AUTO that invokes F6 on its own group and verifies that physical
input remains usable. Both workflows run on fresh and cold boots.

## Start reading

For ongoing work or an agent handoff, read [TASKS_RUST.md](../TASKS_RUST.md) first.
It records the current default boot, recent changes, priorities and test evidence.

Follow [boot main](kernel/boot/uefi/main.rs) for machine startup,
[the user-task session](kernel/core/user_tasks/user_tasks.rs) for kernel execution,
[desktop policy](distribution/desktop/session/shell/shell.rs) for distribution startup, or
[the Notes workflow](../source/distribution/applications/notes/app/session/session.omg) for
input, persistence and drawing. The [composition map](distribution/README.md)
bridges executable and IPC boundaries. The [discoverability convention](../wiki/architecture/discoverability.md)
explains how to preserve these routes as the lab grows.

Public boundaries also have owners: [kernel contracts](kernel/contracts/README.md)
define syscalls and grants; [platform contracts](platform/contracts/README.md) define
service messages. These packages import no implementations.

Within a subsystem folder, start at its named entrance (`memory/memory.rs`, for
example) for state, sequencing or dispatch, then follow the responsibilities
beside it. `mod.rs` only wires Rust modules, exports and the included entrance;
it disappears in the Omega port. Small leaves remain ordinary source files.
The entrance explains this level without exposing every detail below it.

## Layout

| Path | Responsibility / Omega counterpart |
| --- | --- |
| `kernel/boot/uefi/main.rs` | Visible orchestration of firmware entry and post-handoff kernel startup |
| `kernel/boot/uefi/firmware/firmware.rs` | UEFI crate adapter and memory-inventory policy |
| `kernel/boot/uefi/graphics.rs` | GOP snapshot before firmware handoff |
| `kernel/boot/uefi/startup.rs` | Admit a supplied initial program and its bounded launch grants |
| `kernel/boot/uefi/lab/lab.rs` | Smoke-only orchestration, fixtures and display recovery verification |
| `kernel/boot/uefi/memory.rs`, `handoff.rs` | Compose core frame policy with architecture mappings, then transfer boot state |
| `kernel/boot/uefi/interrupts.rs`, `diagnostics.rs` | Interrupt bring-up and serial/fatal reporting |
| `kernel/boot/uefi/heap.rs` | Heap installation; smoke-only heap verification |
| `kernel/boot/uefi/lab/kernel_tasks/kernel_tasks.rs`, `lab/kernel_tasks/lifecycle.rs` | Cooperative/preemptive workloads and dynamic lifecycle checks |
| `kernel/boot/uefi/lab/user_tasks/user_tasks.rs` | Ring-3 syscall, isolation, fault containment and admission rollback experiments |
| `kernel/boot/uefi/lab/applications.rs` | Run the bundled executable twice per session and check loading, private state, exit statuses and cleanup |
| `kernel/boot/uefi/lab/ipc/ipc.rs` | Compose explicit endpoint grants and check service/failure sessions |
| `kernel/core/user_tasks/lifecycle/supervision/supervision.rs` | Host-testable launch authority, child identity and outcome consumption |
| `kernel/core/user_tasks/time.rs` | Boot-issued clock grants and wrapping deadline comparisons |
| `kernel/boot/uefi/lab/supervision/supervision.rs` | Supply launch bounds and verify userspace recovery/cleanup |
| `kernel/boot/uefi/lab/supervision/watchdog.rs` | Verify hung-service cancellation, deadline wakeups and independent progress |
| `kernel/core/ipc/ipc.rs` | Process-local endpoint grants, dynamic channels, checked moves, queues and teardown |
| `kernel/boot/uefi/smoke/smoke.rs` | Test-only fault injection and QEMU result reporting |
| `kernel/core/memory/inventory.rs` | Core-owned memory inventory validated at firmware handoff; internal kernel data |
| `kernel/contracts/user.rs` | Shared experimental entry/syscall constants for kernel and user runtime |
| `kernel/contracts/memory.rs`, `kernel/core/user_tasks/memory/regions/regions.rs` | Page-object grants, quotas, sealing and reader lifetime |
| `kernel/core/user_tasks/memory/memory.rs` | Boot-stack frame ownership, mapping changes and cleanup |
| `distribution/lab/memory/main.rs` | Private allocation, shared leases and permission-fault exercises |
| `distribution/lab/rendering/main.rs`, [investigation](distribution/lab/rendering/README.md) | Checked row delegation versus page-backed flattened composition, host costs and guest bounds probes |
| `kernel/contracts/framebuffer.rs` | Kernel framebuffer aperture, geometry and DISPLAY_INFO record |
| `platform/contracts/display/display.rs` | Userspace drawing requests; sibling files own composition, pointers and focus |
| `platform/libraries/block-device/lib.rs`, `platform/contracts/storage.rs` | Experimental block durability and private-object messages |
| `platform/drivers/ata-pio/` | Userspace bounded PIO driver for a dedicated QEMU data disk |
| `platform/services/storage/` | Private object provider: the disk grant, engine recovery, two private roots and sealed attachments |
| `platform/libraries/storage-volume/`, `distribution/lab/volume/` | Shared envelope and byte/sector adapter; separate transport and guest engine proofs |
| `platform/libraries/database-objects/` | Numbered objects over immutable content: shared chunks, cheap copies, pins and collection |
| `platform/contracts/input/input.rs` | Experimental physical-key events, independent of scan codes or UI policy |
| `platform/contracts/display/pointer.rs` | Experimental logical pointers, source updates, capture and scoped synthetic leases |
| `platform/contracts/display/interaction.rs` | Experimental attenuated keyboard focus and cross-process child placement |
| `kernel/core/memory/memory.rs` | Bootstrap frame accounting and reclaiming bitmap over usable RAM, corresponding to the resource work in `source/kernel/core/` |
| `kernel/core/memory/heap.rs`, `kernel/core/scheduler/scheduler.rs`, `kernel/core/kernel_tasks/kernel_tasks.rs` | IRQ-safe heap, pure scheduling policy, task admission and context/stack lifetime management |
| `kernel/core/user_tasks/user_tasks.rs` | Experimental user-task lifetime, checked diagnostic syscalls and outcomes |
| `kernel/core/user_tasks/admission/elf/elf.rs` | Bounded, host-testable ELF64 preflight before physical admission |
| `kernel/core/user_tasks/devices/devices.rs` | [Register/IRQ grants and driver replacement](kernel/core/user_tasks/devices/README.md), paired with the syscall/board entrances |
| `distribution/lab/devices/main.rs` | Userspace serial loopback, authority rejection, masked interrupts, fault/restart and blocked cancellation |
| `platform/drivers/uart_16550/` | Userspace interrupt protocol plus polling serial diagnostics, corresponding to `source/platform/drivers/uart_16550/` |
| `platform/libraries/user-runtime/` | Entry stub, syscall and protocol clients, linker script and optional funded heap |
| `platform/services/display/` | Separately compiled userspace provider for linear framebuffer drawing |
| `platform/services/input/` | Userspace PS/2 configuration, keyboard/mouse decoding and IPC event delivery |
| `platform/libraries/matrix/` | Reusable view/input/surface/scope authority with caller-selected child capacity |
| `distribution/matrix/` | Stock matrix executable, desktop configuration and recovery; [headless boundary lab](distribution/desktop/README.md) |
| `distribution/desktop/` | Default desktop: provider admission, chooser and user-matrix policy, storage supervision |
| `distribution/libraries/boot-scene/` | Retained test-pattern drawing client shared by the display lab |
| `platform/libraries/bitmap-font/` | Original 5-by-7 printable ASCII glyphs |
| `kernel/core/ipc/link.rs` | Boot-approved peer graph without lifecycle rights |
| `distribution/lab/display/` | Test-pattern layout, display client and restart policy |
| `distribution/lab/supervision/` | Userspace restart policy, persistent client and crashing echo service |
| `distribution/lab/channels/` | [Ordinary parent/worker channels](distribution/lab/channels/README.md), endpoint transfer and reclamation |
| `distribution/lab/ipc/` | Client, echo service and hostile IPC fixture roles in a standalone ELF |
| `distribution/lab/hello/` | Independently compiled `no_std` program exercising initialized data, BSS, yields and writes |
| `kernel/arch/lib.rs`, `kernel/arch/isa/isa.rs` | Explicit interface and compile-time CPU selection; [machine boundary and ownership](kernel/arch/README.md) |
| `kernel/arch/board/board.rs` | Actual board device selection; PC ATA/PS2/PIC/PIT and virt PL011/GICv2 adapters |
| `kernel/core/user_tasks/address_space/address_space.rs` | Shared image layout, frame ledger, construction rollback, checked copies and borrowed region mappings |
| `kernel/arch/isa/aarch64/` | EL1/EL0 paging, guarded task stacks, contexts, architectural timer and headless portability proof; [scope and commands](kernel/arch/isa/aarch64/README.md) |
| `kernel/arch/isa/x86/` | Shared x86 instructions; PC controller policy lives in `arch/board/pc/` |
| `kernel/arch/isa/x86_64/` | Paging, dynamic guarded stack mapping/teardown, CPU contexts, GDT/TSS/IDT and interrupt stubs, using the `x86_64` crate |
| `kernel/arch/isa/x86_64/user/user.rs` | CR3 entry/return policy; `memory/memory.rs` implements translation operations |
| `distribution/profile.json` | Built-in distribution composition consumed by the boot harness |
| `../tools/boot-harness-rs/` | Host build, QEMU launch and smoke verification |

Guest code and shared libraries use `no_std`; the rendering benchmark has a
separate host-only executable using `std`. Core policies are host-testable and have no firmware
dependency. Core's hardware-facing modules explicitly opt into unsafe code and
depend on `kernel/arch/`; drivers never depend on core internals. Boot assembles these
subsystems and the firmware adapter.
The bootstrap UART currently runs privileged; the display, input and storage providers run in userspace. Boot
supplies its port operations; the platform driver imports no kernel package.
`python tools/source-layout/check.py` checks these boundaries from the repository
root. The single `distribution/` is replaceable by forks; platform and kernel do
not import it.
The Rust-specific `kernel/arch/` layer groups hardware mechanisms that Omega currently
spreads across core providers, instruction contracts and libraries. x86-64 and
AArch64 boot through UEFI; shared x86 instructions do not imply a working 32-bit kernel.
UEFI ABI definitions and Boot Services come from the upstream `uefi` crate.
`foundation/` and distribution desktop packages appear when their first code lands,
following the same rule as the Omega tree.

## Run

Prerequisites: Rustup, Python 3.10+, QEMU and OVMF. The workspace pins Rust 1.94.0
and the `x86_64` / `aarch64` `unknown-uefi` and `unknown-none` targets; Rustup
installs them when Cargo is run inside `source-rs/`. The commands below select
x86-64; use the separate [ARM64 runner](kernel/arch/isa/aarch64/README.md) for its proof.
Dependencies are locked in `Cargo.lock`. No nightly language features are used.

From the repository root:

```text
python tools/boot-harness-rs/run.py --build-only
python tools/boot-harness-rs/run.py --smoke
python tools/boot-harness-rs/run.py --kernel-only --smoke
python tools/boot-harness-rs/run.py --window
python tools/boot-harness-rs/run.py
```

The final command boots the desktop with the serial console on stdio; every
service blocks when there is no work, leaving the CPU idle with timer wakeups.
Ctrl+C stops QEMU. `--window` shows the display during an ordinary boot.
`python tools/boot-harness-rs/startup_test.py PATH_TO_ONE_USER_IMAGE` checks
missing login paths, invalid/crashing picker code and explicit retry on disposable
copies. Its transient-refusal fixture verifies recovery into the normal picker
and a working app; it does not modify the supplied image.

`python tools/boot-harness-rs/provider_startup_test.py PATH_TO_ONE_USER_IMAGE`
checks crashed, unresponsive and malformed storage startup, damaged volume
headers, responsive input while waiting and manual retry into a working desktop.
The two restart-budget policy tests run with
`python tools/boot-harness-rs/recovery_policy_test.py`.

`python tools/boot-harness-rs/provider_live_test.py PATH_TO_ONE_USER_IMAGE`
injects precommit/lost-reply stalls into the F5 storage provider while Notes holds
a draft. It checks reconciliation, replacement readiness timeout, exhausted
restart budget, editing and close protection while offline, and exact cold-open
contents. The supplied default-layout, one-user image is copied, never modified.
Use `--case precommit`, `postcommit`, `exhausted` or `readiness-stall` for a subset.

`python tools/boot-harness-rs/bootstrap_test.py PATH_TO_ONE_USER_IMAGE` checks
initial display/root/input death, stall, malformed readiness and admission
refusal. Each is exercised once and persistently: successful retry reaches a
usable app, exhausted retry leaves storage byte-identical, and guest resource
snapshots check rollback. Root/input exhaustion also checks the restart screen.
The fixture uses disposable copies and separate boot files.

`python tools/boot-harness-rs/desktop_test.py` drives that same composition
through QMP and checks its scanout; it uses only Python's standard library.
Add `--nested-desktop --pointer-stress --interaction-delay --timeout 240` to
exercise three nested desktops with continuous pointer movement and five-second
upstream poll, image and publication stalls on fresh and cold boots, and a
two-second stall of the outer desktop's storage during which an app there is
closed, its answer landing only after the stall. The test also injects a mismatched poll
reply and requires local child polling during the stall, draft survival, clean
close (the answer that lands after the storage stall, inside its grace, must
be accepted, not refused as late) and repeated subtree reclamation. Its
`interaction-delay` feature is absent from normal F5 builds.
`--release` selects an optimized build. `--memory 256` selects guest RAM in MiB.
Smoke mode has a 30-second boot deadline, requires ordered serial milestones,
and checks the QEMU debug-exit status. A panic, missing milestone, reset or hang
fails the run. Its debug-exit feature is not enabled for ordinary boots.
The harness reads the distribution profile. Ordinary boots build only `startup.initial`
and the optional `startup.launches` array; smoke builds instead build the named
`user_programs` fixtures. It supplies these ELF artifacts to the UEFI build with
`bundled-user`. `--kernel-only` omits that feature and builds no user executables.
It can be combined with `--smoke` to test raw user isolation without platform
services or distribution programs.
Both builds use the selected debug/release profile. There is no executable filesystem loading
or executable download in the guest, and no Cargo dependency from boot to the
distribution. The boot build tracks each selected artifact's path and contents
through Cargo's build-script inputs.

Exercise actual exception delivery separately:

```text
python tools/boot-harness-rs/run.py --smoke --fault guard
python tools/boot-harness-rs/run.py --smoke --fault invalid-opcode
python tools/boot-harness-rs/run.py --smoke --fault double-fault
```

These require the expected vector and debug-exit status. The guard case also
checks CR2 and the page-fault error bits; the double-fault case checks that the
diagnostic runs on its dedicated emergency stack. Probe builds end at the fault
and do not proceed to timer setup. Normal builds exclude the smoke module.

The runner finds QEMU on PATH or at the usual Windows installation. Set `QEMU`
to override the executable. It discovers bundled/system OVMF; alternatively set
`OVMF` to a combined image, or both `OVMF_CODE` and `OVMF_VARS` to a matching
split pair. Firmware variable stores are private copies reset on every run.
Images and logs go under `build/boot-harness-rs/`. Smoke serial output is saved
in `build/boot-harness-rs/smoke/serial.log`.

From `source-rs/`:

```text
cargo test --locked
python ../tools/source-layout/format-rust.py --check
cargo check-uefi
cargo build-uefi
cargo build --locked --package cathedral-hello --target x86_64-unknown-none
cargo check-userspace
```

The default Cargo members test contracts, core, storage, input decoding, fonts and
rendering algorithms on the host. `check-uefi` selects the kernel packages;
`check-userspace` checks the remaining packages and the rendering guest.
Profile-specific features are exercised by their harness commands. Kernel crates
use the UEFI target; the user program/runtime use the freestanding ELF target,
so the whole workspace cannot be built for one target. `cargo build-uefi` builds
a standalone kernel with no initial program; use the Python harness for the
composed distribution image or add `smoke-test` for kernel-only exercises. Run Cargo inside this directory so its toolchain, aliases and relative
linker-script path apply. The x86-64 target uses SysV and AArch64 uses AAPCS64;
the runtime's CPU entry stub establishes the call boundary before invoking Rust.

## Current milestone

QEMU q35, one qemu64 CPU, software emulation, 128 MiB by default:

1. Enter a real UEFI application through OVMF and report over COM1.
2. Capture the final firmware memory map and exit Boot Services using `uefi`.
3. Disable maskable interrupts and adapt the map into a fixed-capacity inventory.
4. Reject overlapping, unaligned, overflowing or oversized inventories.
5. Move the inventory into the core's frame allocator, skipping reserved memory
   and physical page zero.
6. Deep-copy the active four-level page tables into exclusively allocated frames,
   preserving inherited leaf mappings and their flags. Bound copying to 4,096
   table pages; fail on exhaustion, unsupported NX/LA57/PCID or occupied ranges.
7. Map a guarded 64-KiB kernel stack, four guarded 16-KiB emergency stacks,
   and 256 KiB of heap backing. Reserve a separate virtual branch for dynamic task
   stacks, without allocating their backing yet. New mappings are NX.
8. Load the owned CR3, enable write protection/NX, switch stacks and confirm the
   stack pointer is within its assigned range.
9. Load Cathedral's GDT/TSS and complete 256-entry IDT. Double fault, NMI,
   machine check and maskable interrupts have distinct emergency stacks.
   Execute `int3` and verify return.
10. Initialize the reclaiming heap. Smoke builds exercise alignment, exhaustion,
    `Vec`/`Box` allocations, writes and frees.
11. Start the 100-Hz PIC/PIT bootstrap timer. Receive three ticks and verify the
    IRQ used its assigned stack.
12. Enable physical-frame reclamation using a heap-allocated bitmap; earlier boot
    allocations remain permanently reserved. Run two cooperative kernel tasks
    through yield, sleep, wake and return; one progresses while the other sleeps.
    Allocate guarded stacks on admission, then unmap and free them after exit.
13. Run two non-yielding tasks under timer preemption. Stress heap allocation
    while switching, then verify distinct GPR/SSE/x87/MXCSR patterns survive.
    Require each task to observe its peer making progress before it exits, and
    require actual timer-driven switches away from both tasks.
14. Fail stack admission at all 19 initial allocation boundaries, including
    partial intermediate page tables; verify complete rollback. Also fail with
    live peer tasks, and exhaust the heap to exercise context-admission rollback.
15. Dynamically spawn and retire 128 child tasks across two sessions while a
    non-yielding peer runs. Verify task limits, stale IDs, guards/canaries and
    return to heap/frame baselines after each pair and each session.
16. Build independent sparse task roots with explicitly retained supervisor-only
    image, heap and entry stacks. Enter ring 3 with private RX code, RW/NX data
    and a guarded 32-KiB user stack. Switch back to the kernel root on entry.
17. Exercise diagnostic write, yield and exit through `int 0x80`; reject unknown
    calls, kernel/noncanonical/overflowing pointers, guard-crossing buffers and
    oversized writes. Copy valid code, data and stack buffers, including binary
    bytes. Preempt non-yielding user tasks while checking private data and FP state.
18. Contain ten deliberate user faults: kernel-memory read, CLI, port I/O,
    code-page write, stack execution, guard write, privileged interrupt gate,
    invalid opcode, physical-root alias read and CLI with an invalid user RSP.
    Check vector/error/address, peer progress after each fault and complete cleanup.
19. Fail at every physical-frame admission boundary across two user tasks,
    including a fully admitted first task and partial second root. Require heap
    and frame baselines after every failure and completed session.
20. Validate the separately built ELF and map its RX text, R/NX read-only data
    and RW/NX data/BSS into two private roots. Execute through the shared user
    runtime with distinct initial arguments and collect both exit statuses.
21. Repeat with fresh instances to check zeroed BSS, initialized data and private
    state across yields. Exercise writes spanning page and syscall-size boundaries.
22. Reject a malformed ELF after a valid peer was admitted, then fail at every
    frame-admission boundary for the ELF pair; require complete memory reclamation.
23. Boot-grant request/reply endpoints to isolated ELF client/service instances;
    repeat 32 echo exchanges in each of two sessions. Reject wrong rights,
    raw-name authority and closed local handles; require actual blocking.
24. Revoke an endpoint or exit/fault its sender while its receiver is blocked;
    require the corresponding error and complete heap/frame reclamation.
25. Fill a bounded queue, reject bad/read-only/cross-page copyout destinations
    and undersized receives without losing its message; drain after sender exit.
26. Give a userspace supervisor a bounded launch grant. It spawns an isolated
    echo service, collects a kernel-reported fault after reclamation, and repeats
    32 times while the same client explicitly reconnects and checks stale grants.
27. Check supervisor-exit cancellation, complete returned-status delivery, bad
    wait destinations, and repeated failed spawns while peers remain alive.
28. Grant clock access to a supervisor and observer. Recover four blocked and four
    syscall-free spinning services using deadline waits and explicit cancellation;
    verify unrelated progress, stale grants and complete memory reclamation.
29. Check completed-outcome precedence, clock copyout and deadline wakeups when
    every userspace task is blocked.
30. Allocate bounded private pages and seal shared buffers to an authorized peer.
    Check write/NX faults, quotas, stale handles, peer death, pinned-reader lifetime,
    zeroing/reuse, allocation rollback and complete heap/frame reclamation.
31. Grant a reserved GOP framebuffer to an isolated display provider. Draw a
    distribution-owned pattern, fault/restart the provider, reconnect and redraw
    while an independent observer progresses. Reject ungranted access and check
    NX, guard pages, copy boundaries and every admission allocation failure.
32. Print `CATHEDRAL_RS_BOOT_OK`, then idle or terminate the smoke-test guest.

Only ordinary conventional RAM is eligible. Loader memory, Boot Services memory,
runtime memory, ACPI and MMIO remain reserved. Runtime-marked, hot-pluggable and
specific-purpose conventional regions also remain reserved. This deliberately
retains the image, map buffer, firmware stack and inherited page tables. The
inventory accepts at most 256 descriptors and fails closed beyond that.

Core accounts for physical frames; only the architecture backend accesses them.
Inherited identity mappings remain, including aliases of the new backing frames;
this is bootstrap address-space ownership, not user isolation or a final W^X
policy. Old firmware tables/storage are still reserved, not reclaimed. The heap
uses `linked_list_allocator` behind an interrupt-masked lock. The smoke boot
exercises allocation/free, page alignment, exhaustion and complete reclamation.
Fatal and NMI handlers must never allocate. Display, input and storage providers
run in isolated user tasks; the bootstrap UART remains privileged.
Timer, yield and syscall entries save all GPRs, the return frame, and x87/MMX/SSE
state before entering an allocation-free scheduler callback. Each suspended
task's context is copied into stable heap storage; no task retains a frame on
the shared IRQ stack. The selected qemu64 profile has no AVX state to save.
Tasks return normally so Rust drops their owned data. Exit switches to the boot
context before removing the saved context, unmapping the stack, invalidating
translations and releasing its physical backing. Empty paging-structure frames
are detached and released after a TLB flush. Each stack slot has its own 2-MiB
virtual range with 64 KiB of backing, so a peer's live leaf table is unaffected.
The callback does no allocation, mapping or freeing. Spawn requests also switch
to the boot context, where partial admission is rolled back on failure.
Exception stubs normalize hardware error codes before a terminal diagnostic;
the breakpoint self-test resumes. Vector 48 handles cooperative suspension on
the IRQ stack, through the same complete-context path as the timer. Unexpected
external vectors report 255.
The CPU profile is qemu64, with no claim to optional virtualization exception
semantics or physical-hardware coverage. Unsafe wrappers are boot-only lab
mechanisms, not application APIs.

The runtime supports a configurable limit of 1ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã¢â‚¬Å“64 trusted kernel tasks on one
CPU (default 8), plus an optional frame budget for stacks and their tables.
`kernel_tasks::spawn(fn())` returns a `TaskId` or an admission error. IDs contain slot
generations and are scoped to a session; reused slots do not revive old IDs.
`is_alive` reports liveness; return is the supported exit mechanism. There are
no joins, captured closures, cancellation or recovery from task panic yet.
These trusted kernel tasks share an address space; guards protect against stack
overrun, not against malicious tasks. There is no capability enforcement or SMP. The
256-KiB heap and runtime arena are initialized before executing tasks; ordinary
infallible Rust allocations can still panic on exhaustion. Task context
admission is fallible and returns `OutOfHeap` without leaking stack frames.
The scheduler separates task states from [hierarchical turn selection](kernel/core/scheduler/README.md).
Kernel tasks use its flat round robin; user launch subtrees subdivide their owner's
outer turns. IRQ masking protects its mutable session, and heap
locks are never held across a voluntary suspension. Preemption is tested with
tasks that never yield, including a negative control that fails when their
timer preemption is disabled.

The CPU subtree smoke test compares one versus three real userspace spinners
against an independent outside account. Their combined turn share stays bounded
by their owner. The [measured-budget extension](kernel/core/scheduler/README.md#measured-execution-and-configuration)
also charges partial execution, spawn attempts, page operations and reclamation.
Profile `cpu_budget_us` sets a persistent burst allowance with continuous refill
at allowance/100,000 of one CPU; zero inherits ancestor ceilings. Exhaustion gates
the whole subtree, and overrun debt survives replacement. The single-CPU QEMU clock
and periodic timer remain experimental: there are no hard deadlines, guaranteed
reservations, virtual cores or service-call budget donation yet.

## User-mode experiment

User sessions run 1-8 tasks from either the one-page assembly probes or restricted
static ELF images. Each task has its own root and physical backing
at the same user virtual addresses, with no inherited firmware identity map.
Only the live kernel image, boot/emergency stacks and context-storage heap are
retained supervisor-only for entry/return. The kernel root still has its original
identity aliases. Global translations are disabled before user execution; CR3
switches flush task translations. This is architectural memory/privilege
containment on qemu64, not a claim of speculative-execution mitigation or full
isolation between trusted kernel tasks. [Kernel image protection](kernel/arch/memory/README.md)
now enforces section permissions and NX physical aliases. Hardware rules follow the
[Intel system programming manual](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html).

The temporary ABI uses `int 0x80`, RAX for the call/result and RDI/RSI/RDX for three
arguments. Other saved registers and floating-point state survive calls and
preemption. Initial registers are cleared apart from the explicit arguments;
initial FP state is clean and I/O privilege is zero.

| Call | Arguments | Result |
| --- | --- | --- |
| 0: diagnostic write | Address, byte count | Count copied, or negative error |
| 1: yield | None | 0 |
| 2: exit | Status | Task cannot resume; status recorded for boot |
| 3: IPC handle | Local grant index | Already-installed ticket, or negative error |
| 4: IPC send | Ticket, address, length | Bytes queued, or negative error; never blocks |
| 5: IPC receive | Ticket, address, capacity | Bytes copied, or negative error; blocks on empty |
| 6: IPC revoke | Revoke ticket | 0, or negative error |
| 7: task launch grant | Caller-local launch index (0 for bootstrap) | Caller's boot-issued launch ticket |
| 8: task spawn | Launch ticket, ordinary argument | Child ticket after admission, or error |
| 9: task wait | Child ticket, destination, exactly 40 bytes | 0 after copying and consuming reaped outcome; blocks if live |
| 10: task port grant | Caller-local connection-port index (0 for bootstrap) | Designated peer's boot-issued port ticket |
| 11: task connect | Port ticket | First of two newly accepted local IPC grant indices |
| 12: clock read | Clock ticket, destination, exactly 8 bytes | 0 after copying the boot-local tick count |
| 13: task cancel | Child ticket | 0 after reclamation; leaves the outcome collectible |
| 14: task wait until | Child ticket, 40-byte destination, absolute deadline | Wait result, or `TIMED_OUT` (-110); requires a clock grant |
| 15: clock grant | None | Caller's boot-issued clock ticket, or `DENIED` |
| 16: display info | Destination, exactly 48 bytes | Boot-installed mapping geometry, or `DENIED` when no display grant |
| 17: keyboard read | 0: nonblocking, 1: wait | Raw keyboard byte or `512 | mouse_byte`, 256 on data loss, `WOULD_BLOCK` or `DENIED` |
| 18: keyboard write | 0: data or 1: command, byte | 0, `WOULD_BLOCK`, `INVALID_ARGUMENT` or `DENIED` |
| 19: IPC receive until | Ticket, 64-byte destination, absolute deadline | Bytes copied or error; requires clock access |
| 20: keyboard read until | Absolute deadline | Raw byte/loss or `TIMED_OUT`; requires keyboard and clock access |
| 21: peer link grant | Caller-local link index | Boot-approved connection ticket, no task control |
| 22: peer link connect | Link ticket | First of two accepted local IPC indices |
| 23: IPC wait two | Two receive tickets, absolute deadline | Index 0/1 ready, or error; requires clock, consumes no data |
| 24: Disk register read | Fixed register index 1?8 | One byte; exclusive disk grant required |
| 25: Disk register write | Fixed register index, byte | Whitelisted PIO command/configuration only |
| 26: Disk transfer | Buffer, direction (0 read / 1 write), length 512 | Checked full buffer before bounded single-sector I/O |
| 27: Memory allocate | Page count | Zeroed private RW/NX region handle, within explicit budget |
| 28: Memory address | Region handle | Caller-owned or accepted mapping address |
| 29: Memory release | Region handle | Remove caller mapping; owner waits for peer completion/death |
| 30: Memory seal | Region handle, link token | Irreversibly remove WRITE and offer to that live service |
| 31: Memory map | Producer-local offer, service link | Accept authorized RO/NX lease and return a fresh local reader handle |
| 32: Memory pages | Region handle | Caller-owned or accepted region page count |
| 33: IPC wait many | Pointer to 1ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã¢â‚¬Å“8 LE u64 receive tickets, count, absolute deadline | Ready index or error; validates the copied array and all receive rights, consumes no data, requires clock |
| 34: IPC create | 16-byte writable destination, 0, 0 | 0 after installing local send/receive handles; `NO_MEMORY` at quota |
| 35: IPC close | Local handle, 0, 0 | 0 after dropping the grant; copied handles become stale |
| 36: IPC send capability | Destination handle, 80-byte packet, 0 | Payload length after atomic one-endpoint move into the queue |
| 37: IPC receive capability | Receive handle, writable 80-byte packet, 0 | Payload length and optional installed handle; blocks on empty |
| 38: task spawn with endpoints | Launch grant, 56-byte request, 0 | Child ticket after admitting and moving the whole initial bundle; failure retains all source grants |
| 39: initial environment | Writable 48-byte destination, 0, 0 | 0 after copying count and up to five initial local endpoint names; never reinstalls grants |
| 40: task spawn with launch loans | Launch grant, 152-byte request, 0 | Child ticket after atomic endpoint moves and selected idle launch loans; failure retains all grants |
| 41: IPC budget | Writable 32-byte destination, 0, 0 | 0 after copying caller-local channel limit/charge and dynamic handle limit/charge as four LE u64 words |
| 42: CPU read | Local clock grant, 0, 0 | Caller-local direct CPU microseconds |
| 43/44: resource budget/assignment | Local output or owned idle launch and limits | [Conserved page/IPC quotas](kernel/core/user_tasks/resources/README.md) |
| 45-53: executable and process admission | Caller-local executable/launch grants | [Catalogue, funded reservations, code staging and idle CPU limits](kernel/core/user_tasks/admission/pool/README.md) |
| 54: physical budget | Writable 16-byte destination, 0, 0 | Caller-local reserved and charged physical frames as two LE u64 words |
| 55: physical assignment | Owned idle launch, requested frame reservation, 0 | Conserving reassignment; BUSY if a live charge would lose funding |

Writes accept at most 256 bytes within one known user page. The kernel validates
the entire range and copies through its physical backing before calling the
bounded diagnostic sink; it never follows an unchecked user pointer. Bad
addresses return -14, excessive lengths -22, unknown calls -38 and sink failure
-5. Zero-length writes still require an address inside a user page. This sink
is a lab privilege, not a capability or a production console API.

User faults become task outcomes; kernel faults, NMI, double fault and machine
check remain fatal. Exit/fault first restores the boot context, which retires the
task's private tables and backing while peers remain runnable. Scheduling and
syscall dispatch do not allocate or reclaim memory. Sessions preallocate their
task/report storage; the scheduler's small arena still uses infallible allocation.
The runtime has no dynamic linker, general executable discovery, admission proofs,
dynamic device admission or production resource policy. Kernel and user workloads currently
run in separate boot-managed sessions.

## Executable loading experiment

The [ELF preflight](kernel/core/user_tasks/admission/elf/elf.rs) accepts little-endian ELF64,
System V ABI version 0, `ET_EXEC` for the selected CPU (x86-64 or AArch64). Limits are 4 MiB per file, 16 program
headers, eight load segments and 80 image pages total. Each load segment must be
readable, 4-KiB aligned in both file and virtual memory, have alignment 4096,
and fit in the 1-MiB image window starting at `0x8000000000`. Segments cannot
share pages or combine write and execute permission. The entry must lie within
file-backed executable bytes. The four-page stack has guards and sits separately
at image base + 2 MiB. Every task receives its own copies; BSS and page tails
are zeroed before execution.

`PT_NULL` and one non-executable `PT_GNU_STACK` are the only accepted non-load
headers. Interpreters, dynamic linking, TLS and other program-header semantics
are rejected. Section/debug tables are not used for loading. All offset/size
arithmetic is checked before slicing or allocating. These restrictions are lab
policy over the [ELF program-header format](https://gabi.xinuos.com/elf/08-pheader.html),
not a promise to execute arbitrary Linux binaries or an admission proof.

The [user runtime](platform/libraries/user-runtime/lib.rs) provides `entry!`,
`write`, `yield_now` and `exit`, using the shared experimental ABI. `write` splits
slices at both 256 bytes and page boundaries. Returning from the program exits
with its status; a Rust panic exits with 255. There is no unwinding. The optional `heap` feature installs an explicitly funded
process-lifetime allocator; see [the database proof](distribution/lab/database/README.md).
The [hello program](distribution/lab/hello/main.rs) contains no privileged
instructions or kernel imports. It checks more than one page of BSS and distinct
per-instance data, then returns the supplied status. Its link script explicitly
places LLVM's normal and large-code-model sections into matching segments.

The [kernel design](../wiki/design/part_5_lifecycle/04_kernel_architecture.md)
already calls for hardware walls around unproved apps and drivers. The default
for proved OS components remains an owner-level policy question in
[`OWNER_QUESTIONS.md`](../OWNER_QUESTIONS.md); these experiments do not freeze
Cathedral's component model or shared ABI.

## Capability IPC experiment

The [kernel IPC entrance](kernel/core/ipc/ipc.rs) coordinates local grant checks,
transfer reservation and message commit. [Boot admission](kernel/core/ipc/bootstrap.rs)
uses 32 reserved endpoint slots; [ordinary creation](kernel/core/ipc/channels.rs)
uses a separate pool of 32 dynamic channels. Each one-way channel has one sender,
one receiver, and one queued message of at most 64 bytes. Boot channels may also
have an explicitly assigned revoke holder. No syscall routes on a service name,
matrix identity, manifest or caller-supplied PID.

[Grant tables](kernel/core/ipc/grants.rs) resolve names using the trapping task's
kernel identity. IPC names contain no global task/endpoint/epoch fields. Equal
numbers in different processes may name different grants; copying a number never
installs authority. Closed or moved names remain stale throughout this session,
including task-slot replacement. Independent sessions and reboots have independent
namespaces. `IPC_HANDLE` enumerates admitted boot grants only. Dynamic endpoints
are returned directly by `ipc::channel()` and `Handle::receive_with()`.

The [ordinary-app channel guide](distribution/lab/channels/README.md) documents
creation, quota accounting, endpoint moves and the parent/worker QEMU proof:

```powershell
python tools/boot-harness-rs/run.py --ipc-test --timeout 90
python tools/boot-harness-rs/control_lifecycle_test.py
```

The lifecycle check boots the ordinary desktop with a delayed Notes loss reply.
It verifies continued input, rejection of a stale clean answer after editing,
saving and final close, plus the separate UI presenter's shared owner control.


`send_with` checks all inputs and reserves recipient capacity before moving one
dynamic endpoint into a message. A failed send preserves ownership. A successful
send invalidates the sender's name; successful `receive_with` publishes a
recipient-local name after validating its complete writable output. Its 80-byte
record is length u64, attachment u64 (zero means none on receive), then 64 payload
bytes. The runtime stages it in a 128-byte-aligned object. Plain receive refuses
attached authority with `TOO_SMALL` and preserves the message. Boot grants cannot
be moved; dynamic grants cannot yet be duplicated or independently revoked.

Each initial/launch account has a backed channel reservation: four by default,
or profile `ipc_channels` in 0..32. All accounts, including dormant children, must
fit the 36-channel pool. [Accounting](kernel/core/ipc/budget/budget.rs) retains the original
account's charge through endpoint moves, creator death and replacement; final
closure of both ends refunds it. Each recipient separately pays for its dynamic
handles, including hidden escrow, within a limit of 24. Creation also requires
available handles and fresh names. [`ipc::budget()`](platform/libraries/user-runtime/ipc/budget.rs)
reads only these caller-local limits/charges. The profile and kernel reject
overcommit, so unrelated accounts cannot exhaust promised channel storage.
Owners can subdivide these reservations through launch grants; capacity is
conserved down the tree. Retained endpoints can exhaust their original account.
CPU ceilings are accounted separately.

Send returns `WOULD_BLOCK` (-11) for a full queue. Ordinary receives validate the
complete destination before dequeueing or parking. Raw buffers must fit inside a
mapped user page; wrappers stage bytes so callers' slices may cross pages.
Short destinations preserve the message. Empty messages are valid. Pending
operations store checked integer ranges, not Rust references into user memory.
No message operation allocates or follows an unchecked virtual pointer.

Sender close/exit allows accepted bytes and transfers to drain before
`PEER_CLOSED` (-32). Receiver close/exit discards queued messages and closes their
unaccepted attachments. Bootstrap revocation discards the queue and returns
`REVOKED` (-125); replacement cannot inherit it. Explicit moves survive sender
death and preserve the transferred right. Wrong rights return `DENIED` (-13),
unknown local names `BAD_HANDLE` (-9). Dynamic channel slots are reusable after
both ends close; grant names are not reused. Admission and teardown remain bounded.

The original smoke fixture checks plain messages, rights, local-name collisions,
closed handles, blocked wakeups, peer failures and heap/frame reclamation. The
new channel fixture adds real creation/transfer syscalls and worker-death cleanup.
Host policy tests also cover quotas, escrow chains, forwarding, revocation and
atomic failure. Production transport, sponsorship and revocation semantics remain
experimental; see [`OWNER_QUESTIONS.md`](../OWNER_QUESTIONS.md).

### Deadline receives

`Handle::receive_until` requires explicit clock authority and stages a fixed
64-byte destination. It validates rights and writable memory before checking
expiry. An available message, revocation or peer closure wins over expiry at
observation. An empty expired wait returns `TIMED_OUT` without changing its
buffer. Timer interrupts also retire parked receives when every task is blocked.

Timeout removes only that receive. It neither revokes the endpoint nor cancels
a remote operation; a late reply can still queue. A caller must correlate/drain
late responses or replace the connection. Init takes the latter approach by
reclaiming the provider and accepting fresh local endpoint names before retry.
`keyboard::read_until` follows the same ready-before-timeout rule and additionally
requires the exclusive device grant. An input idle reply means the provider
completed a timed raw read; it is not evidence of a hung provider.

### Peer links and readiness

Boot may reserve request/reply links between child slots independently of their
launch owners. Both live participants must explicitly accept each incarnation.
A link ticket is caller-bound and conveys only acceptance authority; it cannot
spawn, cancel, select an executable, map a device or connect to an arbitrary peer.
Replacing either participant discards undrained old data, invalidates endpoint
tickets, and permits both participants to accept the new pair. Unaffected links
and private control channels retain their tickets and queued messages. The fixed
graph is lab composition, not service discovery, dynamic delegation or a registry.

`ipc::wait_two` validates both receive rights even if the first is ready. It
returns the first ready index without consuming data; queued data or peer closure
counts as ready. Revocation/stale handles return errors. Empty waits expire with
`TIMED_OUT`; timer wakeups also work when all tasks are blocked. Providers use this
to prioritize control requests without abandoning their application connection.

## Sealed objects over ordinary IPC

The [memory guide](kernel/core/user_tasks/memory/README.md) defines copied read
capabilities, optional RO/NX mappings, queued quota reservations and retained
sponsor funding. Start at `Private::freeze`, then `Handle::send_memory` and
`receive_memory`. This is independent of the legacy boot-link offer API.
Both CPU boot proofs exercise forwarding after producer death; the matrix desktop
uses it for retained images without pixel copies at intermediate matrices.

## Userspace supervision experiment

A session may reserve up to eleven launch grants within the twelve-task and
thirty-two-endpoint limits. Each reserves one child slot and two endpoint slots for a
boot-approved executable. Boot initially binds each launch grant to a supervisor
and a connection port to an initial task, which may be the supervisor itself. The grant selects the executable,
fixed first entry argument, peer, and physical-frame budget; userspace can
supply the second ordinary argument and optionally move a bounded set of its
own dynamic IPC endpoints into the child. It cannot nominate another image,
principal or device grants. A launch loan moves both owner and peer to its newly
admitted child; otherwise userspace cannot select the peer. Boot-static endpoint specifications cannot
refer to the reusable child slot. This is an authority-bound lab mechanism, not a general process API,
a manifest/provenance check or a component registry.

`task::Launch::spawn` parks its caller and hands admission to the boot context.
All context/metadata slots are preallocated; runtime spawn maps private physical
pages without growing the arena. Success returns a fresh owner-local child
name. Launch, peer, child, clock and link names come from a caller-local counter;
none contains a global task slot or epoch. Admission reserves names before mapping,
including the fresh launch/peer names used when a loan returns. A second child on the same grant or an uncollected previous outcome
returns `BUSY` (-16). `Launch::at(index)` and `Port::at(index)` select independent
caller-local grants; `bootstrap()` selects index zero.
Failed ELF admission returns `BAD_EXECUTABLE` (-8); failed frame admission
returns `NO_MEMORY` (-12), rolls back, and permits another attempt. Name and internal epoch
exhaustion also fail closed. No mapping or reclamation runs in a trap callback.

`Launch::spawn_with(argument, endpoints)` additionally installs up to five dynamic
endpoint rights before the child can run. Its [startup environment guide](distribution/lab/channels/README.md#initial-child-environment)
connects the `entry_with_environment!` macro, checked syscall request, admission
and atomic commit. All rejected bundles and failed ELF/frame admissions preserve
the parent's endpoints. The environment snapshot contains only explicitly supplied
initial names, whose roles the application defines. It supplements the profile's
boot connection and device/clock authority; it is not a manifest or complete
process-authority inventory.

`Launch::spawn_tree(argument, endpoints, launches)` also lends selected idle
launch/peer grants at the same atomic admission boundary. Its [recursive admission
guide](distribution/lab/channels/README.md#recursive-child-admission) covers the
finite inherited budget, caller-only enumeration, subtree teardown and fresh
names on return to a surviving lender. Executables and per-slot ceilings stay
fixed. The same ordinary userspace algorithm runs at each level of the QEMU proof;
the matrix service now uses these loans for its actual children. See the
[implementation map](STATUS.md) for the visible integration and its restrictions.

The child receives request-receive and reply-send grants. Its designated peer's
matching grants remain hidden, including from guessed-ticket redemption, until
`task::Port::connect` explicitly accepts that instance. It returns request-send
and reply-receive handles through the platform wrapper. A second connection to
the same child returns `BUSY`; connecting without a live child fails closed.
Existing control channels and sibling service pairs keep their identity and
queued messages across restarts. Reconnecting an earlier pair returns its actual
local index even while later sibling pairs remain visible.
A replacement discards any undrained bytes in the retired child's endpoint pair;
old tickets stay invalid, even for a new child occupying the same task slot.

`task::Child::wait` blocks until the kernel has reclaimed the child, then returns
its full exit status or kernel-observed fault details. Pending reclamation keeps
priority across interrupts that arrive while returning to the idle boot context;
runnable peers cannot starve collection of an exited task. The wire record is five
little-endian u64 words: kind, status/vector, error, address, instruction. Kind
0 means returned status, 1 means fault, 2 means cancelled; unused words are zero.
The raw destination must be exactly 40 bytes within one writable user page.
Invalid destinations preserve the outcome; successful copyout consumes it.
Duplicate waits and retired child tickets cannot observe a replacement. The
platform wrapper uses an aligned staging buffer and exposes `task::Outcome`.

The distribution chooses when to spawn again, what failures merit restart, and
how to coordinate reconnection. Its fixture runs 32 crash/restart cycles, checks
fresh child state, requires the client to observe `PEER_CLOSED`, and rejects
stolen launch/child tickets and stale IPC tickets. The kernel implements no echo
protocol or retry policy. It asserts physical accounting against
all remaining live tasks after every reclamation; boot checks heap and physical
baselines after each complete session. Additional sessions exercise returned
status (including all 64 bits), bad wait copyout, supervisor exit/fault cleanup, malformed
executables, and repeated frame failures at budgets 0, 1 and 10.

Supervisor exit or fault cancels all its owned children before userspace resumes
and reclaims them even if blocked on IPC or raw keyboard input. This is abrupt hardware-task teardown,
without user destructors or a graceful drain. Nested launch loans cascade this
teardown through descendants; reclaimed budgets return to surviving lenders.
No orphan adoption, general kill or persistent recovery is implemented.
Component supervision versus task-scope ownership remains an explicit
question in [`OWNER_QUESTIONS.md`](../OWNER_QUESTIONS.md).

### Deadline waits and hung-service recovery

Boot explicitly grants clock access to selected task slots through
`Config::clock_readers`. Reserved children inherit none, but boot may separately
grant a reserved launch slot clock access across its incarnations. Clock tickets bind the
current caller incarnation through a local grant lookup. `time::now` copies all 64 bits into a checked writable
buffer, avoiding confusion between large tick counts and negative syscall errors.
This is a boot-local wrapping counter on the nominal 100 Hz PIT, not wall time or
a calibrated elapsed-time guarantee. Grants gate this API; hardware timing sources
such as RDTSC are not confined, and virtualized clocks are not implemented.

`time::after` constructs an absolute deadline with an interval at most `2^63 - 1`
ticks. Comparisons interpret deadlines within the nearest half-cycle; exactly a
half-cycle is ambiguous and rejected. `Child::wait_until` requires clock authority
as well as the child ticket. If no reaped outcome is available at observation,
expiry returns `TIMED_OUT` without copying, consuming an outcome or killing the
child. An already-available outcome wins over expiry. Timer interrupts wake
parked waits, including when all userspace tasks are blocked.

`Child::cancel` gives only the owning supervisor authority over that exact child
incarnation. It returns after boot-context reclamation and endpoint teardown.
Blocked receivers observe `PEER_CLOSED`; the supervisor can collect `Cancelled`
and launch a replacement with fresh grants. Repeated cancellation before collection
is harmless, and cancelling an already-reaped child preserves its original outcome.
After collection the ticket is stale. Cancellation is abrupt task destruction,
not Omega cooperative cancellation, an IPC rollback or proof that prior work had
no effects. General service resource budgets remain absent.

The distribution watchdog fixture chooses the deadline and restart policy. It
alternates blocked and non-yielding services across eight recoveries. A separate
clock-authorized observer reports progress before cancellation, while the client
checks closure and reconnects. Further probes cover unauthorized and stale
cancellation, completed outcomes, bad clock destinations and an entirely blocked
session. Boot verifies heap and physical-frame baselines after each session.

## Userspace display experiment

The UEFI crate supplies GOP discovery, mode selection and the framebuffer snapshot
before `ExitBootServices`. Boot prefers 1024x768 RGB/BGR, retaining only physical
geometry after dropping all protocol references. The supported aperture must be
page-aligned, page-sized, at most 16 MiB, and large enough for the validated stride
and height. Bitmask and BLT-only modes are unsupported. Missing/unsupported GOP
prevents admission when the supplied startup grant requires a framebuffer, and
fails the composed display smoke test. A profile without that grant remains usable.
Any memory descriptor overlapping the aperture is conservatively excluded from
RAM allocation, even if firmware labels it conventional memory.

Boot may bind that aperture to exactly one supervised child's launch grant. No syscall
accepts an arbitrary physical address, and initial tasks receive no display grant.
The x86-64 backend maps it at `0x0000_0080_0800_0000`, writable, user-accessible and
non-executable, with unmapped adjacent pages. It requires PAT support and an
uncacheable entry at index 3, then uses PCD/PWT leaves. No write-combining setup or
hardware GPU driver is implemented; inherited firmware aliases remain unused.
The caller's unsafe custody obligations require the aperture to stay exclusive.

Device pages are borrowed, never zeroed or freed by the RAM allocator. Only page
tables enter the task's ownership ledger. Normal syscall buffer checks exclude
the device mapping. `display::mapping` returns six little-endian u64 words:
user address, bytes, width, height, stride, pixel format (0 RGB, 1 BGR). It reports
an already-installed mapping; it does not create, transfer or revoke one. Task
teardown retires the root and its translations before a replacement is admitted.
Framebuffer pixels persist after teardown until the next owner redraws them.

`platform/services/display` is a standalone ELF with no kernel imports. It owns
volatile pixel access and accepts a 48-byte experimental request: six little-endian
u64 words `(operation, x, y, width, height, color)`. Operations are info (0), clear
(1) and rectangle (2); colors are `0xRRGGBB`. Unused request fields must be zero.
Rectangles must be nonempty and wholly inside the visible area; malformed lengths,
overflows and unknown operations are rejected before drawing. Replies are six
words: status, then width/height/stride/format/zero for info, or five zeros for other
operations. Text (3) uses a 64-byte packet: six u64 words `(3, x, y, scale,
color, length)`, then up to 16 printable ASCII bytes. Scale is 1-4; length is
1-16. Each cell is 6-by-7 scaled pixels, including horizontal spacing. The entire
text rectangle must fit before any foreground pixels are written; background
pixels are unchanged. Longer labels are split by the distribution client. The
original font lives in `platform/libraries/bitmap-font` and has no device access.
The endpoint grants whole-screen drawing power to the trusted lab
client. This is not an ordinary application's future surface grant, a compositor,
a trusted prompt implementation, a capture protocol or a frozen Cathedral ABI.

`distribution/lab/display` chooses the colors/layout and restart policy.
Its supervisor launches the provider; a persistent client draws through IPC.
Boot-controlled fault injection crashes the first provider on request nine;
the client observes closure and reconnects to a fresh instance. The supervisor
cancels the replacement after a successful redraw. An independent clock-granted
task progresses across each drawing interval. Additional probes exercise missing
authority, NX, both guard pages, checked metadata copyout, rejection of device
memory as a syscall buffer, and two retries at every failed frame-admission budget.
All sessions return their heap and RAM-frame counts to baseline. The display
lab draws through the same distribution scene library.

## Ordinary userspace startup

`main.rs` initializes firmware custody, memory, exception entry, the heap and timer,
then enables frame reclamation. With `bundled-user` it admits a supplied initial
ELF through `startup.rs`; without it, the kernel remains idle. The normal kernel
path has no knowledge of the display service executable or distribution scene.
The bootstrap UART remains the documented privileged platform-driver exception.
`CATHEDRAL_RS_BOOT_OK` marks kernel initialization; `Cathedral desktop: ready;
choose a program` marks the default distribution's successful startup.

The host profile supplies `startup.initial` and an optional `startup.launches`
array (up to eleven, within twelve total task slots). Each selects a package, entry and ELF target; a child may
request a framebuffer, keyboard or disk grant and a fixed first argument. Initial and
child entries may request clock access with `clock: true`. Each device resource
has at most one owner. The kernel grants the initial task distinct launch and
connection tickets for each approved child. It enforces custody and task
lifetime; the initial program chooses when to spawn, connect and restart.
No physical address or executable choice is accepted from untrusted syscall data.
This host-selected authority is still lab composition, not signed manifest admission.

The default profile's initial program is the [desktop owner](distribution/desktop/main.rs).
It launches the display, input, root matrix and storage providers, funds the
root matrix with code grants and a process pool, and from then on only validates
menu choices, admits apps through the matrix and supervises storage. Links
connect the root matrix to the provider's two private roots; the arena is
allocated once at session admission and reclaimed at teardown, and message
operations allocate nothing. Apps receive only their local connections and
granted roots: no device, launch, cancellation or provider control. All tasks
still have the lab diagnostic-write syscall. The [distribution map](distribution/README.md)
follows the whole route; the [desktop guide](distribution/desktop/README.md)
covers its probe compositions and recovery policy.

A replacement initial program need not use the platform services. For example:

```text
python tools/boot-harness-rs/run.py --profile tools/boot-harness-rs/profiles/minimal.json
```

That example supplies only the existing hello ELF, which prints and exits. No
child or framebuffer grant is installed. Custom ordinary profiles have separate
output directories. `--kernel-only` goes further and omits all user artifacts.

All exhaustive kernel workloads live under `kernel/boot/uefi/lab/`, compiled only by
`smoke-test`. Its copied artifacts retain explicit service/fault composition as
test fixtures. The display service's separate `lab` feature likewise excludes
fault injection and negative probes from its normal executable.

## Keyboard and independent-service experiment

The q35 PS/2 controller is temporary bootstrap hardware, like PIC/PIT. Kernel
arch code transfers bytes through fixed ports and acknowledges IRQ1. Core checks
the exclusive child grant, uses a 64-record tagged byte queue and parks readers. IRQs and timer
polls drain at most 64 hardware bytes; controller replies can need timer delivery
while the keyboard interface is disabled. Kernel code never decodes scan codes.
The write interface allows only controller mode/interface commands, plus device
data; reset and A20 commands are rejected. It is not arbitrary port authority.

The platform input ELF selects scan set 2 with controller translation to set 1,
decodes thirteen physical navigation/function keys, and returns two-byte key/state
events for the `NEXT` request, or an explicit IDLE response after 25 ticks without
an event. Startup sends RESET once configuration completes.
The decoder recognizes press, release and repeat, skips Pause/PrintScreen, and
clears held/prefix state on overflow. Overflow discards queued partial sequences
and yields RESET; restarting a child discards old raw bytes and endpoint queues.
This is bounded and intentionally lossy, not a full keyboard/text stack. There is
no USB, hotplug, layout, IME, seat routing, focus or production input authority.
Unresponsive hardware now times out; repeated initialization failure ends startup.

The matrix desktop's fixtures drive physical keyboard and mouse events over
QMP and check exact scanout; provider loss, stalls and replacement are covered
by `--matrix-desktop-test` and `matrix_database_test.py --recovery`.

Smoke additionally exercises a full four-child/three-link graph across 16 app or
provider replacements, then checks complete heap and frame reclamation. IPC
readiness tests cover denied clock/rights, non-consuming observations, first-ready
precedence, closure and an all-blocked timeout. Existing tests retain deadline
copyout, late replies, raw keyboard grants, sibling queues and admission rollback.

## Bounded private pages and shared buffers

The [memory contract](kernel/contracts/memory.rs) defines explicit per-task private and
accepted-reader budgets: up to 1024 4 KiB pages of private memory, and up to four
times that in reader pages, which pin other tasks' backing rather than funding
frames. An allocation contains one to 256 zeroed pages. Independently, a
task can own at most eight objects and hold at most 48 reader rights. The
session slots cover those limits for all 31 tasks, including readers
retained after producer exit; a larger page grant cannot consume another task's
metadata capacity. An unmapped guard follows each process-local mapping slot
(eight private plus 48 reader slots). Mapping addresses never contain the global backing index. Runtime mappings are non-executable. Task admission reserves their
page tables; allocation and mapping changes execute on the boot stack with the
kernel root active. The kernel owns physical backing independently of either
participant's address space.

Follow [the policy model](kernel/core/user_tasks/memory/regions/regions.rs) for budgets and lifetime,
[the syscall entrance](kernel/core/user_tasks/syscalls/memory.rs) for
validation/deferral, [memory execution](kernel/core/user_tasks/memory/memory.rs)
for allocation/sharing/retirement dispatch, [sharing](kernel/core/user_tasks/memory/sharing.rs)
for producer sealing and reader acceptance, [retirement](kernel/core/user_tasks/memory/retirement.rs)
for alias removal before refund, and [shared address-space mappings](kernel/core/user_tasks/address_space/regions.rs)
for PTE and checked-copy permissions. None of these depends on the platform.

The producer fills private RW pages, seals them to a live boot-approved service
link, then sends the handle over copied IPC. Sealing permanently removes write
permission. The consumer supplies its service-side link to `Shared::accept_from`, which
authenticates the producer and installs a separate reader-local handle. Raw
`MEMORY_MAP` returns that handle; `MEMORY_ADDRESS` returns its local address.
The consumer reads RO pages and releases its mapping
before replying; the producer then releases its owner reference. An accepted
reader survives producer death and retains its shared-page charge. An unaccepted
offer is cancelled when the producer dies. Consumer death releases its lease;
replacement tasks cannot inherit old authority. Complete teardown reclaims all
backing and tables without relying on user destructors.

The [runtime API](platform/libraries/user-runtime/memory/memory.rs) expresses that flow
as `Private -> Sealed` and `Shared`. These buffers are not a general heap, an
arbitrary-address mapping API, a growable region, multicast, or executable memory.
A dropped producer token with a pending peer stays charged until task exit;
protocols must complete or recover the exchange. Non-reused local names reject stale
operations, but raw virtual addresses can be reused after release: the kernel
does not make dangling pointers safe.

The display lab supplies its checkerboard this way; the display provider
validates pixels in its read-only mapping and copies them to GOP. The bounded compositor experiment is described below; GPU and capture
protocols remain future platform work.

`--smoke` exercises quota exhaustion, zeroing/reuse, stale and foreign handles,
checked-copy rejection, write/NX/guard faults, producer and consumer replacement,
pinned-reader budgets, parent-exit cleanup and heap/frame baselines. A smoke-only
fixture injects every physical allocation failure boundary and checks rollback,
including all reserved runtime page tables. It also maps the largest buffer across
a leaf-table boundary and in the last slot, checking read-only aliases and guards.
Host tests cover independent page/object limits and global capacity with orphaned
readers. Recovery/storage builds additionally send malformed
pixel buffers to the real display service and require error replies to release
accepted leases. Screenshot/input/recovery/storage checks include the new tile.

This is an explicit lab contract. The transport and revocation question in
[OWNER_QUESTIONS.md](../OWNER_QUESTIONS.md) remains open.

The [interactive pressure experiment](distribution/lab/compositor/PRESSURE.md)
exhausts grants, leaves compositor replies unread, and repeatedly cancels/faults
an isolated peer while testing the visible session. Nonblocking user syscalls
retain the caller's quantum; timer preemption, explicit yield and blocking still
schedule peers. This avoids making every clock query or ready receive an implicit
yield to a CPU-bound task. Device IRQ readiness likewise does not force a rotation
between timer interrupts. This does not establish CPU reservations or real-time
latency guarantees.

## Retained compositor experiment

The [two-client compositor lab](distribution/lab/compositor/README.md) adds
connection-scoped retained drawing, nested groups and exposure redraw through a
separate composition profile. Run `python tools/boot-harness-rs/run.py --compositor-test`
from the repository root, or use its documented window command. The status demo
still uses the original trusted whole-screen provider path.

The compositor accepts bounded rectangles, ASCII text and one small shared image
per client. Kernel memory acceptance can authenticate the producer against a
service-side link. Generic service events expose client disconnects so retained
state cannot silently pass to a replacement. QEMU exercises cross-client denial,
invalid/stale snapshots, movement, failure containment and provider replacement;
the host verifies the final scanout. The optional [pointer lab](distribution/lab/compositor/POINTERS.md)
adds PS/2 motion, clipped hit-testing, capture, root-owned cursors and one scoped
synthetic pointer. Run `python tools/boot-harness-rs/run.py --pointer-test`; use its
documented window command to interact.

The [persistent session lab](distribution/lab/compositor/SESSION.md) adds deferred
event waits, explicit input regions, attenuated keyboard-focus grants and a child
app hosted inside another process's scope. Live compositor replacement preserves
all three app processes and their desired state while cancelling capture/focus.
Run `python tools/boot-harness-rs/run.py --session-test --timeout 60`; add `--release`
for the optimized build. Its guide lists the interactive command and failure
gestures. These remain bounded experiments without general Matrix delegation,
protected output, production seat semantics or GPU presentation.

## Next bring-up steps

- Define executable admission/provenance and service lifetime/adoption contracts.
- Add kernel-task arguments, join/result delivery and explicit ownership of task handles.
- Discover ACPI/APIC topology and replace the temporary PIC/PIT timer route.
- Specify production endpoint/shared-region delivery, acceptance and revocation semantics.

Keep source transitions and invariants recognizable beside their Omega owners.
Record deliberate divergences here and preserve test cases for eventual shared
conformance testing. Rust data validation is not proof of firmware custody;
QEMU success is not verification on physical hardware.

Runtime page and channel allowances can now be subdivided through owned idle
launch grants; see [resource subdivision](kernel/core/user_tasks/resources/README.md). Profiles
may grant `private_pages` and `shared_pages` to the initial program as well as
launches. Retained pages remain charged to their original account after producer death.

[Executable-authorized admission](kernel/core/user_tasks/admission/pool/README.md) separates
code grants from process capacity. `startup.dynamic_tasks` reserves unbound
accounts; apps can reserve and recursively launch granted executables within their
page/channel/process allowances. Run `--admission-test --release --timeout 120`.

[Physical-memory funding](kernel/core/user_tasks/resources/physical/README.md) now reserves
real frames before userspace admission. Image/stack/page-table and runtime-buffer
charges share conserved account funding, distinct from object quotas and the
per-load failure ceiling. `Launch::assign_memory` subdivides it into idle children;
`Executable::reserve_funded` includes it atomically with dynamic admission.

The recursive desktop persists each app's color through its matrix-local
object root. Ordinary `matrix-desktop.json --release --window` boots retain
`build/boot-harness-rs/matrix-database-storage.raw`, separate from the default desktop.
`--matrix-desktop-test` enables destructive probes on a disposable image and then
cold-boots that same image to check restoration. `--pressure` adds a ninth process
for a capped spinner while retaining storage and all recovery checks. See the
[matrix lab guide](distribution/desktop/README.md) for navigation and limitations.

The [private content publication lab](distribution/lab/objects/README.md) exercises
real BLAKE3 chunks/manifests through an explicitly granted client connection:

```powershell
python tools/boot-harness-rs/objects_test.py --release
python tools/boot-harness-rs/objects_test.py --retention --release
python tools/boot-harness-rs/matrix_database_test.py --release
python tools/boot-harness-rs/matrix_database_test.py --release --recovery
```

It uses a new disposable image, checks atomic refusals, kills QEMU after acknowledged
publication, cold-recovers contents and independently verifies every retained record.
The retention trace adds durable pins, bounded collection, sealed bulk reads and
reader-pressure checks across three boots. The matrix trace saves each desktop
app through its private matrix capability and cold-restores it from database
private objects. See the [object store guide](platform/libraries/database-objects/README.md)
for ownership and format limits, and the content guide for dev commands, heap
samples and optional ATA tracing. The intermittent ATA timeout remains undiagnosed.

F5's **Cathedral: Build & Run in QEMU** uses the database provider for users and
their programs in the default desktop. Each launch backs up
`build/boot-harness-rs/cathedral-storage.raw` and seeds a new disk. **Resume Saved
Lab** keeps the existing disk and installs changed programs. Explicit
lab profiles retain their separate images. The recovery proof adds a real startup
deadline, provider death before commit and after commit-before-reply, explicit
re-grants, live sibling interaction and per-stage timing. Apps retain confirmed
state and show a red mark on uncertain saves; Enter reconciles by reading before
another mutation. Bounded owner supervision now replaces dead/stalled providers
and retired root data streams, with at most three replacements per desktop session.

**Notes** in a user's chrome opens a scratchpad with physical text input,
atomic saves and draft-preserving reconciliation. See the [notes guide](distribution/applications/notes/README.md) for controls,
proofs, measured costs and the remaining bulk-content boundary.

## Scoped app bootstrap

Notes and Colors acquire services through [one scoped broker](platform/contracts/services/README.md).
The default desktop uses this path. Focused acquisition/teardown checks and ordinary
F5 tests use disposable media and separate firmware/ESP files:

```powershell
python tools/boot-harness-rs/matrix_composition_test.py
python tools/boot-harness-rs/desktop_test.py
python tools/boot-harness-rs/desktop_test.py --service-imports undeclared
python tools/boot-harness-rs/desktop_test.py --service-imports incompatible
```

The latter variants change only a disposable Notes import declaration. They check
that absent optional storage stays absent, dirty-close protection still works,
and the sibling Colors program persists its own state across cold reboot.

Producer acquisition uses disposable feature builds through the same desktop:

```powershell
python tools/boot-harness-rs/desktop_test.py --input-producer complete
python tools/boot-harness-rs/desktop_test.py --input-producer partial
python tools/boot-harness-rs/desktop_test.py --input-producer denied
python tools/boot-harness-rs/desktop_test.py --participants --trace-storage
python tools/boot-harness-rs/desktop_test.py --render-demand
python tools/boot-harness-rs/desktop_test.py --participant-fault hold
python tools/boot-harness-rs/desktop_test.py --participant-fault disconnect
python tools/boot-harness-rs/desktop_test.py --participant-fault pressure
```

These check the separate producer endpoint, interrupted handoff/capacity refusal,
and absent owner permission respectively, followed by normal app use and cold
persistence. The acquisition probes do not enable production in default Notes. The participants
workflow exercises the installed Split AUTO participant, independent focus, visible
cursor, repeated stop/start, resize revocation and cold persisted documents.
The participant fault workflows also query the scoped input binding directory
through the ordinary interaction pair. The receiver checks that delivered events
match its directory and that consumed retirement boundaries remove those entries;
the host requires successful snapshot evidence, AUTO descriptions and inherited
Pointer/Keyboard descriptions as well as saved text. No registry
access or injection permission is added to ordinary Notes.


### Nested desktop in the normal installation

`python tools/boot-harness-rs/desktop_test.py --nested-desktop --trace-storage`
uses a disposable image and the ordinary F5 composition. It creates an outer user,
enters three successive Nested desktops with separate users, and saves Notes at
the deepest level. It checks recursive dirty-close refusal, closes and reopens
the whole subtree twice, and verifies that outer Notes did not inherit its data.
It repeats the deepest reads and subtree reclamation after cold reboot. Deep
startup is still slow; this fixture allows 120 seconds for a nested scene.
See [the distribution route](distribution/README.md#nested-startup-and-scene-switching)
for code navigation and current funding limits. This test never opens the interactive
`cathedral-storage.raw`.

Add `--pointer-stress` to keep moving the absolute pointer during those scene
waits and for three seconds after each nested desktop becomes visible. It rejects
any matrix service exit immediately, then exercises the same Notes, recursive
close and cold-reopen checks. This reproduces input-poll starvation that ordinary
clicks separated by completed scenes did not catch.
