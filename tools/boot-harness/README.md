# Boot Harness — build the UEFI app and boot it under QEMU/OVMF

Host-side dev tool (never ships). Builds the Cathedral UEFI application
([`source/boot/uefi`](../../source/boot/uefi)) and boots it under QEMU with OVMF
(open-source UEFI firmware), so "does it print?" is one command.

## Status

The QEMU + firmware loop and Omega UEFI build run end to end. The current image
prints the milestone greeting, exits firmware boot services, reports its owned
memory over the 16550 serial driver, and idles with `hlt`.

## What it does

1. **Build** `source/boot/uefi` → `BOOTX64.EFI` (the Uefi64 target is declared in
   that package's `build.omg`; the toolchain reads it).
2. **Assemble an EFI System Partition** — a directory laid out as
   `esp/EFI/BOOT/BOOTX64.EFI`, the path UEFI firmware boots by convention.
3. **Run QEMU** with OVMF as the firmware and the ESP presented as a FAT disk.
   OVMF loads `\EFI\BOOT\BOOTX64.EFI` and calls it — exactly like real hardware.

Output goes to the serial console (`-serial stdio`), where the current milestone
prints `Hello from Omega` and its owned-memory total before idling.

## Prerequisites

- **QEMU** (`qemu-system-x86_64` on `PATH`).
  - Windows: install from qemu.org; add its folder to `PATH`.
  - Linux: `apt install qemu-system-x86` (or your distro's package).
- **OVMF firmware.** Modern QEMU ships the **split** form
  (`edk2-x86_64-code.fd` + `edk2-i386-vars.fd` in its `share/` dir) — the scripts
  prefer it, copying both into `build/boot-harness/` so the vars store is
  writable and paths are space-free. Resolution order:
  1. `$OVMF` — a single *combined* `OVMF.fd`, used via `-bios` (override);
  2. QEMU's bundled `share/edk2-x86_64-code.fd` + `edk2-i386-vars.fd`, including
     Homebrew's `share/qemu` layout;
  3. system split OVMF (`/usr/share/OVMF/OVMF_CODE.fd` + `OVMF_VARS.fd`, …);
  4. `reference_code/rust-osdev/ovmf-prebuilt/`.
  - On Windows, QEMU's own install supplies it — nothing extra to fetch. On
    Linux, `apt install ovmf` (or your distro's `edk2-ovmf`).

## Run

```sh
# bash / Git Bash / Linux
tools/boot-harness/run.sh
```

```powershell
# Windows PowerShell
tools\boot-harness\run.ps1
```

Set `OVMF` first if it is not auto-detected, e.g. `OVMF=/usr/share/ovmf/OVMF.fd`
(bash) or `$env:OVMF = "C:\path\to\OVMF.fd"` (PowerShell).

## Notes

- The **split** `pflash` form (`OVMF_CODE.fd` + writable `OVMF_VARS.fd`) is the
  default — it is what modern QEMU ships and what a real target uses. The vars
  copy is reset each run, so the boot manager starts clean. A single combined
  `OVMF.fd` via `-bios` is still honored through `$OVMF` for older setups.
- The ESP, the firmware copies, and QEMU/OVMF logs land under
  `build/boot-harness/` (gitignored). `ovmf-debug.log` captures the firmware's
  (and later the app's) debug output.
- `-machine q35` (modern chipset), `-no-reboot` (a failed boot won't loop), and
  `-serial stdio` (firmware/app console mirrored to your terminal).
