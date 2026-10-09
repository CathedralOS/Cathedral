# Direct object logical results

Status: all 346 checked behavior/control pairs and three constant-expression
pairs pass from final repository paths.

`object_logic::binary(input, length, unit, store, left, right, size, op)` and
`negate(input, length, unit, store, source, size)` consume already evaluated direct
Integer, String or Buffer IDs. `LogicalResult` is Failure(ConversionFailure) or
Integer(number); its default is Failure(InvalidState). Success is zero for false
or the selected 32/64-bit all-ones Integer for true. Results are detached values:
no canonical slot is allocated, no target is changed and no opcode is retired.

[ACPI 6.6 LAnd](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#land-logical-and),
[LOr](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#lor-logical-or) and
[LNot](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#lnot-logical-not) require
Integer operands. Both binary truth operands pass independently through the
existing primary implicit Integer adapter, left first. The helper admits the
complete right value even when the left value determines the truth result. This
is data admission over already evaluated IDs, not expression evaluation or a
claim about short-circuiting Method calls. Unary Not uses the same conversion.
The existing integer helpers construct the normalized Boolean result.

The six relations Equal, NotEqual, Less, LessEqual, Greater and GreaterEqual reuse
[direct object comparison](object-comparison.PORT.md): the left type selects the
right conversion. Integer order is unsigned and normalized; String and Buffer
order is lexical, with length breaking equal-prefix ties. Empty same-type byte
values compare, but cannot become an Integer. For example, String `0x12` becomes
zero under implicit hexadecimal-prefix conversion; a five-byte Buffer whose only
set byte is fifth is false under 32-bit truth conversion and true under 64-bit
conversion. This differs from explicit decimal/0x String conversion.

Canonical conversion validates complete Source/Owned storage, including String
encoding beyond any numeric prefix. Invalid IDs/counts and storage failures keep
their existing semantic errors and left-first precedence. Truth conversion never
uses the left-directed String/Buffer conversion used by relations. Existing
conversion limits remain: 64 object slots, 1024 source bytes and 256-byte backing,
nonzero-ASCII String admission, and fixed conversion output capacity. Unused
source metadata is irrelevant for Integer/Owned inputs. Reference, NameReference,
BufferField, Package, service and uninitialized objects are outside this direct
adapter; callers must implement evaluation policies separately.

The licensed source component is
[`Interpreter::do_logical_op`, pin 257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/mod.rs#L1972),
MIT OR Apache-2.0, copyright 2018 Isaac Woods. Exact license texts and notices
remain in licenses/rust-osdev/acpi and THIRD_PARTY_NOTICES.md. The pin restricts
some mixed types, uses its explicit numeric String policy, forces four-byte
Buffer truth conversion, compares Buffer length first, and constructs raw u64
all-ones results. These observations come from pinned source inspection; the
primary-policy helper does not claim equivalent behavior. No new public Rust
execution or private Rust mirror is claimed. Existing components retain their
separately recorded public observations.

The 346 independent Python expectations cover all nine basic-type pairings for
every binary operator and both widths; unary normalization; unsigned endpoints;
lexical Buffer counterexamples; full malformed String tails; conversion capacity;
all six reference kinds; invalid IDs/counts/owners; source bounds and unused source
metadata; full right admission despite a decisive left truth value; and default
failure. Every successful body compares the exact Integer, not only nonzeroness.
Controls alter that exact value or the expected failure case. Three representative
constant pairs cover 32-bit true, a mixed lexical relation and unary String
conversion. The whole upstream logical execution anchor remains pending because
operand evaluation and context effects are not implemented by this component.
