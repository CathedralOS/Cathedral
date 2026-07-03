# tools/boot-harness/run.ps1
# Build the milestone-1 UEFI app and boot it under QEMU/OVMF. Host-side dev tool;
# never ships. See README.md. The QEMU + firmware loop is verified; the build
# step is stubbed until the Omega toolchain can emit a UEFI target.
$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$BootPkg  = Join-Path $RepoRoot "source\boot\uefi"
$BuildDir = Join-Path $RepoRoot "build\boot-harness"
$Esp      = Join-Path $BuildDir "esp"
$EfiOut   = Join-Path $Esp "EFI\BOOT\BOOTX64.EFI"

# --- locate QEMU (PATH, then the default Windows install) ------------------
$Qemu = (Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue).Source
if (-not $Qemu -and (Test-Path "C:\Program Files\qemu\qemu-system-x86_64.exe")) {
  $Qemu = "C:\Program Files\qemu\qemu-system-x86_64.exe"
}
if (-not $Qemu) { Write-Error "qemu-system-x86_64 not found (PATH or C:\Program Files\qemu)." }
$QemuShare = Join-Path (Split-Path $Qemu) "share"

New-Item -ItemType Directory -Force -Path (Split-Path $EfiOut) | Out-Null

# --- resolve OVMF firmware -------------------------------------------------
# Prefer the SPLIT form (code + writable vars) — what modern QEMU ships. Copy
# both into the build dir so paths are space-free and the vars store is
# writable. Fall back to a single combined OVMF.fd via $env:OVMF.
$FwArgs = $null
if ($env:OVMF -and (Test-Path $env:OVMF)) {
  $FwArgs = @("-bios", $env:OVMF)                       # single combined image
} else {
  $codeSrc = $null; $varsSrc = $null
  foreach ($pair in @(
      @((Join-Path $QemuShare "edk2-x86_64-code.fd"), (Join-Path $QemuShare "edk2-i386-vars.fd")),
      @((Join-Path $RepoRoot "reference_code\rust-osdev\ovmf-prebuilt\OVMF_CODE.fd"),
        (Join-Path $RepoRoot "reference_code\rust-osdev\ovmf-prebuilt\OVMF_VARS.fd")))) {
    if ((Test-Path $pair[0]) -and (Test-Path $pair[1])) { $codeSrc = $pair[0]; $varsSrc = $pair[1]; break }
  }
  if (-not $codeSrc) {
    Write-Error "OVMF firmware not found. Set `$env:OVMF to a combined OVMF.fd, or ensure QEMU ships edk2-x86_64-code.fd + edk2-i386-vars.fd in its share dir."
  }
  $code = Join-Path $BuildDir "ovmf_code.fd"
  $vars = Join-Path $BuildDir "ovmf_vars.fd"
  Copy-Item $codeSrc $code -Force
  Copy-Item $varsSrc $vars -Force                        # per-run reset; vars must be writable
  $FwArgs = @("-drive","if=pflash,format=raw,readonly=on,file=$code",
              "-drive","if=pflash,format=raw,file=$vars")
}

# --- build the boot package (STUB) -----------------------------------------
# When the toolchain can emit the Uefi64 target (declared in build.omg):
#     omega build $BootPkg -o $EfiOut
if (Get-Command omega -ErrorAction SilentlyContinue) {
  Write-Host "building $BootPkg -> $EfiOut"
  & omega build $BootPkg -o $EfiOut
} else {
  Write-Warning "no 'omega' toolchain on PATH - UEFI build not available yet (milestone 1)."
  if (-not (Test-Path $EfiOut)) {
    Write-Warning "no BOOTX64.EFI to boot yet. Firmware + ESP are set up and the QEMU loop is verified (empty ESP boots OVMF to its boot manager). Stopping."
    return
  }
}

# --- boot ------------------------------------------------------------------
Write-Host "booting under QEMU (firmware ready; watch the serial console for output)..."
$dbg = Join-Path $BuildDir "ovmf-debug.log"
& $Qemu -machine q35 @FwArgs `
    -drive "format=raw,file=fat:rw:$Esp" `
    -net none -serial stdio `
    -debugcon "file:$dbg" -global "isa-debugcon.iobase=0x402" `
    -no-reboot
