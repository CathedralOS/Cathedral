# Cathedral Rust lab

A bootable Rust implementation for working out Cathedral's OS mechanisms while
Omega develops. `source/` remains the Omega implementation; this workspace
mirrors its ownership layers and names as behavior is implemented. It is an
experimental kernel, with explicit unsafe hardware operations and no claim to
Omega's proof or authority guarantees.

## Layout

| Path | Responsibility / Omega counterpart |
| --- | --- |
| `boot/uefi/main.rs` | Visible orchestration of firmware entry and post-handoff kernel startup |
| `boot/uefi/firmware.rs` | UEFI crate adapter and memory-inventory policy |
| `boot/uefi/memory.rs`, `handoff.rs` | Compose core frame policy with architecture mappings, then transfer boot state |
| `boot/uefi/interrupts.rs`, `diagnostics.rs` | Interrupt bring-up and serial/fatal reporting |
| `boot/uefi/smoke.rs` | Test-only fault injection and QEMU result reporting |
| `contracts/boot.rs` | Firmware-neutral memory handoff; experimental Rust data, not a frozen ABI |
| `core/extent.rs` | Physical-memory inventory and initial frame accounting, corresponding to the resource work in `source/core/` |
| `drivers/uart_16550/` | Polling serial diagnostics, corresponding to `source/drivers/uart_16550/` |
| `arch/lib.rs` | Compile-time CPU backend selection and the boot-facing machine interface |
| `arch/x86/` | Shared instructions and the selected PC platform's temporary PIC/PIT route |
| `arch/x86_64/` | Paging, stack entry, GDT/TSS/IDT and interrupt stubs, using the `x86_64` crate |
| `../tools/boot-harness-rs/` | Host build, QEMU launch and smoke verification |

Each crate uses `no_std`. Core policies are host-testable and have no firmware
dependency. Core's hardware-facing modules explicitly opt into unsafe code and
depend on `arch/`; drivers never depend on core internals. Boot assembles these
subsystems and the firmware adapter.
The bootstrap UART currently runs privileged; user-mode drivers come later.
The Rust-specific `arch/` layer groups hardware mechanisms that Omega currently
spreads across core providers, instruction contracts and libraries. Only x86-64
boots today; shared x86 instructions do not imply a working 32-bit kernel.
UEFI ABI definitions and Boot Services come from the upstream `uefi` crate.
`foundation/`, `services/` and `applications/` appear when their first code lands,
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
7. Map a guarded 64-KiB kernel stack, four guarded 16-KiB emergency stacks and
   64 KiB of heap backing. New mappings are writable and non-executable.
8. Load the owned CR3, enable write protection/NX, switch stacks and confirm the
   stack pointer is within its assigned range.
9. Load Cathedral's GDT/TSS and complete 256-entry IDT. Double fault, NMI,
   machine check and maskable interrupts have distinct emergency stacks.
10. Execute `int3` and verify return, then start the 100-Hz PIC/PIT bootstrap
    timer. Receive three ticks and verify the IRQ used its assigned stack.
11. Print `CATHEDRAL_RS_BOOT_OK`, then idle or terminate the smoke-test guest.

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
Fatal and NMI handlers must never allocate. There is no scheduler,
syscall path or isolated driver. The timer ISR only records a tick, acknowledges
the PIC and returns; it makes no Rust calls and touches no SIMD/FPU state.
Exception stubs normalize hardware error codes before a terminal diagnostic;
only the breakpoint self-test resumes. Unexpected external vectors report 255.
The CPU profile is qemu64, with no claim to optional virtualization exception
semantics or physical-hardware coverage. Unsafe wrappers are boot-only lab
mechanisms, not application APIs.

## Next bring-up steps

- Add context switching and two scheduled tasks.
- Discover ACPI/APIC topology and replace the temporary PIC/PIT timer route.
- Add user-mode address spaces, capability checks and shared-memory IPC.

Keep source transitions and invariants recognizable beside their Omega owners.
Record deliberate divergences here and preserve test cases for eventual shared
conformance testing. Rust data validation is not proof of firmware custody;
QEMU success is not verification on physical hardware.
