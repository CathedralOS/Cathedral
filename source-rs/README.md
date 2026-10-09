# Cathedral Rust lab

A bootable Rust implementation for working out Cathedral's OS mechanisms while
Omega develops. `source/` remains the Omega implementation; this workspace
mirrors its ownership layers and names as behavior is implemented. It is an
experimental kernel, with explicit unsafe hardware operations and no claim to
Omega's proof or authority guarantees.

## Layout

| Path | Responsibility / Omega counterpart |
| --- | --- |
| `kernel/boot/uefi/main.rs` | Visible orchestration of firmware entry and post-handoff kernel startup |
| `kernel/boot/uefi/firmware.rs` | UEFI crate adapter and memory-inventory policy |
| `kernel/boot/uefi/graphics.rs` | GOP snapshot before firmware handoff |
| `kernel/boot/uefi/startup.rs` | Admit a supplied initial program and its bounded launch grants |
| `kernel/boot/uefi/lab.rs`, `lab/` | Smoke-only orchestration, fixtures and display recovery verification |
| `kernel/boot/uefi/memory.rs`, `handoff.rs` | Compose core frame policy with architecture mappings, then transfer boot state |
| `kernel/boot/uefi/interrupts.rs`, `diagnostics.rs` | Interrupt bring-up and serial/fatal reporting |
| `kernel/boot/uefi/heap.rs` | Heap installation; smoke-only heap verification |
| `kernel/boot/uefi/lab/tasks.rs`, `lab/task_lifecycle.rs` | Cooperative/preemptive workloads and dynamic lifecycle checks |
| `kernel/boot/uefi/lab/users.rs` | Ring-3 syscall, isolation, fault containment and admission rollback experiments |
| `kernel/boot/uefi/lab/applications.rs` | Run the bundled executable twice per session and check loading, private state, exit statuses and cleanup |
| `kernel/boot/uefi/lab/ipc.rs` | Compose explicit endpoint grants and check service/failure sessions |
| `kernel/core/supervision.rs`, `supervision/` | Host-testable launch authority, child identity and outcome consumption |
| `kernel/core/deadline.rs` | Boot-issued clock grants and wrapping deadline comparisons |
| `kernel/boot/uefi/lab/supervision.rs` | Supply launch bounds and verify userspace recovery/cleanup |
| `kernel/boot/uefi/lab/watchdog.rs` | Verify hung-service cancellation, deadline wakeups and independent progress |
| `kernel/core/ipc.rs`, `ipc/` | Host-testable endpoint rights, bound tickets, queues and teardown |
| `kernel/boot/uefi/smoke.rs` | Test-only fault injection and QEMU result reporting |
| `contracts/boot.rs` | Firmware-neutral memory handoff; experimental Rust data, not a frozen ABI |
| `contracts/user.rs` | Shared experimental entry/syscall constants for kernel and user runtime |
| `contracts/display.rs` | Bounded framebuffer geometry and experimental drawing messages |
| `contracts/input.rs` | Experimental physical-key events, independent of scan codes or UI policy |
| `kernel/core/extent.rs` | Bootstrap frame accounting and reclaiming bitmap over usable RAM, corresponding to the resource work in `source/kernel/core/` |
| `kernel/core/heap.rs`, `scheduler.rs`, `tasks.rs`, `tasks/` | IRQ-safe heap, pure scheduling policy, task admission and context/stack lifetime management |
| `kernel/core/users.rs`, `users/` | Experimental user-task lifetime, checked diagnostic syscalls and outcomes |
| `kernel/core/users/elf.rs` | Bounded, host-testable ELF64 preflight before physical admission |
| `platform/drivers/uart_16550/` | Polling serial diagnostics, corresponding to `source/platform/drivers/uart_16550/` |
| `platform/libraries/user-runtime/` | Entry stub, syscall wrappers and linker script; imports only shared contracts |
| `platform/services/display/` | Separately compiled userspace provider for linear framebuffer drawing |
| `platform/services/input/` | Userspace PS/2 configuration, key decoding and IPC event delivery |
| `distribution/init/`, `distribution/libraries/boot-scene/` | Ordinary userspace startup, restart policy and scene layout |
| `distribution/programs/display/` | Test-pattern layout, display client and restart policy |
| `distribution/programs/supervision/` | Userspace restart policy, persistent client and crashing echo service |
| `distribution/programs/ipc/` | Client, echo service and hostile IPC fixture roles in a standalone ELF |
| `distribution/programs/hello/` | Independently compiled `no_std` program exercising initialized data, BSS, yields and writes |
| `kernel/arch/lib.rs` | Compile-time CPU backend selection and the boot-facing machine interface |
| `kernel/arch/x86/` | Shared instructions and the selected PC platform's temporary PIC/PIT route |
| `kernel/arch/x86_64/` | Paging, dynamic guarded stack mapping/teardown, CPU contexts, GDT/TSS/IDT and interrupt stubs, using the `x86_64` crate |
| `kernel/arch/x86_64/user.rs`, `user/` | Sparse task address spaces, CR3 entry/return policy and embedded x86-64 test payload |
| `distribution/profile.json` | Built-in distribution composition consumed by the boot harness |
| `../tools/boot-harness-rs/` | Host build, QEMU launch and smoke verification |

Each crate uses `no_std`. Core policies are host-testable and have no firmware
dependency. Core's hardware-facing modules explicitly opt into unsafe code and
depend on `kernel/arch/`; drivers never depend on core internals. Boot assembles these
subsystems and the firmware adapter.
The bootstrap UART currently runs privileged; the display and input providers run in userspace. Boot
supplies its port operations; the platform driver imports no kernel package.
`python tools/source-layout/check.py` checks these boundaries from the repository
root. The single `distribution/` is replaceable by forks; platform and kernel do
not import it.
The Rust-specific `kernel/arch/` layer groups hardware mechanisms that Omega currently
spreads across core providers, instruction contracts and libraries. Only x86-64
boots today; shared x86 instructions do not imply a working 32-bit kernel.
UEFI ABI definitions and Boot Services come from the upstream `uefi` crate.
`foundation/` and distribution desktop packages appear when their first code lands,
following the same rule as the Omega tree.

## Run

Prerequisites: Rustup, Python 3.10+, QEMU and OVMF. The workspace pins Rust 1.94.0
and both `x86_64-unknown-uefi` and `x86_64-unknown-none`; Rustup installs them
when Cargo is run inside `source-rs/`.
Dependencies are locked in `Cargo.lock`. No nightly language features are used.

From the repository root:

```text
python tools/boot-harness-rs/run.py --build-only
python tools/boot-harness-rs/run.py --smoke
python tools/boot-harness-rs/run.py --kernel-only --smoke
python tools/boot-harness-rs/run.py --screenshot
python tools/boot-harness-rs/run.py --input-test
python tools/boot-harness-rs/run.py --recovery-test
python tools/boot-harness-rs/run.py --window
python tools/boot-harness-rs/run.py
```

The final command keeps init, display and input alive; all block when
there is no work, leaving the CPU idle with timer wakeups. Ctrl+C stops QEMU.
`--window` shows the display during an ordinary boot. `--screenshot` runs a
bounded ordinary boot, captures QEMU's actual scanout through local QMP, checks
every pixel against the distribution pattern, confirms no lab suite ran, writes `capture/display.png`
under the build directory, and stops QEMU. This uses only Python's standard
library. The capture check requires the lab's 1024x768 mode.
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
Both builds use the selected debug/release profile. There is no filesystem read
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
cargo fmt --all -- --check
cargo check-uefi
cargo build-uefi
cargo build --locked --package cathedral-hello --target x86_64-unknown-none
cargo clippy --locked --package cathedral-hello --package cathedral-ipc-lab --package cathedral-supervision-lab --package cathedral-display-service --package cathedral-input-service --package cathedral-display-lab --package cathedral-init --target x86_64-unknown-none -- -D warnings
```

The default Cargo members test contracts, core and the input decoder on the host. Kernel crates
use the UEFI target; the user program/runtime use the freestanding ELF target,
so the whole workspace cannot be built for one target. `cargo build-uefi` builds
a standalone kernel with no initial program; use the Python harness for the
composed distribution image or add `smoke-test` for kernel-only exercises. Run Cargo inside this directory so its toolchain, aliases and relative
linker-script path apply. The bare-metal target uses the SysV calling convention;
the runtime's entry stub establishes its call alignment before invoking Rust.

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
   and 64 KiB of heap backing. Reserve a separate virtual branch for dynamic task
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
    and a guarded 16-KiB stack. Switch back to the kernel root on entry.
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
    foreign tickets and retired-session handles; require actual blocking.
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
30. Grant a reserved GOP framebuffer to an isolated display provider. Draw a
    distribution-owned pattern, fault/restart the provider, reconnect and redraw
    while an independent observer progresses. Reject ungranted access and check
    NX, guard pages, copy boundaries and every admission allocation failure.
28. Print `CATHEDRAL_RS_BOOT_OK`, then idle or terminate the smoke-test guest.

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
Fatal and NMI handlers must never allocate. There is no isolated driver yet.
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

The runtime supports a configurable limit of 1ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â¦ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦ÃƒÂ¢Ã¢â€šÂ¬Ã…â€œ64 trusted kernel tasks on one
CPU (default 8), plus an optional frame budget for stacks and their tables.
`tasks::spawn(fn())` returns a `TaskId` or an admission error. IDs contain slot
generations and are scoped to a session; reused slots do not revive old IDs.
`is_alive` reports liveness; return is the supported exit mechanism. There are
no joins, captured closures, cancellation or recovery from task panic yet.
These trusted kernel tasks share an address space; guards protect against stack
overrun, not against malicious tasks. There is no capability enforcement or SMP. The
64-KiB heap and runtime arena are initialized before executing tasks; ordinary
infallible Rust allocations can still panic on exhaustion. Task context
admission is fallible and returns `OutOfHeap` without leaking stack frames.
The scheduler is a pure
round-robin state machine; IRQ masking protects its mutable session, and heap
locks are never held across a voluntary suspension. Preemption is tested with
tasks that never yield, including a negative control that fails when their
timer preemption is disabled.

## User-mode experiment

User sessions run 1-8 tasks from either the one-page assembly probes or restricted
static ELF images. Each task has its own root and physical backing
at the same user virtual addresses, with no inherited firmware identity map.
Only the live kernel image, boot/emergency stacks and context-storage heap are
retained supervisor-only for entry/return. The kernel root still has its original
identity aliases. Global translations are disabled before user execution; CR3
switches flush task translations. This is architectural memory/privilege
containment on qemu64, not a claim of speculative-execution mitigation or full
kernel W^X. Hardware rules follow the
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
| 17: keyboard read | 0: nonblocking, 1: wait | Raw byte, 256 on data loss, `WOULD_BLOCK` or `DENIED` |
| 18: keyboard write | 0: data or 1: command, byte | 0, `WOULD_BLOCK`, `INVALID_ARGUMENT` or `DENIED` |
| 19: IPC receive until | Ticket, 64-byte destination, absolute deadline | Bytes copied or error; requires clock access |
| 20: keyboard read until | Absolute deadline | Raw byte/loss or `TIMED_OUT`; requires keyboard and clock access |

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
device grants or production resource policy. Kernel and user workloads currently
run in separate boot-managed sessions.

## Executable loading experiment

The [ELF preflight](kernel/core/users/elf.rs) accepts little-endian ELF64,
System V ABI version 0, x86-64 `ET_EXEC`. Limits are 4 MiB per file, 16 program
headers, eight load segments and 64 image pages total. Each load segment must be
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
with its status; a Rust panic exits with 255. There is no allocator or unwinding.
The [hello program](distribution/programs/hello/main.rs) contains no privileged
instructions or kernel imports. It checks more than one page of BSS and distinct
per-instance data, then returns the supplied status. Its link script explicitly
places LLVM's normal and large-code-model sections into matching segments.

The [kernel design](../wiki/design/part_5_lifecycle/04_kernel_architecture.md)
already calls for hardware walls around unproved apps and drivers. The default
for proved OS components remains an owner-level policy question in
[`OWNER_QUESTIONS.md`](../OWNER_QUESTIONS.md); these experiments do not freeze
Cathedral's component model or shared ABI.

## Capability IPC experiment

The kernel owns at most six anonymous, one-way endpoints per user session.
Each has one sender, one receiver, an optional revoke holder, and one queued
message of at most 64 bytes. Boot explicitly assigns these grants to task slots;
`IPC_HANDLE` enumerates only the caller's installed grants. There is no global
lookup, general endpoint creation, delegation, transfer, manifest check, lease,
persistent authority arena or production admission policy.

Tickets combine an endpoint epoch, owner slot and grant slot. The trapping task's
kernel identity selects authority; user arguments cannot select another caller.
Epochs never wrap during a boot. Static endpoints retain their session epoch;
a supervised child's endpoint pair receives a fresh epoch on each spawn, so
reusing its task slot never revives old tickets. Zero, foreign and retired-session
tickets fail. Raw ticket bytes can
be sent as ordinary data but confer no authority on the recipient. Tickets are
not secret. They are boot-local; there is no cross-reboot stale-ticket guarantee.

Send copies bytes into the kernel queue and returns `WOULD_BLOCK` (-11) if full.
Receive validates the complete destination, including write permission, before
consuming a message or parking. The raw ABI requires a buffer inside one user
page; platform `ipc::Handle` wrappers stage through a 64-byte-aligned buffer so
ordinary Rust slices may cross page boundaries. Empty messages are valid; even
zero-capacity raw buffers require a valid writable address. A short destination
returns `TOO_SMALL` (-90) and retains the queued message. Error precedence is
handle/rights, length, address, then queue state.

A receive on an empty live endpoint parks the task; sending wakes it and writes
the result into its saved context. No callback allocates, frees or follows an
unchecked virtual pointer. Pending operations store integer ranges, not Rust
references into user memory. Each live task's mappings remain immutable until retirement.
When no task is ready, the boot context halts between timer ticks; there is no
general deadlock recovery. Deadline receives are described below.

Sender exit/fault allows already-accepted bytes to drain, then returns
`PEER_CLOSED` (-32), waking a blocked receiver. Receiver exit discards queued
bytes and subsequent sends fail closed. Revocation requires a separate right:
it discards queued bytes, cancels pending receives with `REVOKED` (-125), and
rejects later use. Wrong rights return `DENIED` (-13); invalid tickets return
`BAD_HANDLE` (-9). Teardown retires the task's grants before reclaiming its
address space; session destruction releases endpoint storage. Admission and
reclamation have separate stack frames to keep debug construction temporaries
within the existing 64-KiB boot stack.

The distro fixture runs one ELF in separate client/service roles, tests stolen
handle bits and a stale ticket from a prior session, and injects peer failures.
Boot asserts actual blocking before reporting wakeup success. Host tests cover
queue retention, admission limits, rights, epoch/caller checks and teardown;
QEMU checks real syscall copy boundaries and returns to memory baselines.

Copied queues and cancellation of already-parked receives deliberately remain
experimental. The shared-region IPC design and its relationship to the
capability lifecycle need reconciliation, recorded in
[`OWNER_QUESTIONS.md`](../OWNER_QUESTIONS.md).

### Deadline receives

`Handle::receive_until` requires explicit clock authority and stages a fixed
64-byte destination. It validates rights and writable memory before checking
expiry. An available message, revocation or peer closure wins over expiry at
observation. An empty expired wait returns `TIMED_OUT` without changing its
buffer. Timer interrupts also retire parked receives when every task is blocked.

Timeout removes only that receive. It neither revokes the endpoint nor cancels
a remote operation; a late reply can still queue. A caller must correlate/drain
late responses or replace the connection. Init takes the latter approach by
reclaiming the provider and accepting fresh epoch-bound endpoints before retry.
`keyboard::read_until` follows the same ready-before-timeout rule and additionally
requires the exclusive device grant. An input idle reply means the provider
completed a timed raw read; it is not evidence of a hung provider.

## Userspace supervision experiment

A session may reserve up to three launch grants within the eight-task and
six-endpoint limits. Each reserves one child slot and two endpoint slots for a
boot-approved executable. Boot binds each launch grant to one supervisor and a
connection port to one initial task, which may be the supervisor itself. The grant selects the executable,
fixed first entry argument, peer, and physical-frame budget; userspace can
supply only the second ordinary argument. It cannot nominate another image,
principal, authority set or peer. Initial endpoints cannot refer to the reusable
child slot. This is an authority-bound lab mechanism, not a general process API,
a manifest/provenance check or a component registry.

`task::Launch::spawn` parks its caller and hands admission to the boot context.
All context/metadata slots are preallocated; runtime spawn maps private physical
pages without growing the arena. Success returns a fresh owner-bound child
ticket. A second child on the same grant or an uncollected previous outcome
returns `BUSY` (-16). `Launch::at(index)` and `Port::at(index)` select independent
caller-local grants; `bootstrap()` selects index zero.
Failed ELF admission returns `BAD_EXECUTABLE` (-8); failed frame admission
returns `NO_MEMORY` (-12), rolls back, and permits another attempt. Epoch
exhaustion also fails closed. No mapping or reclamation runs in a trap callback.

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
its full exit status or kernel-observed fault details. The wire record is five
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
without user destructors or a graceful drain. No orphan adoption, general kill,
nested supervision, delegation or persistent recovery is implemented.
Component supervision versus task-scope ownership remains an explicit
question in [`OWNER_QUESTIONS.md`](../OWNER_QUESTIONS.md).

### Deadline waits and hung-service recovery

Boot explicitly grants clock access to selected task slots through
`Config::clock_readers`. Reserved children inherit none, but boot may separately
grant a reserved launch slot clock access across its incarnations. Clock tickets bind the
caller and session epoch. `time::now` copies all 64 bits into a checked writable
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
The x86-64 backend maps it at `0x0000_0080_0100_0000`, writable, user-accessible and
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
operations. The endpoint grants whole-screen drawing power to the trusted lab
client. This is not an ordinary application's future surface grant, a compositor,
a trusted prompt implementation, a capture protocol or a frozen Cathedral ABI.

`distribution/programs/display` chooses the colors/layout and restart policy.
Its supervisor launches the provider; a persistent client draws through IPC.
Boot-controlled fault injection crashes the first provider on request nine;
the client observes closure and reconnects to a fresh instance. The supervisor
cancels the replacement after a successful redraw. An independent clock-granted
task progresses across each drawing interval. Additional probes exercise missing
authority, NX, both guard pages, checked metadata copyout, rejection of device
memory as a syscall buffer, and two retries at every failed frame-admission budget.
All sessions return their heap and RAM-frame counts to baseline. The ordinary
init uses the same distribution scene library; its live scanout is
independently checked by the harness's `--screenshot` mode.

## Ordinary userspace startup

`main.rs` initializes firmware custody, memory, exception entry, the heap and timer,
then enables frame reclamation. With `bundled-user` it admits a supplied initial
ELF through `startup.rs`; without it, the kernel remains idle. The normal kernel
path has no knowledge of the display service executable or distribution scene.
The bootstrap UART remains the documented privileged platform-driver exception.
`CATHEDRAL_RS_BOOT_OK` marks kernel initialization; `Cathedral: startup ready`
marks the default distribution's successful drawing and service startup.

The host profile supplies `startup.initial` and an optional `startup.launches`
array (at most three). Each selects a package, entry and ELF target; a child may
request a framebuffer or keyboard grant and a fixed first argument. Initial and
child entries may request clock access with `clock: true`. Each device resource
has at most one owner. The kernel grants the initial task distinct launch and
connection tickets for each approved child. It enforces custody and task
lifetime; the initial program chooses when to spawn, connect, draw and restart.
No physical address or executable choice is accepted from untrusted syscall data.
This host-selected authority is still lab composition, not signed manifest admission.

`distribution/init` launches independent display and input providers. It owns the
scene selection and toggle state; the providers own their devices. Arrow keys
move the white selection border, Enter toggles a dark stripe in the selected
panel, F1 restarts input and F2 restarts display. State survives either restart.
Init redraws through `distribution/libraries/boot-scene` and waits for key events
over IPC. Input blocks on raw bytes with a 25-tick deadline and returns IDLE if
there was no event; display blocks on drawing requests. Init checks display INFO
on idle replies, so a display fault does not require another key to be noticed.
These manual shortcuts require functioning input/init and are not a trusted
recovery path. No intentional faults or exhaustive tests run during ordinary startup.

Each initialization reply, input request and whole scene redraw has a 100-tick
response budget (nominally one second). Controller replies get 50 ticks. Input
startup and failed requests each allow three attempts; request recovery can run
that bounded startup sequence. A detected timeout, closure or malformed reply
cancels/reclaims the affected child, collects its outcome, and accepts a fresh
connection before retrying. Partial drawing is repaired by clearing/redrawing
the scene from init's state. The sibling's task and endpoint grants stay live.
Repeated failure ends init and the kernel reclaims its children. There is no
backoff, persistent state or general service dependency manager yet.

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
the exclusive child grant, uses a 64-byte queue and parks readers. IRQs and timer
polls drain at most 64 hardware bytes; controller replies can need timer delivery
while the keyboard interface is disabled. Kernel code never decodes scan codes.
The write interface allows only controller mode/interface commands, plus device
data; reset and A20 commands are rejected. It is not arbitrary port authority.

The platform input ELF selects scan set 2 with controller translation to set 1,
decodes seven physical navigation/function keys, and returns two-byte key/state
events for the `NEXT` request, or an explicit IDLE response after 25 ticks without
an event. Startup sends RESET once configuration completes.
The decoder recognizes press, release and repeat, skips Pause/PrintScreen, and
clears held/prefix state on overflow. Overflow discards queued partial sequences
and yields RESET; restarting a child discards old raw bytes and endpoint queues.
This is bounded and intentionally lossy, not a full keyboard/text stack. There is
no USB, hotplug, layout, IME, seat routing, focus or production input authority.
Unresponsive hardware now times out; repeated initialization failure ends startup.

`--input-test` injects actual QEMU keyboard events through local QMP, validates
every scanout pixel after navigation/toggling and ten independent provider
restarts, then saves `input-test/display.png` and stops. The smoke suite separately
checks 16 sibling restarts with a reply queued on the surviving connection,
stale tickets, failed admission beside a live child, denied raw access, forbidden
controller commands, parent-exit cancellation of a blocked keyboard reader, and
return to frame/heap baselines. Host tests cover byte-queue loss, key decoding,
wire validation and sibling endpoint indexing.

`--recovery-test` uses separate `recovery-lab` features on init and both providers.
Their first incarnations spin during startup; replacement generations initialize
normally. The harness then injects crashes, syscall-free spins and blocked
request loops into each provider. It checks sibling task identity, queued input
during display failure, retained selection/toggles, healthy idle without restart,
and every scanout pixel after recovery. Test F1/F2 request faults rather than
manual restarts; normal builds contain none of those fault handlers. Captures
and logs live under `build/boot-harness-rs/recovery-test/`.

Smoke tests additionally verify deadline authority, invalid copyout without
message loss, late replies after timeout, ready/terminal precedence, all-blocked
wakeups, granted/ungranted child clock access and memory reclamation.

## Next bring-up steps

- Define executable admission/provenance and service lifetime/adoption contracts.
- Add kernel-task arguments, join/result delivery and explicit ownership of task handles.
- Discover ACPI/APIC topology and replace the temporary PIC/PIT timer route.
- Specify endpoint delivery/revocation semantics, then prototype shared-region IPC.

Keep source transitions and invariants recognizable beside their Omega owners.
Record deliberate divergences here and preserve test cases for eventual shared
conformance testing. Rust data validation is not proof of firmware custody;
QEMU success is not verification on physical hardware.
