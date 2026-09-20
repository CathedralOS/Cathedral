# Owner questions

Only unresolved owner-level Cathedral semantic, compatibility, architecture,
or trust decisions belong here. Settled decisions live in the
[specification](wiki/spec/README.md); implementation work and deliberately
deferred research do not.

Before adding a question:

1. Name the concrete product or implementation requirement that is blocked.
2. Cite the specification owners and machine-readable contracts checked and
   found silent.
3. Separate the owner choice from ordinary engineering or extraction work.
4. State the viable choices, their invariant and compatibility consequences,
   and a recommended answer.

A missing specification page is not by itself an owner question. Neither is an
implementation difficulty. Promote a gap only when a real customer forces a
choice that cannot safely be inferred from an accepted contract.

Questions are a mutable decision queue, not stable contract identities. Code,
tests, and settled documentation must cite the specification clause produced by
the answer rather than an owner-question number. Resolving a question updates
the specification and affected source contracts in the same change, then
removes the question. Git retains the discussion history.

Existing `Key Questions` and `Open Questions` in design chapters are design
inventory. They are not automatically owner questions under this narrower
standard.

## Open questions

### UEFI semantic-entry resource carriers

The physical-to-semantic UEFI adapter cannot be completed until Cathedral fixes
the exact resource carriers and qualifications accepted by its semantic boot
entry. The [entry-handoff specification](wiki/spec/boot/uefi_entry_handoff.md)
now rules out naked-geometry grants but deliberately leaves the complete schema
open.

The owner choice is whether loaded-image storage, bootstrap storage, and the
post-exit machine inventory are represented as transparent `Extent` carriers
with precise qualifications, opaque linear resource types, or authority
retained behind provider services. This choice determines which exact boundary
occurrence establishes each root and which split, transfer, return, and
firmware-exit obligations survive the adapter.

**Recommendation:** use a scoped, nonconstructible firmware-session carrier
while Boot Services are live; use qualified `Extent` values for ranges that are
actually delegated to Cathedral; and use a linear post-exit inventory to
account for ranges not yet delegated. The loaded image must receive only its
real attenuated executable/storage rights, not an unrestricted physical-memory
grant. Establish direct-entry parameters when there is only one consumer, while
retaining the adapter and external-receipt provenance across inlining.
