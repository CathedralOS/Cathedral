# Description-aware Concatenate composition

Status: tested in isolated scratch: 472 checked behavior/control pairs, three
constant pairs and 52 actual public pinned opcode observations passed. This is a
bounded direct-object composition, not an AML opcode executor.

`object_concat_described::concatenate(input, length, unit, store, left, right,
size)` reuses `object_concat::Concatenated`: Failure(reason), String(length,
bytes[256]) or Buffer(length, bytes[256]). The default is Failure(InvalidState).
It accepts direct canonical object IDs and returns detached initialized bytes,
with zero unused tails. It never allocates an ID, changes source/store data,
resolves a reference, evaluates a method or field, installs a result, applies a
target conversion or advances execution.

[ACPI 6.6 §19.6.12, Tables 19.30 and 19.31](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#concatenate-concatenate-data)
provides the composition. Let D mean any represented description: Uninitialized,
Package, BufferField, Device, Event, Method, Mutex, OperationRegion,
PowerResource, Processor or ThermalZone. The descriptions use exact bracketed
Table 19.31 text through `object_descriptions::describe`.

| Left | Right | Detached result |
|---|---|---|
| Integer/String/Buffer | Integer/String/Buffer | Delegate the established `object_concat` API |
| String | D | String containing left text followed by the description |
| Buffer | D | Buffer containing left bytes followed by description ASCII and NUL |
| D | Integer/String/Buffer | String containing the description followed by primary implicit String conversion |
| D | D | String containing the two descriptions |
| Integer | D | UnsupportedValue under this local bounded profile |

Integer × D is a **local exclusion, not a normative ACPI rejection**. The prose
first describes other object types as Strings, while the Integer row of Table
19.30 explicitly lists only Integer/String/Buffer as its second operand. Applying
the preprocessing prose first would feed a leading `[` into Table 19.7's implicit
hexadecimal parser, which can yield zero after no valid digits. This adapter does
not silently settle that interpretation or inherit the pin's rejection as an
ACPI rule. References and NameReference are excluded on both sides. No fake
FieldUnit or Debug model variants are introduced.

Primary implicit String conversion means width-normalized 8/16-digit uppercase
hexadecimal Integer text without `0x`, and two uppercase hexadecimal digits per
Buffer byte separated by spaces. String identity validates the full nonzero ASCII
extent. This is distinct from explicit ToDecimalString/ToHexString formatting.
Buffer × D creates a new Buffer operand and therefore includes the description's
NUL; it does not use existing-destination Buffer assignment geometry. All-basic
pairs retain the existing helper's conversion and failure behavior.

Admission order is fixed: validate count/left ID, completely admit the left
value, validate right ID and completely convert/admit the right value, then
check the combined 256-byte capacity. The all-basic route repeats its bounded
pure left admission when it delegates to the established API. Malformed right
String data remains visible even when the eventual append would exceed capacity.
Unsigned count/ID guards use staged scalar values. Description dispatch ignores
its payload: poisoned Package metadata, Method spans and BufferField backing do
not cause evaluation or source access. Source metadata is inspected only by a
byte operand which requires it; labels and Integers ignore unused invalid source
metadata. Initialized nonlogical backing tails are never copied into results.

The 64-slot store, 1024-byte source snapshot, nonzero ASCII Strings and 256-byte
result limit are Cathedral's bounded profile, not ACPI maxima. Descriptor labels
are a direct object API: ordinary AML TermArg evaluation invokes named Methods
and reads named BufferFields before Concatenate, so their direct labels cannot
be equated with ordinary named-operand interpreter observations.

This component accompanies rust-osdev/acpi
[`Interpreter::do_concat` and nested `resolve_as_string`](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/mod.rs#L2171),
exact pin `257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, MIT OR Apache-2.0,
copyright 2018 Isaac Woods. Licenses and attribution remain under
`licenses/rust-osdev/acpi` and `THIRD_PARTY_NOTICES.md`. This is an Omega
composition using primary conversion rules: the pin formats mixed Integer text
in decimal without normalizing raw 64-bit Integer payloads in a 32-bit context,
formats Buffer text as lossy raw UTF-8, and rejects Buffer × Package/Device.
Those differences are retained explicitly in public opcode observations rather
than treated as agreement. No private Rust mirror is needed or claimed.

The aggregate Concatenate and formatter anchors remain pending: unresolved
references/context, unrepresented types, the excluded Integer × description
profile, target application and opcode retirement are outside this helper.

Validation corpus: 472 behavior/control pairs retain 304 applicable basic cases
and add 168 description cases. It covers every represented label on either side,
poisoned ignored metadata, both integer widths, source/owned bytes, empty and full
arrays, exact/overflow capacity including NUL, expansion before append, invalid
late String bytes before capacity failure, invalid left before right ID errors,
all six reference kinds, NameReference, MAX counts/IDs/lengths, same-object IDs,
last slots and dirty nonlogical tails. Successful expectations compare all 256
bytes; controls change an expected byte or error inside the assertion body.
Three representative constant pairs supplement actual checked execution.

The 52 actual public pinned opcode observations cover Package/Device with basic
and described operands in both widths. They retain the pin's successful Strings
and its Integer/Buffer conversion errors, including raw 64-bit decimal formatting
at width 32. These are separate observations of pin behavior, not assertions that
all results equal the primary helper. Each probe records exact synthetic AML,
zero forbidden service calls and the normal interpreter mutex construction.

Reproduce with `tools/ports/acpi/aml/object-concat-described/README.md`. Runtime
and constant receipts bind the resolved execution root, exact generated build
text/hash, the complete 20-file source/build closure, authored fixture bodies,
control bodies and compiler/runner binaries. The verifier rejects changed root
or build bindings and requires exact checked headers and execution counts.
