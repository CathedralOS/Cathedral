# Canonical Buffer/String Index validation

This adds verification of the existing `byte_storage::make_byte_index` implementation
from the owned-byte storage milestone. There is no new production constructor.
The aggregate Index anchor remains pending for operand evaluation, optional target
Store, context contribution and retirement. Package Index construction remains a
separate published component.

[ACPI 6.6 §19.6.63](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#index-indexed-reference-to-member-object)
requires an indexed byte/character reference for Buffer/String sources. The canonical
helper resolves Named/Local/Arg wrappers through read_bytes, fully validates backing,
checks the unsigned index and reserves two fresh slots before any write. It appends
a BufferField at index×8 with8-bit length and a RefOf pointing to that field. The
ByteResult reports outcome, new reference ID and8-bit length; value remains0.
Repeated calls create distinct field/reference identities over the same backing.

Both slots must exist before publication. One remaining slot returns Capacity and
leaves every store byte unchanged. Success preserves all old objects and every
namespace entry, and retains the entire byte arena including inactive blocks at the
new IDs. The two new objects have cleared sibling links. Numeric namespace entry
counts are irrelevant to this object-only allocation and are intentionally neither
validated nor modified. Existing canonical semantics are preserved.

The117 complete-store behavior/control pairs cover Source/Owned Buffer/String,
empty and256-byte extents, maximum counts/IDs/index values, late String encoding
errors before index or allocation failures, transparent reference identity/cycles,
opaque reference rejection, two-slot success, one-slot failure, repeated allocation
and exhaustion. All64 object payloads, links and byte blocks plus32 namespace entries
and all initialized path segments are compared. Controls change field metadata,
allocation counts, failure outcome, unrelated links, namespace tails or byte tails.
Three constant pairs check all payload/namespace metadata and first/last block bytes
within the constant budget; full checked execution compares all bytes.

Forty-four actual public pinned Interpreter observations invoke Index twice each.
Fourteen successes retain backing pointer identity, exact field geometry, RefOf
kind and fresh field/wrapper identities; thirty errors are IndexOutOfBounds.
All service callbacks are trapped except the inert constructor mutex. There are
no fabricated tokens, private calls, field reads/writes or hardware observations.
The public record is separate evidence of pin behavior, not Omega execution.

This corresponds to rust-osdev/acpi do_index, src/aml/mod.rs:2283,
revision257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5, copyright2018 Isaac Woods,
MIT OR Apache-2.0. Existing repository notices apply. The bounded capacities and
nonzero ASCII String policy are described in byte-storage.PORT.md and are not ACPI
maxima. Reproduce with tools/ports/acpi/aml/byte-index/README.md.
