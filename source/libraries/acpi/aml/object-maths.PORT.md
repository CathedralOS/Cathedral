# Direct object arithmetic results

Status: all 506 checked behavior/control pairs and three constant-expression
pairs pass from final repository paths.

`object_maths::binary(input, length, unit, store, left, right, size, op)` accepts the
existing `integers::Binary` selection. `unary(input, length, unit, store, source,
size, op)` accepts BitwiseNot, FindSetLeft, FindSetRight, FromBcd or ToBcd. Operands
are already evaluated direct Integer/String/Buffer object IDs. Every source is
admitted through the existing primary implicit Integer conversion before the
existing mathematical helper runs; binary admission is left first.

`MathResult` has Failure(MathFailure), Integer(number), and
Division(quotient, remainder) cases. `MathFailure` distinguishes
Conversion(ConversionFailure), DivideByZero, InvalidBcd and Overflow; default
initialization is Failure(Conversion(InvalidState)). Only Divide produces the
Division case, preserving both results. This data component does not choose,
resolve or write either destination. In particular it cannot implement the
[Divide operator's remainder and quotient stores](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#divide-integer-divide)
or their ordering. Divide-by-zero becomes a semantic failure for the caller to
handle; it does not trigger a host arithmetic fault or claim that an AML method
may resume after a fatal error.

The existing kernels supply wrapping Add/Subtract/Multiply; Divide/Mod; zero-filled
shifts; And/Nand/Or/Nor/Xor; bitwise complement; one-based highest/lowest set-bit
positions; and strict BCD conversion. Inputs and results normalize to the selected
32/64-bit width. Shifts at or above that width produce zero. FindSetLeft and
FindSetRight return zero for normalized zero. BCD decoding rejects nondecimal
nibbles and encoding rejects values needing more than 8/16 decimal digits; these
are the existing strict helper policies. No arithmetic is duplicated here.

Relevant primary definitions include
[implicit source conversion](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#implicit-source-operand-conversion),
[FindSetLeftBit](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#findsetleftbit-find-first-set-left-bit),
[FindSetRightBit](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#findsetrightbit-find-first-set-right-bit),
[bitwise Not](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#not-integer-bitwise-not),
[FromBCD](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#frombcd-convert-bcd-to-integer),
and [ToBCD](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#tobcd-convert-integer-to-bcd).
String inputs use implicit hexadecimal-prefix conversion, not explicit decimal/0x
parsing; Buffer inputs use the low-order 4/8 bytes. Empty byte inputs cannot become
Integers. Full String backing and encoding are validated even beyond the numeric
prefix, and complete right admission precedes arithmetic faults. The canonical
64-object, 1024-source-byte, 256-byte backing and nonzero-ASCII String limits remain.
Malformed counts/IDs, owners, source bounds and capacity preserve canonical errors.
Reference/NameReference, fields, Package and service values are not evaluated.

This is a modified component of rust-osdev/acpi, pin
257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5, MIT OR Apache-2.0, copyright 2018 Isaac Woods.
The mapped source anchors are `src/aml/mod.rs:1896` do_binary_maths, `:1931`
do_unary_maths, `:2233` do_from_bcd and `:2250` do_to_bcd. The
[pinned source](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/mod.rs#L1896)
uses different raw-width, shift, String conversion, unary Not/left-bit and BCD
behavior. Those differences remain documented with the
[existing integer helpers](../interpreter/PORT.md). This composition adds no
compatibility branch. Exact license texts and notices remain in
licenses/rust-osdev/acpi and THIRD_PARTY_NOTICES.md. No new public Rust execution
or private Rust mirror is claimed; reused components retain their own observations.

The 506 original fixtures cover all nine basic-source pairings for twelve binary
operations in both widths, unary source conversion, unsigned/wrapping boundaries,
zero divisors, normalized zero divisors, width-sized and maximum shifts, every
invalid BCD nibble position, BCD capacity endpoints, complete malformed tails,
all reference kinds, source/owner/count failures and default results. Success
checks require the exact semantic result case and value; division checks both
quotient and remainder. Controls independently alter expected quotient/remainder,
integer values or semantic failures. Independent Python arithmetic, bit operations,
and decimal/hexadecimal digit operations provide expectations. Three constant
pairs cover mixed-source division, BCD decoding and a width-sized shift.

No object allocation, alias resolution, namespace lookup, target mutation, method
execution or opcode retirement is performed. The four aggregate upstream methods
remain pending; this component supplies their direct data computation only.
