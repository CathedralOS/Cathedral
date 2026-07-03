#!/usr/bin/env bash
# tools/boot-harness/run.sh
# Build the milestone-1 UEFI app and boot it under QEMU/OVMF. Host-side dev tool;
# never ships. See README.md. The QEMU invocation is real; the build step is
# stubbed until the Omega toolchain can emit a UEFI target.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BOOT_PKG="$REPO_ROOT/source/boot/uefi"
BUILD_DIR="$REPO_ROOT/build/boot-harness"
ESP="$BUILD_DIR/esp"
EFI_OUT="$ESP/EFI/BOOT/BOOTX64.EFI"

# --- locate OVMF firmware --------------------------------------------------
OVMF="${OVMF:-}"
if [[ -z "$OVMF" ]]; then
  for cand in \
    "$REPO_ROOT/reference_code/rust-osdev/ovmf-prebuilt/OVMF-pure-efi.fd" \
    "/usr/share/ovmf/OVMF.fd" \
    "/usr/share/edk2-ovmf/x64/OVMF.fd" \
    "/usr/share/OVMF/OVMF_CODE.fd" ; do
    [[ -f "$cand" ]] && OVMF="$cand" && break
  done
fi
if [[ -z "$OVMF" || ! -f "$OVMF" ]]; then
  echo "error: OVMF firmware not found. Set OVMF=/path/to/OVMF.fd" >&2
  echo "  (install edk2-ovmf, or clone rust-osdev/ovmf-prebuilt into reference_code/)" >&2
  exit 1
fi

mkdir -p "$(dirname "$EFI_OUT")"

# --- build the boot package ------------------------------------------------
# STUB: the Uefi64 target is declared in $BOOT_PKG/build.omg; when the toolchain
# can emit it, this is the whole build:
#     omega build "$BOOT_PKG" -o "$EFI_OUT"
if command -v omega >/dev/null 2>&1; then
  echo "building $BOOT_PKG -> $EFI_OUT"
  omega build "$BOOT_PKG" -o "$EFI_OUT"
else
  echo "note: no 'omega' toolchain on PATH — UEFI build not available yet" >&2
  echo "      (milestone 1 of the first-boot ladder; see source/boot/uefi/main.omg)" >&2
  if [[ ! -f "$EFI_OUT" ]]; then
    echo "      no prebuilt $EFI_OUT to boot; firmware + ESP are set up, stopping here." >&2
    exit 2
  fi
  echo "      booting the existing $EFI_OUT" >&2
fi

# --- boot it ---------------------------------------------------------------
echo "booting under QEMU (OVMF: $OVMF)"
exec qemu-system-x86_64 \
  -bios "$OVMF" \
  -drive "format=raw,file=fat:rw:$ESP" \
  -net none \
  -serial stdio \
  -no-reboot
