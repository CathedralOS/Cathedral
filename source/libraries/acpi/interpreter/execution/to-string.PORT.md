# ToString opcode execution

Status: **transcribed; checked Omega execution pending**. Opcode `0x9c` takes
two operands and one target. The authored corpus contains 66 Program scenarios
and 79 complete-store/frame retirement scenarios. Another 315 unchanged Mid,
integer, generic, pipeline and ToInteger scenarios are selected for regression
execution. Authored cases and public Rust observations are not Omega passes.

`to_string_execution.omg` adapts rust-osdev/acpi
[`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, `src/aml/mod.rs:2068`](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/mod.rs#L2068),
`do_to_string`, copyright 2018 Isaac Woods, MIT OR Apache-2.0. Operand shape and
dispatch live in `operator_specs.omg` and `retire.omg`. The
[inventory](generic-inventory.json), [notices](../../../../../THIRD_PARTY_NOTICES.md)
and [licenses](../../../../../licenses/rust-osdev/acpi/) retain attribution.
The complete source anchor, aggregate interpreter anchors and ACPI-005 remain
pending.

## Behavior and composition

[ACPI 6.6 §19.6.143](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#tostring-convert-buffer-to-string)
selects bytes up to Length or the first NUL; an empty Buffer produces an empty
String. The primary [source conversion rules](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#implicit-source-operand-conversion)
require Buffer and Integer operands. The
[explicit conversion rules](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#explicit-data-type-conversions)
replace an ordinary target's type with the result type.

The adapter admits and converts the complete source first, then converts Length,
then scans the selected prefix. Inline Integers become four/eight little-endian
bytes without allocating a temporary object. Canonical String-to-Buffer coercion
adds a NUL for a nonempty String, so a 256-byte String cannot fit the temporary
256-byte Buffer even when Length is zero. String Length operands use implicit
hexadecimal-prefix conversion, and scalar Length is normalized to the active
Integer width. Empty numeric source operands retain the existing Empty outcome.

Transparent Named/Local/Arg carriers use the existing 64-inspection resolver.
Explicit RefOf/Index carriers supplied directly to this conversion reject;
existing decoder/argument-read behavior still applies before retirement.
Canonical source backing, including owner and source-unit identity, must be
valid before a zero-length selection. A String source must satisfy its complete
encoding contract before Length conversion. A raw Buffer's selected bytes are
checked only after Length conversion. Result Strings use Cathedral's existing
ASCII profile: selected bytes 1 through 127; bytes after a NUL or Length do not
participate. Every inactive result byte is zero.

Success allocates one fresh owned String, including an empty result. Optional
target writes use the existing CopyObject bridge to replace an ordinary named
data object's type or update a Local/Arg binding. Argument references retain
their existing destination redirection. The expression contributes the fresh
result identity independently of its target. A new Local/Arg cell may require
one additional slot. Target identity is checked against the original store
before allocation; a dangling destination cannot become valid by naming the
new result slot.

Allocation, target mutation and parent contribution occur in one staged
ObjectStore/Frame. A later error publishes none of those retirement changes.
Earlier operand evaluation and AML statement effects remain outside that
boundary. Existing 64-object, 32-name, 256-byte, 1024-source-byte and frame/fuel
limits still apply. Null skips the target write. Debug, permanent Field and
region targets retain unresolved outcomes; BufferField and Method targets
remain unsupported by this execution path.

This component supplies no Field/BufferField source-read continuation, reference
constructor bytecode, service provider, synchronization or hardware access.
It remains outside production roots.

## Evidence and pin differences

[Owned tooling](../../../../../tools/ports/acpi/interpreter/to-string-execution/README.md)
binds exact authored and generated source, selections, dependencies, build text,
runner identity and observed outcomes. The 79 retirement pairs compare the whole
ObjectStore and Frame, including inactive bytes and metadata. Changed-expectation
controls must also execute. Cases cover both widths, source/Length coercion,
NUL/empty/extreme lengths, nested expressions, aliases, target type replacement,
fresh and stale identities, capacity, error precedence and failure preservation.
The six retained regression groups preserve their original generated bodies
apart from receiver names and separate module wrappers.

The [public Rust receipt](../../../../../tools/ports/acpi/interpreter/to-string-execution/public/verification.json)
contains 56 actual public load/evaluate observations: 16 exact value/state
agreements and 40 retained nonagreements, including negative-case observations.
Ten canonical-edit/Debug cases are explicitly omitted. The unchanged service-trap
driver records zero forbidden callbacks and one inert mutex per observation.
Receipt verification checks AML, fixture/source hashes and the rebuilt binary.

The pin requires exact Buffer/Integer operands where primary conversion accepts
other basic types. Its inclusive split retains NUL, its UTF-8 admission differs
from Cathedral's ASCII profile, and its Length handling differs at the 32-bit
wrap boundary. Its ordinary Store target path can preserve a Buffer destination
where this explicit conversion produces a String. Other negative cases retain
their raw errors rather than being counted as successful semantic comparisons.
No private Rust algorithm is copied into the public probe.

No checked Omega, constant-evaluator, native Omega or hardware result is claimed
for this opcode extension yet. Earlier direct conversion receipts remain scoped
to their unchanged helper implementations.
