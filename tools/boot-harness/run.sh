#!/usr/bin/env bash
# tools/boot-harness/run.sh
# Build the milestone-1 UEFI app and boot it under QEMU/OVMF. Host-side dev tool;
# never ships. See README.md. The QEMU + firmware loop is verified; the build
# step is stubbed until the Omega toolchain can emit a UEFI target.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BOOT_PKG="$REPO_ROOT/source/boot/uefi"
BUILD_DIR="$REPO_ROOT/build/boot-harness"
ESP="$BUILD_DIR/esp"
EFI_OUT="$ESP/EFI/BOOT/BOOTX64.EFI"

# --- locate QEMU (PATH, then the default Windows install) ------------------
QEMU="$(command -v qemu-system-x86_64 || true)"
if [[ -z "$QEMU" && -x "/c/Program Files/qemu/qemu-system-x86_64.exe" ]]; then
  QEMU="/c/Program Files/qemu/qemu-system-x86_64.exe"
fi
[[ -z "$QEMU" ]] && { echo "error: qemu-system-x86_64 not found" >&2; exit 1; }
QEMU_SHARE="$(dirname "$QEMU")/share"

mkdir -p "$(dirname "$EFI_OUT")"

# --- resolve OVMF firmware -------------------------------------------------
# Prefer the SPLIT form (code + writable vars). Copy both into the build dir so
# paths are space-free and the vars store is writable. Fall back to a single
# combined OVMF.fd via $OVMF.
FW_ARGS=()
if [[ -n "${OVMF:-}" && -f "${OVMF:-}" ]]; then
  FW_ARGS=(-bios "$OVMF")
else
  CODE_SRC=""; VARS_SRC=""
  for pair in \
    "$QEMU_SHARE/edk2-x86_64-code.fd|$QEMU_SHARE/edk2-i386-vars.fd" \
    "/usr/share/OVMF/OVMF_CODE.fd|/usr/share/OVMF/OVMF_VARS.fd" \
    "/usr/share/edk2/x64/OVMF_CODE.fd|/usr/share/edk2/x64/OVMF_VARS.fd" \
    "$REPO_ROOT/reference_code/rust-osdev/ovmf-prebuilt/OVMF_CODE.fd|$REPO_ROOT/reference_code/rust-osdev/ovmf-prebuilt/OVMF_VARS.fd" ; do
    c="${pair%%|*}"; v="${pair##*|}"
    [[ -f "$c" && -f "$v" ]] && { CODE_SRC="$c"; VARS_SRC="$v"; break; }
  done
  if [[ -z "$CODE_SRC" ]]; then
    echo "error: OVMF firmware not found. Set OVMF=/path/to/OVMF.fd (combined), or install split OVMF_CODE.fd + OVMF_VARS.fd." >&2
    exit 1
  fi
  cp -f "$CODE_SRC" "$BUILD_DIR/ovmf_code.fd"
  cp -f "$VARS_SRC" "$BUILD_DIR/ovmf_vars.fd"     # per-run reset; vars must be writable
  FW_ARGS=(-drive "if=pflash,format=raw,readonly=on,file=$BUILD_DIR/ovmf_code.fd"
           -drive "if=pflash,format=raw,file=$BUILD_DIR/ovmf_vars.fd")
fi

# --- build the boot package (STUB) -----------------------------------------
# When the toolchain can emit the Uefi64 target (declared in build.omg):
#     omega build "$BOOT_PKG" -o "$EFI_OUT"
if command -v omega >/dev/null 2>&1; then
  echo "building $BOOT_PKG -> $EFI_OUT"
  omega build "$BOOT_PKG" -o "$EFI_OUT"
else
  echo "note: no 'omega' toolchain on PATH — UEFI build not available yet (milestone 1)" >&2
  if [[ ! -f "$EFI_OUT" ]]; then
    echo "      no BOOTX64.EFI to boot yet. Firmware + ESP are set up and the QEMU loop is verified;" >&2
    echo "      stopping (an empty ESP boots OVMF to its boot manager)." >&2
    exit 2
  fi
fi

# --- boot ------------------------------------------------------------------
echo "booting under QEMU (watch the serial console for output)..."
exec "$QEMU" -machine q35 "${FW_ARGS[@]}" \
  -drive "format=raw,file=fat:rw:$ESP" \
  -net none -serial stdio \
  -debugcon "file:$BUILD_DIR/ovmf-debug.log" -global "isa-debugcon.iobase=0x402" \
  -no-reboot
