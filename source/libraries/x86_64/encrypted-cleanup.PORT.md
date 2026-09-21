# Explicit-profile cleanup composition

Status: **tested**. All 384 pinned Rust complete-tree cases, 430 Omega positive
bodies and nine body-mutating controls pass. This is the encryption-profile
composition of detached cleanup branch and range plans, not a live mapper or
frame-reclamation service.

Modified from rust-osdev/x86_64 crate 0.15.5 at
`cc35c876d3badb57df54a66e22f7768a52be95f2`, under the retained
[MIT OR Apache-2.0 notices](../../../licenses/rust-osdev/x86_64/).
The [full-file source inventory](encrypted-cleanup-inventory.json) maps the
cleanup anchors in `mapped_page_table.rs` and `recursive_page_table.rs`, and
binds the PTE, physical-address and memory-encryption source files. Other mapper
methods remain in their separate inventories. The library charter and current
Omega data, ownership and boundary specifications were consulted.

## Existing types and profile semantics

The three modules introduce no nominal data types. They reuse
`memory_encryption::State`, `CleanupPlan`, `CleanupOutcome`, `Cursor`,
`RangeStatus`, `RangeStep`, `RecursiveCursor` and `RecursiveStep`.

`encrypted_cleanup_branch::plan` accepts the explicit profile followed by the
ordinary selected-word, snapshot-ID, other-entry-occupancy and page inputs of the
existing cleanup branch. It decodes child identities with the accumulated
`profile.address_mask`. It retains the original words for PRESENT/HUGE tests,
whole-word emptiness, unchanged output words and all sibling observations. A
word containing only an old encryption bit remains nonempty even when its
decoded address is zero. A PRESENT non-huge entry with decoded address zero may
name an ordinary captured frame-zero snapshot; the number grants no access.

Profiles are the states derived by `memory_encryption::initial/configure`.
Repeated configuration clears all configured address positions from PTE decoding,
while its latest physical bit and polarity remain distinct fields. Cleanup uses
the accumulated PTE mask; it adds no independent latest-bit rejection before
traversal or retirement. Decoded PTE frames already exclude the configured bits.
Plain arbitrary records do not establish configuration provenance or CPU support.

Missing/nonpresent/huge parents stop descent under the source's `frame()` rules.
Every descended link must match the next captured snapshot ID. A mismatch keeps
all words and reports no detach request from that branch. Emptiness considers
the selected word and every other entry. Empty children produce parent clears
and decoded retirement addresses in deepest-first order; occupied siblings stop
upward retirement after retaining earlier clears. The root is never retired.
The existing `cleanup_branch::has_other_entries` computes occupancy from a full
initialized PTE snapshot and needs no profile adaptation: zero is tested on the
original word.

## Range and recursive composition

`encrypted_cleanup_ranges` delegates `begin/resume` to the canonical range
machines and applies the profile-aware branch in `step`. Its private coverage,
canonical-gap and budget arithmetic is retained verbatim from `cleanup_ranges`;
the harness checks this identity. The original private geometry cannot be called
independently, so this module contains the same small implementation while
reusing the public cursor and result types. No normalized substitute words are
fed to the default cleanup machine.

`encrypted_cleanup_recursive` delegates `begin/resume` and excluded-self-slot
steps to `recursive_cleanup`. Ordinary slots use the profile range step. A skipped
self-slot ignores supplied captures, consumes one unit and skips its remaining
512 GiB coverage. Its link remains part of every ordinary root occupancy
observation. Active cursors and recursive indices are revalidated. Maximum u64
budgets retain the existing scalar-staging source form.

Each step consumes a detached consistent capture. Callers retain successful
earlier plans and apply their proposed effects to their next capture. A later
capture mismatch neither rolls back those earlier plans nor advances its failing
cursor. Resumption replaces the budget only for an exhausted cursor. The same
geometry accepts complete address-space ranges through bounded successive steps;
this is not limited to the finite witness topologies.

Snapshot IDs and retirement lists establish no live pointer, allocator custody,
alias freedom, synchronization, mapping publication or completed invalidation.
The machines never invoke deallocation or release backing ownership. None of
those live operations is replaced by a callable authority-shaped value.

## Evidence and limits

The Rust corpus passes 181 actual public mapped cleanup calls and 203 explicitly
adapted private recursive cleanup executions in 11 isolated profiles. It compares
complete final trees and ordered deallocator observations. Stable disjoint owned
tables stay allocated until inspection; no root is installed. The recursive
reference substitutes only owned-registry pointer resolution with argument
plumbing and uses the verified public Step delegate for private virtual-address
stepping. It does not construct or call a public recursive mapper.

The finite cases cover configured and bare links, repeated masks, encryption-only
leaves and siblings, zero-address children, nonpresent/huge stopping depths,
partial upward retirement, multiple child/root branches, budget exhaustion and
resumption, the canonical gap, the final page and skipped recursive slots.
Six Omega-only validation fixtures distinguish ordinary numeric/capture policy
from inputs constructible through public typed Rust APIs.

The current profile-sensitive constant fixture `cases/step-213.omg` passes a
21-source Omega check. Changing its expected first retirement from the decoded
frame to the encryption-marked number fails the unchanged success contract with
`cannot prove requires contract`, `1 == 0`.

The complete checked-interpreter run source-checks the selected package, then
executes 424 cursor/branch bodies and all six additional validation bodies. Every
positive returns 0; all nine changed-body controls return 1. No interpreter errors
or filesystem attempts occur. Maximum measured fuel is 1,218 under a 10,000,000
ceiling. The [verification record](../../../tools/ports/x86_64-encrypted-cleanup/checked-verification.json)
binds 464 selected source/harness/runner files, unchanged before and after the
601.569-second run. The controls change expected results inside actual authored
bodies while retaining their success requirement.

The [harness README](../../../tools/ports/x86_64-encrypted-cleanup/README.md) records
reproduction commands, the precise public/mirrored reference distinction and
body-control mechanics. Compiler revision is
`eaa7993a23623cd8fabf45350340479c5c9c7879`; native Omega execution, ABI/layout
measurement, universal proof and hardware/provider integration are not claimed.
The compiler binary SHA-256 is
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
