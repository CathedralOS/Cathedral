# Bounded AML field declaration metadata

Status: **tested**. All 18 original syntax bodies and mutations pass through
current forwarding entry points, alongside a shared-type field read/write pair
and a FieldUnit execution-boundary pair. The
[owner-migration record](../../../../../tools/ports/acpi/aml/fields/migration-verification.json)
binds the current parent, child and affected consumer/execution source closure.

The original 18 constant-evaluator cases and 18 controls, with a 19-file source
check, retain their original hashes in the
[historical verification record](../../../../../tools/ports/acpi/aml/fields/verification.json).
The later [method-capture closure record](../../../../../tools/ports/acpi/aml/fields/checked-verification.json)
also predates this ownership move. Neither historical record is relabeled as
current-source execution.

The `cathedral-acpi-field-syntax` package retains ordinary forwarding machines
into the single AML-owned parser. Shared data types live at `aml::field_model`;
parser and flag bodies live at `aml::field_declarations`, `aml::field_elements`
and `aml::field_flags`. The [normal Field installer](../field-namespace.PORT.md)
consumes those descriptions inside the static loader. The parsing entry points
themselves install no namespace objects and perform no region/register access,
locking or bank selection. Index/Bank installation, full ACPI-004 and field
execution remain pending.

The pin is [rust-osdev/acpi 257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5](https://github.com/rust-osdev/acpi/tree/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5), copyright 2018 Isaac Woods, MIT OR Apache-2.0. Exact texts remain in `licenses/rust-osdev/acpi/LICENCE-MIT` and `LICENCE-APACHE`. Omega metadata and validation are Cathedral adaptations. `inventory.json` independently binds both relevant pinned Rust files, retaining full interpreter operations as pending. `tools/ports/acpi/aml/fields/provenance.json` hashes the licenses, manifest and four reviewed upstream test assets; those tests are metadata only, not copied firmware or handler scripts.

## Public boundary

`declarations::parse_declaration` consumes one Field, IndexField or BankField package at the supplied cursor. Its input is an initialized 1024-byte array, enclosing end, source unit ID, absolute declaration scope, cumulative bit limit and element budget. On success `next` is the package end. The declaration retains original region/index/data/bank names and scope, flags, source bounds and FieldList bounds. Named-field paths are resolved only lexically against the declaration scope; referenced objects remain unresolved.

`field_list::parse_list` independently parses a bounded list. Each of at most 32 named descriptors retains its absolute name, bit offset and length, source span, effective access metadata and current connection. Reserved elements advance the bit cursor; access/connection elements affect subsequent named fields without advancing it. Zero-length fields remain syntax metadata, without a claim about runtime validity. Duplicate names are preserved for later namespace policy rather than silently installed or merged.

The initial profile allows 1024 source bytes, the parent's 16-segment names, 32 named descriptors and at most 1024 caller-budgeted element turns. The caller supplies a bit limit; subtraction-based admission precedes addition, including when that limit is `u64` maximum. Package-length integers used as field bit lengths do not include or subtract their own encoding width. These are implementation limits, not ACPI maxima. Partial candidates on error must not be installed. Public records are ordinary constructible metadata; `Access` passed to `change_access` is expected to come from `decode_flags` or a successful preceding change, rather than act as an arbitrary-record validation certificate.

BankField accepts integer literals, including Ones and Revision, with the parent's 64-bit literal semantics. A dynamic BankValue returns `UnsupportedSyntax` and `Pending::BankTermArgRemainder`. That span covers **the entire unparsed package remainder**, including the unknown operand boundary and potential following flags/list. It is not a TermArg span. No field-list start, flags or descriptors are inferred after it. A future execution-aware parser must establish the operand boundary and value first.

ConnectField retains either its lexical NameString (with the declaration scope supplied by the enclosing result) or a complete bounded Buffer encoding span. Literal BufferSize also records the declared size and initializer span without materializing bytes. Dynamic BufferSize sets `size_known=false`; the whole Buffer package remains retained, and its inner expression/initializer boundary is deliberately unknown. Envelope retention permits parsing later list elements, but does not validate or execute that inner expression. Callers retain immutable bytes for each unique unit ID. These spans grant no region or execution authority.

## Pin and grammar review

The primary [ACPI 6.5 Errata A grammar, section 20.2.5.2](https://uefi.org/specs/ACPI/6.5_A/20_AML_Specification.html#named-objects-encoding), defines this declaration family and all five FieldElement forms. The following are explicit adaptations:

| Pinned behavior | Metadata parser behavior |
| --- | --- |
| NamedField uses general `namestring`, with a TODO noting NameSeg | Reads exactly one validated NameSeg |
| AccessField discards attributes and sets access bits from the entire byte | Retains attribute and upper attribute-mode bits, rejects reserved access bits, updates only the effective access nibble |
| ExtendedAccessField discards extended attribute/length | Preserves both, validates the three specified extended attribute codes |
| ConnectField panics through `todo!` | Retains name or bounded Buffer metadata; unresolved BufferSize stays explicitly opaque |
| FieldFlags access type returns an error; reserved update rule panics | Validates access, update and reserved flag encodings before use |
| Parser resolves live objects and inserts runtime FieldUnits | Produces uninstalled descriptions and retains unresolved names |
| BankValue executes/converts a TermArg before parsing the list | Literal subset succeeds; dynamic remainder is explicitly pending |

Lock/update rules are metadata. `nominal_width` preserves the pin's one-byte fallback for AnyAcc and BufferAcc; it is not a safe access plan for a particular region. Normal attribute bytes remain raw facts because validity depends on region/protocol context. Runtime register-width comparisons, bank/index operations, connection-resource decoding, global-lock handling, further field-kind namespace installation and actual field reads/writes remain pending. CreateField and CreateBit/Byte/Word/DWord/QWordField are separate executable syntax and remain outside this package.

## Validation and update audit

Run `python3 tools/ports/acpi/aml/fields/audit.py` and `python3 tools/ports/acpi/aml/fields/migration.py` for the current checked closure. `check.py` retains the separate constant-evaluator workflow. `fixtures.py` generates original cases with mutations inside expected behavior; each changed body must compute 1 and fail its zero-result contract. These are semantic evaluator checks, not native execution, AML handler execution or ABI observations.

For pin updates, recheck both full source hashes and every anchor, license/manifest deltas, field grammar and all adaptations above. Reclassify pending anchors only when their full operation is implemented. Review existing original assertions before regenerating and rerun positive and body-mutating cases against one unchanged parent/child source closure.
