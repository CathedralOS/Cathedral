# Encryption-profile physical frame arithmetic

This is a licensed derivative of `rust-osdev/x86_64` pinned at
`cc35c876d3badb57df54a66e22f7768a52be95f2`, primarily
`src/structures/paging/frame.rs`, with its physical address rules from `addr.rs`
and `structures/mem_encrypt.rs`. MIT OR Apache-2.0 notices are retained in
`reference_code/rust-osdev/x86_64/LICENSE-MIT` and `LICENSE-APACHE`; the Omega
source identifies the origin and modifications. The inventory records every
anchor in that file; its 30 translated anchors describe this component, and
36 omissions retain nominal types, unchecked constructors, formatting, trait
plumbing, existing default tests and Kani proof infrastructure outside it.
This inventory does not replace the default `pages` inventory.

## Semantic profile

`encrypted_frames.omg` accepts an ordinary copied `memory_encryption::State`
snapshot, raw numeric addresses, and one of the existing 4 KiB, 2 MiB or 1 GiB
geometries. Use states produced by `initial` and `configure`; no CPU support is
inferred from an accepted bit position. Admission excludes only the **current**
`bit_mask & PHYSICAL_MAX`. The accumulated PTE `address_mask` and polarity do
not change physical frame admission. Repeated 47 → 48 → 47 configuration is
therefore observably different from accumulating forbidden frame bits. A bit
at or above 52 adds no exclusion within the physical 52-bit range.

Every supplied address is admitted under the supplied snapshot before use,
including before `containing` rounds down. Already aligned addresses do not
bypass current-bit validation. PFN multiplication and ordinary frame arithmetic
check `u64` multiplication before physical address arithmetic. This deliberately
rejects multiplication overflow even where the pinned release Rust operator
wraps `count * size`; observations retain both the actual Rust outcome and this
chosen checked policy. Negative differences, physical overflow, an excluded
result bit, invalid sizes and unaligned frame inputs return `NumberResult::Rejected`.
`arithmetic_overflowing` returns the original address and `overflow=true` for
any rejection, matching the existing numeric result convention.

Inclusive/exclusive counts and byte sizes use numeric endpoint distance. They
do not remove excluded-bit bands. `range_at` selects a bounded numeric index
from either end and admits the selected result; it does not execute iterator
cursor advancement. Consequently a selection can succeed while advancing an
upstream iterator would panic.

`iterator_step` separately models pinned `nth`/`nth_back`, including index zero
for `next`/`next_back`. `IteratorResult` retains numeric start/end cursors,
selection, and a failure flag. `Rejected` with `failed=false` means exhaustion;
`failed=true` means rejected input or the corresponding checked arithmetic /
upstream panic boundary. Cursor updates completed before a later failure remain
visible. Yield occurs only after the pinned successor/predecessor update succeeds.
In particular, inclusive forward iteration computes its masked maximum before
advancing: a low excluded bit can make that maximum calculation fail. An
overshooting `nth` first consumes the last remaining selection, then tries the
next step; its partial cursor changes are preserved. All operations are bounded
numeric calculations, with no loop proportional to a caller index.

This profile admits inputs as if freshly constructed under the snapshot. It
does not model Rust objects kept alive across concurrent changes to global
configuration. The reference probes configure each isolated process before
constructing its values. Iterator size arithmetic uses the observed 64-bit
`usize` profile; 32-bit `size_hint` saturation, native trait objects, generic
iterator laws and Kani universal proofs are not claimed.

## Integration and authority

This optional pure module leaves existing default APIs unchanged. Numeric
addresses and cursor results are not `PhysFrame` identity, allocated backing,
page-table ownership, a mapping or access permission. There is no physical
access, allocator call, pointer conversion, device operation or boot integration.
Callers must separately establish all such capabilities and select a coherent
configuration snapshot. There is no native ABI layout claim.

## Verification and reproduction

See `tools/ports/x86_64-encrypted-frames/README.md` for the commands and retained
stage-specific evidence. The reference runner uses actual public pinned Rust
methods with `memory_encryption` enabled, all three sizes and 13 isolated
profiles. It records panic-versus-exhaustion distinctions and both cursor fields.
The Omega fixtures execute the authored bodies and compare those observations;
additional cases exercise invalid geometry. Changed expected acceptance/failure
conditions must produce a nonzero fixture result. Representative constant
`requires` checks are recorded separately from checked-interpreter execution.
Neither stage demonstrates native code execution, hardware behavior, universal
proof, or a production caller. Evidence is current only when `verify_record.py`
and inventory regeneration succeed against its exact source hashes.
