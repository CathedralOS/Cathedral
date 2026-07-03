# tools/boot-harness/run.ps1
# Build the milestone-1 UEFI app and boot it under QEMU/OVMF. Host-side dev tool;
# never ships. See README.md. The QEMU invocation is real; the build step is
# stubbed until the Omega toolchain can emit a UEFI target.
$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$BootPkg  = Join-Path $RepoRoot "source\boot\uefi"
$BuildDir = Join-Path $RepoRoot "build\boot-harness"
$Esp      = Join-Path $BuildDir "esp"
$EfiOut   = Join-Path $Esp "EFI\BOOT\BOOTX64.EFI"

# --- locate OVMF firmware --------------------------------------------------
$Ovmf = $env:OVMF
if (-not $Ovmf) {
  foreach ($cand in @(
    (Join-Path $RepoRoot "reference_code\rust-osdev\ovmf-prebuilt\OVMF-pure-efi.fd"),
    "C:\Program Files\qemu\share\edk2-x86_64-code.fd"
  )) { if (Test-Path $cand) { $Ovmf = $cand; break } }
}
if (-not $Ovmf -or -not (Test-Path $Ovmf)) {
  Write-Error "OVMF firmware not found. Set `$env:OVMF to OVMF.fd (install QEMU's edk2-ovmf, or clone rust-osdev/ovmf-prebuilt into reference_code\)."
}

New-Item -ItemType Directory -Force -Path (Split-Path $EfiOut) | Out-Null

# --- build the boot package ------------------------------------------------
# STUB: the Uefi64 target is declared in $BootPkg\build.omg; when the toolchain
# can emit it, this is the whole build:  omega build $BootPkg -o $EfiOut
$omega = Get-Command omega -ErrorAction SilentlyContinue
if ($omega) {
  Write-Host "building $BootPkg -> $EfiOut"
  & omega build $BootPkg -o $EfiOut
} else {
  Write-Warning "no 'omega' toolchain on PATH - UEFI build not available yet (milestone 1)."
  if (-not (Test-Path $EfiOut)) {
    Write-Warning "no prebuilt BOOTX64.EFI to boot; firmware + ESP are set up, stopping here."
    exit 2
  }
  Write-Host "booting the existing $EfiOut"
}

# --- boot it ---------------------------------------------------------------
Write-Host "booting under QEMU (OVMF: $Ovmf)"
& qemu-system-x86_64 -bios $Ovmf -drive "format=raw,file=fat:rw:$Esp" -net none -serial stdio -no-reboot
