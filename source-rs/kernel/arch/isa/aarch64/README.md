# ARM64 portability proof

This backend runs on one QEMU `virt` Cortex-A57 with GICv2, 128 MiB RAM and
EDK2's AArch64 UEFI firmware. It proves that firmware handoff, frame/heap policy,
ELF admission, scheduling, IPC, supervision and page-object lifetime can share
the existing core. It is not an ARM desktop or a general ARM board port.

From the repository root:

```text
python tools/boot-harness-rs/arm64/run.py --release
python tools/boot-harness-rs/arm64/run.py
python tools/boot-harness-rs/arm64/run.py --kernel-only
```

The runner builds the standalone [guest fixture](../../../../distribution/lab/portability/main.rs),
then embeds its ELF through the existing UEFI artifact boundary. `--kernel-only`
runs timer/heap bring-up and the shared trusted-kernel-task suite; `--build-only` prepares the EFI image without
launching QEMU. It uses `BOOTAA64.EFI`, a fresh private firmware variable store,
ordered serial milestones and semihosted exit status. Logs and the ESP are in
`build/arm64/`. Pass `--qemu` to select an executable and `--firmware` for a
directory containing `edk2-aarch64-code.fd` and `edk2-arm-vars.fd`.

## Reading and execution paths

Start at [UEFI main](../../../boot/uefi/main.rs), which is shared with x86-64.
Its console selects PL011, while the upstream `uefi` crate continues to own the
firmware ABI and ExitBootServices transaction.

| Entrance | Next responsibility |
| --- | --- |
| [aarch64.rs](aarch64.rs) | Interrupt masking, stack handoff and terminal CPU operations |
| [memory/memory.rs](memory/memory.rs) | Construct and activate kernel tables, guarded stack, exception stacks and heap; `tables.rs` walks 4-KiB translations |
| [interrupts/interrupts.rs](interrupts/interrupts.rs) | Install vectors and fault reporting; `timer.rs` owns the 100-Hz virtual timer and calls the [board GICv2 adapter](../../board/virt/gic.rs) |
| [context/context.rs](context/context.rs) | Classify exceptions, enter the shared scheduler/syscall dispatcher, select the return root; `state.rs` and `entry.S` own saved registers |
| [stacks.rs](stacks.rs) | Map guarded EL1 stacks, roll back partial construction, detach empty tables and flush before refund |
| [user/user.rs](user/user.rs) | Select the task root for exception return; [mappings.rs](user/mappings.rs) encodes inactive user translations |
| [Shared address-space owner](../../../core/user_tasks/address_space/address_space.rs) | Image loading, frame ledger and rollback, checked copies, sealing and region bookkeeping |
| [QEMU virt board](../../board/virt/virt.rs) | Physical address map, PL011 console, GICv2 routing and available devices |
| [ARM64 lab entrance](../../../boot/uefi/lab/arm64/arm64.rs) | Sequence actual process, IPC, isolation, sharing and reclamation assertions |

An EL0 `svc #0` saves all general registers and FP/SIMD state, restores the
kernel translation root, and invokes the same
[core trap dispatcher](../../../core/user_tasks/execution/traps.rs) as x86-64.
The selected context returns through `eret`. x8 carries the operation,
x0/x1/x2 its arguments, and x0 its result. Application entry receives x0/x1.
The runtime changes only this CPU stub; callers keep the same capability APIs.

### Stack-safe entry and terminal faults

[Entry assembly](context/entry.S) saves the 800-byte context at the top of the
permanent interrupt stack before calling Rust below its reserved 1-KiB area.
It never stores onto the interrupted kernel stack. `TPIDR_EL1` is reserved as
temporary single-CPU entry scratch to preserve x0 while moving x1 and the original
SP to that frame; it is cleared before dispatch and is not a kernel TLS register.
EL0's original stack still comes from `SP_EL0`. The
[Arm exception model](https://developer.arm.com/-/media/Arm%20Developer%20Community/PDF/Learn%20the%20Architecture/Exception%20model.pdf)
describes the hardware exception registers and stack selection; the independent
interrupt/fault-stack policy here is software-owned.

Fatal EL1 synchronous exceptions bypass the normal save/dispatch path. Assembly
captures ESR/FAR/ELR and the original SP, switches to a separate permanent fault
stack, then calls the terminal reporter. This also works when the interrupted
stack was the IRQ stack itself. Returning EL1 BRK/SVC and timer IRQs retain the
normal path; EL0 faults still reach core for process containment. Faults during
terminal reporting park without recursively restarting the reporter.

The profile remains one EL1h kernel CPU, with masked interrupts during dispatch
and no nested resumable callbacks. Stack exhaustion terminates the kernel; it
does not recover a kernel task or promise diagnostics after arbitrary corruption
of the emergency stack, supervisor mappings or UART.

```text
python tools/boot-harness-rs/arm64/run.py --release --kernel-only --fault stack-exhaustion
python tools/boot-harness-rs/arm64/run.py --release --kernel-only --fault irq-stack-exhaustion
python tools/boot-harness-rs/arm64/run.py --release --kernel-only --fault stack-entry
```

The first two probes require the exact faulting store, guard address, original
exhausted SP and reporting on the dedicated fault stack. The last leaves only
16 bytes of stack headroom and checks return from BRK, SVC and a real timer IRQ,
including preservation of x0/x1 and the original stack. They run before ordinary
task admission and are separate from the healthy-stack guard-access probe.

## Evidence and bounds

The debug and release portability suites exercise:

- Firmware exit, owned tables/stack, breakpoint return, heap exhaustion/reuse,
  timer delivery on the exception stack.
- Trusted EL1 tasks yielding, sleeping and preempting with GPR/FP/SIMD/FPCR/FPSR
  sentinels; dynamic spawn, stale IDs, frame/heap exhaustion and live-peer progress.
- Userspace-session startup under progressively released heap pressure, with
  recoverable refusal and exact heap/frame reclamation before any task runs.
- Two private EL0 peers switched by the timer, with different page contents
  and SIMD register sentinels surviving switches.
- Repeated 32-message IPC exchanges, blocking receives, local rights, stale names.
- Rejection of kernel heap, physical page tables and UART access; read-only code,
  stack guards and non-executable data. A peer continues after every fault.
- Parent-driven launch/wait and immutable page sharing. Sealing removes the
  producer's write permission; accepted readers survive producer exit or fault.
- Ordinary shared-object IPC through three processes: unmapped forwarding,
  post-exit reads, checked copyout, queue discard and sealed reader write faults.
- Rejection of the wrong ELF machine, admission failure at every frame boundary,
  shared core runtime-allocation rollback, and full frame/heap reclamation.

The targeted stack-entry/exhaustion probes also pass in debug builds. The full
debug suite's former `__chkstk` guard fault is fixed: shared core constructs its
session directly in fallibly allocated heap storage, and IPC reserves startup
handle rows on the heap for the configured task slots. Neither aggregate crosses
startup/result frames as a large inline value. The 64-KiB boot stack and its guard
remain unchanged. See the current Rust handoff for verification scope.

The board addresses are intentionally fixed, following
[QEMU's virt machine](https://www.qemu.org/docs/master/system/arm/virt.html).
The backend requires an EL1 firmware handoff, 4-KiB translation granules and
at least 40-bit physical addresses. It maps 0–1 GiB as supervisor device memory
and 1–2 GiB as supervisor normal RAM; the runner supplies only 128 MiB at
`0x40000000`. No DT/ACPI discovery, EL2 handoff, SMP, ASIDs or TLB shootdowns yet.

Task roots borrow the supervisor-only kernel branch and own their user branch.
EL0 cannot use the physical aliases; the lab checks this directly.
Runtime page-object table reservation visits one address per 2-MiB leaf-table
span, including partially covered first/last tables. With the current 52 guarded
slots, this takes 14 walks instead of 6,708 per-page walks. It allocates the same
tables without installing leaf mappings; frame custody and rollback remain in
core. The real-frame fixture reserves twice without additional frames, maps and
unmaps every slot using no additional table allocations, checks read-only data
and guards, and fails admission at every physical-frame allowance.
[Kernel image protection](../../memory/README.md) gives code RX, immutable data
RO/NX and mutable data RW/NX; all other identity RAM is NX, including heap and
stack backing. Fatal probes verify these permissions on the CPU.
Every root switch flushes translations. Loaded code is
cleaned to the point of unification before instruction-cache invalidation.

The shared [kernel-task lab](../../../boot/uefi/lab/kernel_tasks/kernel_tasks.rs)
runs before user processes, with an ISA-specific register probe. Each 64-KiB
EL1 task stack occupies a separate 2-MiB slot in a dedicated root branch.
Teardown detaches empty tables and flushes translations before refund; admission
failure tests cover all nineteen allocations for the first stack.

This port reports kernel stack exhaustion terminally; recovery and userspace
framebuffer mapping remain unimplemented. PC PS/2 and ATA grants are rejected at composition. The matrix
desktop remains on x86-64 until ARM device
adapters exist. CPU-budget policies are shared; their full stress suite has only
been exercised on x86-64. Passing these tests is not full backend parity.
