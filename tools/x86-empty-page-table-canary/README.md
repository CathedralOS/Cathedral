# x86 empty page-table-page source canary

This compile-only harness checks Cathedral's first source-owned bootstrap
page-table policy and state. The QEMU/UEFI-x64 profile explicitly selects the
baseline four-level, 48-bit canonical-address regime with LA57 disabled: four
9-bit table indexes at shifts 39, 30, 21, and 12 over 4-KiB pages. One ordinary
page candidate remains exactly 512 existing 8-byte x86 PTE values. A checked
decreasing-fuel scan visits all 512 slots and accepts only when every one of the
fourteen PTE fields has exact zero encoding. Success returns the complete
runtime-relevant candidate; a dirty slot rejects the whole page, and no partial
result escapes.

Run:

```sh
tools/x86-empty-page-table-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order.

`ZeroConsistent` is ordinary pre-construction policy data, not a sealed empty
capacity or installable table. The selected four-level profile is Cathedral's
first bootstrap policy, not an assertion that all x86-64 targets lack LA57. It
chooses no virtual layout, large-page policy, hierarchy page count, physical
frame, or mapping. It binds no pre-reserved storage and grants no `Extent`,
placement, page-table mutation, TLB, machine-control, installation, or teardown
authority.
