# Omega boot frontier

This is the real Cathedral boot source, not a reduced hello-world surrogate.
It is currently **not compile- or boot-verified** with the current compiler.
Historical July boot comments do not establish that the current tree runs.

The intended acceptance is one freshly compiled `uefi_x86_64` image under
QEMU/OVMF: `Hello from Omega`, successful firmware exit, `Owned N MiB` over
COM1, then idle while retaining the granted root. A failed build or an old
`BOOTX64.EFI` cannot satisfy acceptance.

## Reproduce

Use an optimized current Omega checkout or pass its exact binary:

```text
python tools/boot-harness/run.py --compiler PATH_TO_OMEGA --prepare --build-only
python tools/boot-harness/run.py --compiler PATH_TO_OMEGA --smoke
```

`--prepare` invokes ordinary package review; it does not accept trust grants.
Follow the emitted review instructions before attempting the build again.
The obsolete text-format `omega.lock` was removed; a replacement must come
from a successful checked review, not hand-authored acceptance records.

## Work still required

- Finish checking the migrated source. The current compiler counts the explicit
  `&Binding<R>` argument of `R::operation` as an extra ordinary argument in
  the four firmware/root-provider calls. Do not restore receiver-call syntax
  or discard the authority argument to get past this.
- Discharge the memory-map recast bounds at `walk` and `audit_descriptor`,
  and finish explicit arithmetic-domain conversions in geometry and serial
  formatting. Keep the actual descriptor integrity, overlap and retry checks.
- Connect the checked physical firmware adapter to the semantic entry and
  Cathedral continuation. Omega's `UEFI-PHYSICAL-SEMANTIC-ENTRY` and
  `UEFI-OS-HANDOFF` outcomes own the general compiler/target-source mechanics.
  `Main::run` is ordinary checked logic, not an implicitly installed raw ABI.
  PE subsystem selection, a same-shaped pointer, or an empty root provider is
  not proof of firmware validity or storage authority.

The local calling policy deliberately remains a normal named implementation;
it must not collide with std's same-spelled policy. Firmware layouts and OS
map-selection/retry policy stay in authored Omega, not compiler intrinsics.

The Rust bring-up lab is a separate product and does not discharge this
acceptance. Keep the harness failing visibly until this exact source produces
and boots a fresh image.
