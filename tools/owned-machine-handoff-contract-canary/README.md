# owned-machine handoff contract source canary

This compile-only harness checks Cathedral's existing milestone-2/3 handoff
machine independently of the unresolved UEFI target-entry bridge. It pins each
fresh memory map to its returned key and permits exactly one stale-key
`EFI_INVALID_PARAMETER` result from `ExitBootServices` to discard the old
candidate and restart that whole transaction. A second stale rejection parks;
only success admits the qualified extent grant. Other failures park without
acquiring an extent; the post-firmware serial/owned-idle graph retains the one
successful grant.

Before any Boot Services dispatch, the enclosing EFI System Table and the Boot
Services table must carry their standard signatures, advertise headers large
enough to cover every field Cathedral reads, agree on a well-formed UEFI
revision at or above the x86-64 profile's 2.0 floor, and keep the common
reserved header field zero. The canary pins both fail-closed gates and permits
the first fresh map transaction only after they succeed. It does not yet claim
whole-table CRC validation.

The owned-memory report remains exact through 99,999 MiB. Larger roots take a
separate `99999+ MiB` route rather than overflowing the five-digit formatter;
page-to-MiB conversion saturates at 100,000 subtraction rounds because no larger
exact value affects that route. Both outputs fit the 16550's 16-byte FIFO burst
and retain the same qualified extent through owned idle. Each burst gets at
most 1,000,000 readiness reads; a stuck transmitter parks without writing into
the busy UART and retains the qualified root.

Before the irreversible exit, the selected conventional span must not carry
`EFI_MEMORY_RUNTIME`, `EFI_MEMORY_HOT_PLUGGABLE`, or `EFI_MEMORY_SP`; it must be
nonempty, page-aligned, small enough for exact page-to-byte conversion, and have
a representable one-past end. Runtime-marked, removable, and specific-purpose
descriptors remain ineligible. The validated byte length stays in the same
map/key transaction while a second full-map pass validates every descriptor's
standard/vendor memory-type range, physical and virtual alignment, and range
end. It also rejects attribute bits outside the revision-1 standard and ISA-
specific masks, rejects ISA-specific bits without their validity flag, then
requires every other physical range to be strictly disjoint from the chosen
span. Invalid type `16..0x6fffffff`, reserved or unflagged ISA attributes,
malformed geometry, or overlap parks inside firmware without
calling the provider. Only completion of that audit may reach
`ExitBootServices`; successful exit makes the checked span the exact grant
geometry.

Map storage is a fixed 64-KiB zero-initialized field, not an allocation or
memory capability. An explicit leading `u64` makes the byte view 8-byte
aligned; the canary requires both storage members to remain runtime-relevant so
neither part of that representation can erase. Successful map results
additionally require an 8-byte-multiple
descriptor stride, a whole number of descriptors, and the exact supported UEFI
descriptor revision (version 1) before any typed recast. An unknown revision is
a malformed success and parks unowned.
Cathedral advertises 16 KiB first; only exact
`EFI_BUFFER_TOO_SMALL` with a reported requirement in `(16 KiB, 64 KiB]`
permits one retry advertising the full backing. A second capacity failure,
oversize requirement, malformed success, or unrelated error parks unowned.

Run:

```sh
tools/owned-machine-handoff-contract-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order. Artifact validation uses `jq`
and fails with an explicit dependency error when it is unavailable.

The canary compiles and inspects `Main::own_machine` but never calls it. It does
not select or generate the UEFI program-entry stub, compose physical firmware
inputs with semantic roots, call firmware, grant memory, perform port I/O, or
execute `hlt`.
