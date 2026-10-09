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
| `kernel/boot/uefi/memory.rs`, `handoff.rs` | Compose core frame policy with architecture mappings, then transfer boot state |
| `kernel/boot/uefi/interrupts.rs`, `diagnostics.rs` | Interrupt bring-up and serial/fatal reporting |
| `kernel/boot/uefi/heap.rs`, `tasks.rs`, `task_lifecycle.rs` | Heap installation, cooperative/preemptive workloads and dynamic lifecycle checks |
| `kernel/boot/uefi/smoke.rs` | Test-only fault injection and QEMU result reporting |
| `contracts/boot.rs` | Firmware-neutral memory handoff; experimental Rust data, not a frozen ABI |
| `kernel/core/extent.rs` | Bootstrap frame accounting and reclaiming bitmap over usable RAM, corresponding to the resource work in `source/kernel/core/` |
| `kernel/core/heap.rs`, `scheduler.rs`, `tasks.rs`, `tasks/` | IRQ-safe heap, pure scheduling policy, task admission and context/stack lifetime management |
| `platform/drivers/uart_16550/` | Polling serial diagnostics, corresponding to `source/platform/drivers/uart_16550/` |
| `kernel/arch/lib.rs` | Compile-time CPU backend selection and the boot-facing machine interface |
| `kernel/arch/x86/` | Shared instructions and the selected PC platform's temporary PIC/PIT route |
| `kernel/arch/x86_64/` | Paging, dynamic guarded stack mapping/teardown, CPU contexts, GDT/TSS/IDT and interrupt stubs, using the `x86_64` crate |
| `distribution/profile.json` | Built-in distribution composition consumed by the boot harness |
| `../tools/boot-harness-rs/` | Host build, QEMU launch and smoke verification |

Each crate uses `no_std`. Core policies are host-testable and have no firmware
dependency. Core's hardware-facing modules explicitly opt into unsafe code and
depend on `kernel/arch/`; drivers never depend on core internals. Boot assembles these
subsystems and the firmware adapter.
The bootstrap UART currently runs privileged; user-mode drivers come later. Boot
supplies its port operations; the platform driver imports no kernel package.
`python tools/source-layout/check.py` checks these boundaries from the repository
root. The single `distribution/` is replaceable by forks; platform and kernel do
not import it.
The Rust-specific `kernel/arch/` layer groups hardware mechanisms that Omega currently
spreads across core providers, instruction contracts and libraries. Only x86-64
boots today; shared x86 instructions do not imply a working 32-bit kernel.
UEFI ABI definitions and Boot Services come from the upstream `uefi` crate.
`foundation/`, `platform/services/` and distribution desktop packages appear when their first code lands,
following the same rule as the Omega tree.

## Run

Prerequisites: Rustup, Python 3.10+, QEMU and OVMF. The workspace pins Rust 1.94.0
and the UEFI target; Rustup installs them when Cargo is run inside `source-rs/`.
Dependencies are locked in `Cargo.lock`. No nightly language features are used.

From the repository root:

```text
python tools/boot-harness-rs/run.py --build-only
python tools/boot-harness-rs/run.py --smoke
python tools/boot-harness-rs/run.py
```

The final command leaves the CPU idling with timer wakeups; Ctrl+C stops QEMU.
`--release` selects an optimized build. `--memory 256` selects guest RAM in MiB.
Smoke mode has a 30-second boot deadline, requires ordered serial milestones,
and checks the QEMU debug-exit status. A panic, missing milestone, reset or hang
fails the run. Its debug-exit feature is not enabled for ordinary boots.

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
```

The default Cargo members are the host-testable contracts and core. The whole
workspace requires the explicit `x86_64-unknown-uefi` target. Run Cargo inside
this directory so its toolchain and aliases apply.

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
16. Print `CATHEDRAL_RS_BOOT_OK`, then idle or terminate the smoke-test guest.

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
Fatal and NMI handlers must never allocate. There is no syscall path or isolated
driver. Timer and yield entries save all GPRs, the return frame, and x87/MMX/SSE
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

The runtime supports a configurable limit of 1–64 trusted kernel tasks on one
CPU (default 8), plus an optional frame budget for stacks and their tables.
`tasks::spawn(fn())` returns a `TaskId` or an admission error. IDs contain slot
generations and are scoped to a session; reused slots do not revive old IDs.
`is_alive` reports liveness; return is the supported exit mechanism. There are
no joins, captured closures, cancellation or recovery from task panic yet.
Tasks share an address space; guards protect against stack overrun, not against
malicious tasks. There is no user mode, capability enforcement or SMP. The
64-KiB heap and runtime arena are initialized before executing tasks; ordinary
infallible Rust allocations can still panic on exhaustion. Task context
admission is fallible and returns `OutOfHeap` without leaking stack frames.
The scheduler is a pure
round-robin state machine; IRQ masking protects its mutable session, and heap
locks are never held across a voluntary suspension. Preemption is tested with
tasks that never yield, including a negative control that fails when their
timer preemption is disabled.

## Next bring-up steps

- Add user mode, separate address spaces and a minimal syscall boundary.
- Add task arguments, join/result delivery and explicit ownership of task handles.
- Discover ACPI/APIC topology and replace the temporary PIC/PIT timer route.
- Add capability checks and shared-memory IPC.

Keep source transitions and invariants recognizable beside their Omega owners.
Record deliberate divergences here and preserve test cases for eventual shared
conformance testing. Rust data validation is not proof of firmware custody;
QEMU success is not verification on physical hardware.
