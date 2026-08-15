# x86 empty page-table-page source canary

This compile-only harness checks Cathedral's first source-owned bootstrap
page-table policy and state. The QEMU/UEFI-x64 profile explicitly selects the
baseline four-level, 48-bit canonical-address regime with LA57 disabled: four
9-bit table indexes at shifts 39, 30, 21, and 12 over 4-KiB pages. One ordinary
address helper accepts only `0x0000000000000000..=0x00007fffffffffff` or
`0xffff800000000000..=0xffffffffffffffff`, retains the original address, and
derives the four bounded indexes plus its 12-bit page offset. A separate pure
validator accepts an ordinary retained decomposition only when the address is
canonical and all four indexes and the offset exactly match those same shifts
and masks. An ordinary physical-frame helper separately accepts only a
4-KiB-aligned base inside the
52-bit physical-address envelope, retains that address, and derives its bounded
40-bit PFN. An ordinary PTE composer then binds those address bits while
requiring the caller to supply every other field in the complete 64-bit entry
schema; it does not interpret the role-dependent `page_size_or_pat` bit or
select leaf/link policy. One ordinary page candidate remains exactly 512
existing 8-byte x86 PTE values. A checked decreasing-fuel scan visits all 512
slots and accepts only when every one of the fourteen PTE fields has exact zero
encoding. Success returns the complete runtime-relevant candidate; a dirty slot
rejects the whole page, and no partial result escapes.

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
authority. Canonical address indexes are numeric walk coordinates only; they do
not prove that a hierarchy, mapping, or address-space claim exists; successful
revalidation adds only numeric consistency. Physical
frame geometry is likewise only a numeric candidate: a later provider must
rebind the same address and PFN while holding the exact physical source and
mapping authority. The detached PTE candidate adds no claim that its supplied
bits form a valid leaf or hierarchy link and grants no installation reach.
