# Bounded ObjectType execution

This component dispatches AML `ObjectType` (`0x8e`) for SimpleName operands
(NameString, Local0–7, Arg0–6) and DebugObj. It calls the existing canonical
`aml::object_queries::object_type` for every object-backed value. There is no
second stored-value type mapping, new Value variant, object allocation, byte
read, field access or method invocation. Inline Integer and Uninitialized
bindings report 1 and 0; Debug reports 16 without invoking a debug service.

The primary references are ACPI 6.6
[§19.6.97 ObjectType and Table 19.36](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#objecttype-get-object-type)
and the [AML grammar](https://uefi.org/specs/ACPI/6.6/20_AML_Specification.html).
The source correspondence is rust-osdev/acpi
`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, `src/aml/mod.rs` ObjectType dispatch,
SuperName argument role and `object_type`. Original copyright 2018 Isaac Woods,
MIT OR Apache-2.0; the parent port retains the license texts and inventory.

## Operand and result contract

The dedicated decoder consumes the opcode and complete admitted operand as
one bounded execution token, then contributes the numeric Integer through the
existing operand path. This preserves the executor's fuel model: internal
bounded lookup work does not consume additional dispatch turns. The Integer
uses the active 32/64-bit width. No retirement operation or target is introduced.

A method name is inspected as metadata, including parameterized methods; it
does not consume argument bytes or require executable body provenance. Named
aliases preserve their canonical object identity. Object-backed Local/Arg
bindings and all existing canonical reference descriptor kinds use the shared
bounded resolver; lexical NameReference descriptors retain their own scope.
Malformed byte owners, package links or method body spans do not prevent kind
inspection because this operation never reads those payloads.

Pure Scope entries have no object ID and report 0. The execution lookup resolves
NameStrings with the existing path helpers and namespace `locate`, checking
each ancestor for an unprefixed single segment. A nearer typeless scope shadows
an outer object. This also supports running method scopes that have no separate
namespace level. Absolute and parent-prefixed paths retain exact lookup rules.
The root scope is encoded as RootChar/NullName; a bare NullName does not designate
an object and returns InvalidTarget in this profile.

The complete AML grammar also permits syntactic RefOf, DerefOf and Index
operands. Their bytecode execution remains excluded here and returns
UnsupportedOpcode. Preexisting reference descriptors can still be traversed to
inspect their base type. General TermArg expressions and literal operands are
not added to ObjectType's admitted grammar. Dynamic declarations, literals and
other query opcodes remain separate work.

## Failures and compatibility

Unknown names return MissingObject. Truncated or malformed names retain parser
failures; corrupt namespace/object bounds return InvalidState. Canonical mixed
reference cycles and exhausted resolution work retain ReferenceCycle and
WorkLimit. The helper inspects at most 64 objects; lexical ancestor search is
bounded by the existing 16-segment path capacity. These are Cathedral limits.

Inspection is read-only. The instruction cursor and pending result are published
only after successful inspection and operand contribution. Failure leaves the
entire supplied Frame and ObjectStore unchanged, including lookup-cache and
inactive slots. Earlier successful method instructions remain committed, as in
the existing executor; this is not whole-method rollback.

The canonical helper's legacy Processor code 12 is preserved. ACPI 6.6 marks
12 reserved; this is an explicit existing compatibility behavior, not a claim
that the current primary table assigns Processor that number. New canonical
type variants must extend the single helper; this adapter must not copy its map.
This isolated checkpoint starts from `fbc04ed`, before nominal FieldUnit
namespace integration. It does not claim an ObjectType-on-Field bytecode test;
the later FieldUnit mapping is owned by the canonical query helper.

The actual pinned public interpreter observations in
[`public/verification.json`](../../../../../tools/ports/acpi/interpreter/object-type-execution/public/verification.json)
cover 37 of the same encoded fixtures, with 27 successful value/state agreements
and zero forbidden host callbacks. Sixteen cases require direct canonical
metadata edits and are explicitly omitted from public-API comparison.

The pin errors on a pure Scope name, panics on the root scope, and resolves an
outer Integer when a nearer pure Scope shares its name. This implementation
follows the primary specification's Scope result 0. The pin also accepts literal
and bare Zero operands, supports the separately excluded RefOf/DerefOf/Index
bytecodes, and returns Uninitialized for the truncated operand. Unknown names
reject in both with distinct error representations. Both implementations retain
legacy Processor 12. These observations do not override the primary grammar.

## Verification boundary

The component fixtures use actual `prepare_program` / `run_program` bodies for
53 execution scenarios and changed-expectation controls. Twelve direct decoder
scenarios compare every Frame and ObjectStore field after success and failure,
with controls changing inactive cache metadata. Both integer widths, private
Local/Arg bindings, scope/alias lookup, parameterized method noninvocation,
existing references, malformed metadata, explicit exclusions and failure
publication are represented. The fixtures reuse the existing generic renderer
and exhaustive bridge comparator, rather than a second behavioral model.

Source-bound checked results and reproduction commands live in
[`object-type-execution`](../../../../../tools/ports/acpi/interpreter/object-type-execution/README.md).
Receipt snapshots belong to the exact recorded source, including their isolated
base; later integration requires a new run before claiming current-source
verification. Omega native ABI, firmware, hardware and production integration
are not exercised.
