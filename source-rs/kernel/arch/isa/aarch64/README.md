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

## Evidence and bounds

Debug and release boots exercise:

- Firmware exit, owned tables/stack, breakpoint return, heap exhaustion/reuse,
  timer delivery on the exception stack.
- Trusted EL1 tasks yielding, sleeping and preempting with GPR/FP/SIMD/FPCR/FPSR
  sentinels; dynamic spawn, stale IDs, frame/heap exhaustion and live-peer progress.
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

This port does not recover from kernel stack overflow or map a userspace
framebuffer. PC PS/2 and ATA grants are rejected at composition. The matrix
desktop remains on x86-64 until ARM device
adapters exist. CPU-budget policies are shared; their full stress suite has only
been exercised on x86-64. Passing these tests is not full backend parity.
