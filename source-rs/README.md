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
| `kernel/boot/uefi/users.rs` | Ring-3 syscall, isolation, fault containment and admission rollback experiments |
| `kernel/boot/uefi/applications.rs` | Run the bundled executable twice per session and check loading, private state, exit statuses and cleanup |
| `kernel/boot/uefi/ipc.rs` | Compose explicit endpoint grants and check service/failure sessions |
| `kernel/core/ipc.rs`, `ipc/` | Host-testable endpoint rights, bound tickets, queues and teardown |
| `kernel/boot/uefi/smoke.rs` | Test-only fault injection and QEMU result reporting |
| `contracts/boot.rs` | Firmware-neutral memory handoff; experimental Rust data, not a frozen ABI |
| `contracts/user.rs` | Shared experimental entry/syscall constants for kernel and user runtime |
| `kernel/core/extent.rs` | Bootstrap frame accounting and reclaiming bitmap over usable RAM, corresponding to the resource work in `source/kernel/core/` |
| `kernel/core/heap.rs`, `scheduler.rs`, `tasks.rs`, `tasks/` | IRQ-safe heap, pure scheduling policy, task admission and context/stack lifetime management |
| `kernel/core/users.rs`, `users/` | Experimental user-task lifetime, checked diagnostic syscalls and outcomes |
| `kernel/core/users/elf.rs` | Bounded, host-testable ELF64 preflight before physical admission |
| `platform/drivers/uart_16550/` | Polling serial diagnostics, corresponding to `source/platform/drivers/uart_16550/` |
| `platform/libraries/user-runtime/` | Entry stub, syscall wrappers and linker script; imports only shared contracts |
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
and both `x86_64-unknown-uefi` and `x86_64-unknown-none`; Rustup installs them
when Cargo is run inside `source-rs/`.
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
The harness reads the one distribution profile, builds its named `user_programs`
as ELFs, then supplies those artifacts to the UEFI build with `bundled-user`.
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
cargo clippy --locked --package cathedral-hello --package cathedral-ipc-lab --target x86_64-unknown-none -- -D warnings
```

The default Cargo members are the host-testable contracts and core. Kernel crates
use the UEFI target; the user program/runtime use the freestanding ELF target,
so the whole workspace cannot be built for one target. `cargo build-uefi` builds
the kernel/probe lab alone; use the Python harness for the composed distribution
image. Run Cargo inside this directory so its toolchain, aliases and relative
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
26. Print `CATHEDRAL_RS_BOOT_OK`, then idle or terminate the smoke-test guest.

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

The runtime supports a configurable limit of 1â€“64 trusted kernel tasks on one
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
The runtime has no dynamic linker, dynamic user spawn, admission proofs,
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

The kernel owns at most four anonymous, one-way endpoints per user session.
Each has one sender, one receiver, an optional revoke holder, and one queued
message of at most 64 bytes. Boot explicitly assigns these grants to task slots;
`IPC_HANDLE` enumerates only the caller's installed grants. There is no global
lookup, runtime endpoint creation, delegation, transfer, manifest check, lease,
persistent authority arena or production admission policy.

Tickets combine a session epoch, owner slot and grant slot. The trapping task's
kernel identity selects authority; user arguments cannot select another caller.
Epochs never wrap during a boot, and task slots are not reused within a user
session. Zero, foreign and retired-session tickets fail. Raw ticket bytes can
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
references into user memory. Mappings remain immutable throughout a session.
When no task is ready, the boot context halts between timer ticks; there are no
receive deadlines or deadlock recovery yet.

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

## Next bring-up steps

- Define executable admission/provenance and explicit user-task supervision.
- Add kernel-task arguments, join/result delivery and explicit ownership of task handles.
- Discover ACPI/APIC topology and replace the temporary PIC/PIT timer route.
- Specify endpoint delivery/revocation semantics, then prototype shared-region IPC.

Keep source transitions and invariants recognizable beside their Omega owners.
Record deliberate divergences here and preserve test cases for eventual shared
conformance testing. Rust data validation is not proof of firmware custody;
QEMU success is not verification on physical hardware.
