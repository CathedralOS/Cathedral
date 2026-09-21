# Bounded byte literal preflight

Scratch proposal; not imported by any executor and not opcode completion. Intended
placement is `execution/byte_literals.omg`: this component applies the execution
integer-width policy to canonical AML parser results, without changing the parser.

`preflight_byte_literal(input, source_length, source_unit, frame_unit, frame_end,
at, size) -> ValueRead` accepts only StringPrefix (0x0d) and Buffer (0x11) with a
constant integer BufferSize. It returns existing canonical Source String/Buffer
values and a next offset. No Value variant, arena, object ID or namespace entry
is added. All inputs are immutable. A temporary canonical ObjectStore containing
one Source value delegates admission to existing `byte_storage::read_bytes`; its
slot identity never escapes and it does not consume the caller's 64-ID capacity.

Input precedence is fixed: invalid extent/unit relationship -> InvalidState;
empty active extent -> Truncated; other opcode -> UnsupportedSyntax; parser
error -> exact parser outcome/offset; nonadvancing/outside-block parse result ->
InvalidState; byte admission -> mapped outcome. Every failure returns default
Uninitialized Value and next=0. Source length must be <=1024, frame_end<=length,
at<=frame_end, and units must match. The future executor may map unit mismatch to
ExecutionOutcome::SourceUnit after its own admission; this helper uses canonical
Outcome::InvalidState. No parser access crosses the active frame_end even when
additional initialized source bytes exist in the snapshot.

The Buffer size is normalized to the selected 32/64-bit width before admission;
logical bytes are max(normalized size, initializer length), with zero padding and
capacity 256. String source excludes the NUL, consumes it in next, and accepts
ASCII1..127. All successful tails are initialized zero by the canonical reader.
ByteOutcome::Capacity maps to Outcome::Capacity; Encoding to BadEncoding; Bounds
and other unexpected byte failures to InvalidState. Valid parser-produced spans
cannot currently produce Bounds, and String encoding errors are caught earlier
by atom. These defensive mappings are not claimed as directly reached branches.
Malformed package envelopes follow parser BadEncoding; missing bytes/terminator
follow parser Truncated. Dynamic size expressions stop UnsupportedSyntax at the
size operand; the helper does not guess their end or interpret following bytes.

No operation budget is spent by this helper: work is bounded by1024 source bytes
and256 logical bytes. Proposed integration charges one ordinary decode step,
validates before contribution, then advances pc only after successful contribution.
No helper claims rollback of earlier retired instructions. Store/CopyObject must
use existing generic copy admission to install destination-owned bytes, and a
returned Source value remains relative to the same retained Program source.
Dynamic size, Package, Index/reference exposure, target mutation and runtime Name
publication remain separate executor work. The standalone tests do not claim any
of those future integration scenarios pass.

## Provenance and intentional differences

Pinned rust-osdev/acpi `257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`,
[src/aml/mod.rs](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/mod.rs)
StringPrefix1234-1244, Buffer decode1261-1269 and retire718-743. The inventory retains the exact file and source-anchor hashes. Copyright2018 Isaac Woods; MIT OR Apache-2.0, exact notices in
Cathedral licenses/rust-osdev/acpi and THIRD_PARTY_NOTICES.md. This is a bounded
modified translation, not an upstream API wrapper.

Primary basis: ACPI6.6 [§20.2.3 Data Objects Encoding](https://uefi.org/specs/ACPI/6.6/20_AML_Specification.html#data-objects-encoding),
[§20.2.5.4 Expression Opcodes Encoding](https://uefi.org/specs/ACPI/6.6/20_AML_Specification.html#expression-opcodes-encoding),
and [§19.6.10 Buffer](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#buffer-declare-buffer-object).
The pin accepts UTF8 and panics on invalid UTF8; the AML grammar requires ASCII.
Its Buffer copy panics when initializer length exceeds declared size. We preserve
the existing canonical spec correction: use the larger size, never truncate the
initializer. The pin's RevisionOp returns1, while existing canonical atom returns2;
this inherited interpreter revision choice is explicitly observed, not claimed as
value agreement. Capacity256 is this profile's bound, not an ACPI maximum.

Tests: tools/ports/acpi/interpreter/byte-literals. Receipts distinguish actual
checked-interpreter execution, representative constant proofs, and actual pinned
public Interpreter::load_table/evaluate observations. Public observations include
caught Rust panics/errors and valid differences; no private body mirrors are used.
Neither source checking nor these host proofs establish native execution, ABI or
hardware behavior. Full do_execute_method inventory remains pending.

Frozen scratch validation:75 actual Omega behavior/control pairs,3 representative
constant-expression pairs,47 actual pinned public observations. The receipt
verifier passes against this unchanged copied baseline. Shared names/namespace
guards changed independently afterward; final publication requires authorized
rebase and fresh dependency-bound verification. Current receipts do not attest
those newer shared dependency bytes.
