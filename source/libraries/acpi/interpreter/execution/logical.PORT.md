# Logical opcode execution

Status: **transcribed; complete checked Omega execution pending**. Nine existing logical
opcodes now dispatch through `logical_execution.omg` rather than the strict
Integer-only retirement path. Authored coverage comprises 221 Program scenarios
and 138 complete ObjectStore/Frame retirement scenarios, with 315 unchanged
Mid/integer/generic/pipeline/ToInteger regression scenarios selected separately.
Nine selected direct retirement pairs passed in the original focused run, with
maximum fuel 517,363 and every positive/control observed. Its eight AML pairs
failed compilation because a generated negative control exceeded `u64::MAX`;
no AML execution result is claimed. The local renderer now uses zero for those
all-ones controls, leaving production and positive bodies unchanged. The
[retained diagnostic](../../../../../tools/ports/acpi/interpreter/logical-execution/diagnostics/control-overflow/)
records the original mixed result and exact source scope. Corrected AML and
complete-corpus checked results remain pending.

The adapter modifies rust-osdev/acpi
[`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, `src/aml/mod.rs:1972`](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/mod.rs#L1972),
`do_logical_op`, copyright 2018 Isaac Woods, MIT OR Apache-2.0. Dispatch is in
`retire.omg`; the existing opcode shapes and Integer helpers are unchanged.
The [inventory](generic-inventory.json), [notices](../../../../../THIRD_PARTY_NOTICES.md)
and [licenses](../../../../../licenses/rust-osdev/acpi/) retain the source map.
Whole-function and aggregate interpreter anchors, including ACPI-005, remain
pending.

## Operand and result policy

[ACPI 6.6 LAnd](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#land-logical-and),
[LOr](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#lor-logical-or) and
[LNot](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#lnot-logical-not)
require Integer operands. For the six comparisons, the
[first operand selects the required second type](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#implicit-source-operand-conversion).
The execution adapter extends the primary policies documented by
[direct logical results](../../aml/object-logic.PORT.md) and
[direct comparison](../../aml/object-comparison.PORT.md), composing the existing
implicit conversion, Integer and lexical byte helpers for execution operands.

Inline Integers normalize to the active 32/64-bit width. Object operands follow
transparent Named/Local/Arg references through the existing bounded resolver.
An admitted left operand is converted completely before the right is resolved
and converted. Both And/Or operands undergo Integer conversion even when the
left value determines the Boolean result. Not never inspects its unused right
operand. AML expression evaluation still precedes retirement in normal argument
order; the fixtures include right-hand Method effects and a Method that replaces
the object already selected by the left operand.

Relations use unsigned Integer or lexical String/Buffer comparison. Buffer
length breaks an equal-prefix tie. String numeric conversion uses the existing
implicit hexadecimal-prefix rule; Integer-to-String comparison uses fixed-width
uppercase hexadecimal text. Integer-to-Buffer comparison uses four/eight
little-endian bytes. All canonical backing and complete String encoding must
be admitted before comparing values. Empty byte values compare to their own
type but cannot convert to Integer. Existing conversion capacity boundaries and
ASCII/NUL-free String rules remain in force.

The Boolean result is an inline Integer: zero or the active width's all-ones
mask. No temporary canonical object, result cell or backing block is allocated;
full valid object arenas remain usable. Conversion holds only shared store
borrows. The existing parent-contribution helper checks all bounds before its
sole successful mutation, so failure preserves the entire Frame and ObjectStore.
With no parent, successful conversion has no Frame effect. Earlier operand and
statement effects remain outside retirement and are preserved on later failure.

Explicit RefOf/Index carriers presented directly to the adapter are unsupported;
the decoder's existing argument-read dereferencing rules apply before this
boundary. Package, Field/BufferField and service payload conversion remains
unsupported here. The existing decoder continues to own Field/service admission;
this component supplies no deferred read, provider, synchronization or hardware
capability. Inline values do not consult unrelated source/store metadata.
The bounded 64-object, 256-byte and 1024-source-byte profile remains unchanged.

## Evidence and public comparison

[Owned tooling](../../../../../tools/ports/acpi/interpreter/logical-execution/README.md)
binds original/generated fixture bodies, exact selections, all source inputs,
build text and runner identity. The 138 retirement pairs compare every current
Store/Frame member, including inactive bytes and metadata. Their controls alter
the exact Boolean, expected error, untouched backing byte or inactive Frame
field. Cases cover all nine opcodes, both widths, directional conversion,
complete right admission, malformed left precedence, ignored Not-right values,
transparent and opaque carriers, full arenas and parent bounds. The 221 AML
cases additionally exercise decoding, nested operations and Method effects.
The retained 315 regression bodies are unchanged except module/receiver names.

The [public receipt](../../../../../tools/ports/acpi/interpreter/logical-execution/public/verification.json)
records 212 actual public Rust load/evaluate observations, with 61 exact
value/state agreements and 151 retained nonagreements. Those nonagreements contain
82 public errors for primary-success cases, 57 different Integer/state results,
and 12 primary Empty/Capacity cases whose raw observations are retained without
inventing an error equivalence. Nine canonical-edit cases are explicitly omitted.
Each observation records zero forbidden callbacks and one inert mutex. The
unchanged public driver, pinned source, exact AML, full primary rows, rebuilt
binary and raw output are bound and independently verified.

The observations expose the pin's narrower mixed-type admission, raw u64 true
results, four-byte Buffer truth rule, length-first Buffer ordering and different
numeric String policy. For example, the five-byte Buffer containing only a set
fifth byte yields false under the pin's 64-bit truth evaluation; the primary
eight-byte conversion yields true. A short Buffer starting with `ff` sorts before
a longer zero Buffer under the pin's length rule, whereas lexical order places
it after. The public probe is comparison evidence, not execution of this Omega
adapter. No new constant-evaluator, native Omega or hardware result is claimed.
