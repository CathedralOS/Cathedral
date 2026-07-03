# Boot Harness — build the UEFI app and boot it under QEMU/OVMF

Host-side dev tool (never ships). Builds the milestone-1 UEFI application
([`source/boot/uefi`](../../source/boot/uefi)) and boots it under QEMU with OVMF
(open-source UEFI firmware), so "does it print?" is one command.

## Status

The QEMU invocation is **real and correct today**; the **build step is stubbed**
because the Omega toolchain cannot emit a UEFI target yet (milestone 1 of the
first-boot ladder — see [`source/boot/uefi/main.omg`](../../source/boot/uefi/main.omg)
for the exact list of Omega features it is waiting on). Until then the scripts
set up the firmware and the EFI System Partition, then report the one missing
piece instead of pretending to boot nothing.

When `omega build` can produce `BOOTX64.EFI`, this harness runs end to end with
no change — that is the point of stubbing it now.

## What it does

1. **Build** `source/boot/uefi` → `BOOTX64.EFI` (the Uefi64 target is declared in
   that package's `build.omg`; the toolchain reads it). *Stubbed today.*
2. **Assemble an EFI System Partition** — a directory laid out as
   `esp/EFI/BOOT/BOOTX64.EFI`, the path UEFI firmware boots by convention.
3. **Run QEMU** with OVMF as the firmware and the ESP presented as a FAT disk.
   OVMF loads `\EFI\BOOT\BOOTX64.EFI` and calls it — exactly like real hardware.

Output goes to the serial console (`-serial stdio`), where you should see
`Hello from Omega` once the app builds.

## Prerequisites

- **QEMU** (`qemu-system-x86_64` on `PATH`).
  - Windows: install from qemu.org; add its folder to `PATH`.
  - Linux: `apt install qemu-system-x86` (or your distro's package).
- **OVMF firmware** (`OVMF.fd`). The scripts look for it via, in order:
  1. the `OVMF` environment variable (`OVMF=/path/to/OVMF.fd`),
  2. `reference_code/rust-osdev/ovmf-prebuilt/` (the gitignored reference clone),
  3. common system locations (`/usr/share/ovmf/…`, QEMU's `edk2-*` on Windows).
  - Get it from your distro's `ovmf` / `edk2-ovmf` package, from QEMU's Windows
    install, or by cloning `rust-osdev/ovmf-prebuilt` into `reference_code/`.

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

- The single-file `-bios OVMF.fd` form is used for simplicity. The "proper"
  split `OVMF_CODE.fd` + `OVMF_VARS.fd` (via `-drive if=pflash,…`) is what a real
  target uses; switch to it when NVRAM variables start mattering (past
  milestone 1).
- The ESP and build outputs land under `build/boot-harness/` (gitignored).
- `-no-reboot` and `-serial stdio` keep a failed boot from looping and route
  firmware/app output to your terminal.
