# Cathedral Rust lab

A bootable Rust implementation for working out Cathedral's OS mechanisms while
Omega develops. `source/` remains the Omega implementation; this workspace
mirrors its ownership layers and names as behavior is implemented. It is an
experimental kernel, with explicit unsafe hardware operations and no claim to
Omega's proof or authority guarantees.

## Layout

| Path | Responsibility / Omega counterpart |
| --- | --- |
| `boot/uefi/main.rs` | Firmware entry, exit and adaptation, like `source/boot/uefi/main.omg` |
| `contracts/boot.rs` | Firmware-neutral memory handoff; experimental Rust data, not a frozen ABI |
| `core/extent.rs` | Physical-memory inventory and initial frame accounting, corresponding to the resource work in `source/core/` |
| `drivers/uart_16550/` | Polling serial diagnostics, corresponding to `source/drivers/uart_16550/` |
| `libraries/x86_64/` | Explicit unsafe instruction wrappers for the x86 hardware seam |
| `../tools/boot-harness-rs/` | Host build, QEMU launch and smoke verification |

Each crate uses `no_std`. The core depends only on contracts and has no firmware
dependency or unsafe code. Boot assembles the core, UART and firmware adapter.
The bootstrap UART currently runs privileged; user-mode drivers come later.
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

The final command leaves the CPU idling after handoff; Ctrl+C stops QEMU.
`--release` selects an optimized build. `--memory 256` selects guest RAM in MiB.
Smoke mode has a 30-second boot deadline, requires ordered serial milestones,
and checks the QEMU debug-exit status. A panic, missing milestone, reset or hang
fails the run. Its debug-exit feature is not enabled for ordinary boots.

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

QEMU q35, one x86-64 CPU, software emulation, 128 MiB by default:

1. Enter a real UEFI application through OVMF and report over COM1.
2. Capture the final firmware memory map and exit Boot Services using `uefi`.
3. Disable maskable interrupts and adapt the map into a fixed-capacity inventory.
4. Reject overlapping, unaligned, overflowing or oversized inventories.
5. Move the inventory into the core and allocate the first available 4-KiB frame
   in accounting, skipping reserved memory and physical page zero.
6. Print `CATHEDRAL_RS_BOOT_OK`, then idle or terminate the smoke-test guest.

Only ordinary conventional RAM is eligible. Loader memory, Boot Services memory,
runtime memory, ACPI and MMIO remain reserved. Runtime-marked, hot-pluggable and
specific-purpose conventional regions also remain reserved. This deliberately
retains the image, map buffer, firmware stack and inherited page tables. The
inventory accepts at most 256 descriptors and fails closed beyond that.

The frame allocator does not access the frame, create mappings, free memory or
establish Omega-qualified ownership. There is no heap, custom stack/page table,
Cathedral IDT, enabled timer, scheduler, syscall path or isolated driver yet.
The inherited firmware execution environment is temporary. Unsafe wrappers are
boot-only lab mechanisms, not a capability API to expose to applications.

## Next bring-up steps

- Reserve and build Cathedral-owned page tables and a guarded stack; retain all
  live mappings through the switch before reclaiming any bootstrap resources.
- Establish diagnostic exception entries and emergency stacks before enabling
  the first timer; follow `wiki/boot/02a_idt_handoff.md`.
- Add the heap, timer, context switching and two scheduled tasks.
- Add user-mode address spaces, capability checks and shared-memory IPC.

Keep source transitions and invariants recognizable beside their Omega owners.
Record deliberate divergences here and preserve test cases for eventual shared
conformance testing. Rust data validation is not proof of firmware custody;
QEMU success is not verification on physical hardware.
