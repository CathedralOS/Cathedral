# Bounded static AML syntax and namespace layer

Current stage: **tested** for the static subset below. All 27 semantic cases and
27 body mutations pass. The historical constant-evaluator baseline source-checked
as 18 files. After declaration capture, all 27 original assertion bodies and
mutations pass through the checked interpreter; load-method/load-rollback constant proofs retain their original hashes as
historical evidence after the additive reference-model change. The verification record separates these stages.
[Recorded verification](../../../../tools/ports/acpi/aml/verification.json)
binds compiler, package and fixture hashes to those results.

This is a staged ACPI-004 implementation, **not a complete AML interpreter or a completed ACPI-004 checkbox**. It is an independent `cathedral-acpi-aml` package with no production boot import. The public entry `loader::load` consumes an already checked definition block's AML TermList. It does not find, map, checksum, dispatch, or execute firmware tables.

The source is translated and adapted from [rust-osdev/acpi](https://github.com/rust-osdev/acpi/tree/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml), exact revision `257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5` (crate 6.1.1). Upstream copyright is 2018 Isaac Woods, licensed MIT OR Apache-2.0; exact texts remain in `licenses/rust-osdev/acpi/LICENCE-MIT` and `LICENCE-APACHE`. Omega representations, resource profiles, transactional errors and original fixtures are Cathedral adaptations, not unmodified upstream code. `inventory.json` binds all six pinned AML source files and every scanner anchor; `tools/ports/acpi/aml/coverage.json` additionally maps all 113 opcode/range facts to the supported static subset or pending behavior; incomplete operations remain **pending**, not omitted or blocked.

## Implemented boundary

- Bounds-checked little-endian reads; extended and negated opcode tokenization; all pinned raw opcode/range facts; validated package envelopes and every encoded NameString form.
- Integer constants (including AML Revision), ASCII string spans, buffer descriptions, fixed packages and literal-count variable packages. Packages have an explicit parser stack, uninitialized padding, and linked object IDs. Package NameStrings retain their declaration scope and resolve lazily; forward references are not mistaken for method invocation.
- Static Name/Alias, Scope, Device, Processor, PowerResource, ThermalZone, Method, External, Mutex/Event and literal-address OperationRegion declarations. Methods retain flags and uninterpreted body spans. External declarations retain metadata without manufacturing a defined namespace object. Mutex, Event and OperationRegion are inert descriptions, with no runtime lock/event/region handler.
- Absolute and parent-prefixed resolution; nearest-ancestor lookup only for a single unqualified segment; separate level and object slots; stable alias identity across rebinding; bounded lazy reference chasing with exact repeated-object cycle detection.

Dynamic TermArgs, executable NameString invocation, fields/CreateField, If/While/control flow, methods/locals/arguments, conversions/operators, synchronization, region access, table loading/unloading, namespace traversal/removal, generic string-based names, and resource/PCI routing evaluation remain pending. Unsupported executable terms return `UnsupportedSyntax`; method bodies are deliberately retained without interpreting their contents. The parent inventory must not infer completion from this package's static subset.

## Resource and ownership policy

The initial profile is **1024 initialized input bytes, 16 path segments or parent prefixes, 32 namespace path entries including root, 64 object slots, eight package frames, eight scope frames, and 32 External records**. These are Cathedral resource limits, not ACPI maxima. Independent caller budgets bound loader turns, per-value parser turns (at most 1024 each), and reference hops (at most 64). A driver may consume remaining turns as terminal no-ops; these counters are not compiler logical-work or instruction meters.

`Capacity`, `Depth`, `WorkLimit`, `Truncated`, `BadEncoding`, `UnsupportedSyntax` and namespace errors are semantic outcomes. `load` restores the input namespace on any failure. Lower-level `parse_value` exposes a partial candidate arena with its error; callers must not install it as a successful result. Replacement allocates a fresh object ID; existing aliases retain the old ID. IDs are not reclaimed in this profile, so repeated replacement can exhaust the arena.

All records are ordinary initialized data, not validated authority types. Namespace inputs should come from `empty`, explicit seeding, or previous successful namespace operations. The public indexed `entry_at`/`object_at` kernels use physical-capacity guards and return defaults outside those capacities; consumers must use checked lookup and require an object ID below `object_count` before interpreting a slot. They are not namespace-validation certificates. `Span { unit, start, end }` retains byte offsets, never an address or live borrow. The caller must retain the matching immutable input and assign distinct unit IDs when combining definition blocks. A span, region space code or namespace ID grants no right to access hardware or invoke a handler. No layout policy or native ABI claim applies to these semantic cases.

`namespace::empty` creates only root. `predefined_scopes` also creates `_GPE`, `_SB_`, `_SI_`, `_PR_`, `_TZ_`. The pin's global-lock handler and `_OS`/`_OSI` impersonation choices are deliberately deferred to host policy. They are not silently initialized.

## Interpreter handoff

The public namespace functions take ordinary `Namespace` values. `get` performs exact absolute lookup; `search` resolves a name against its declaration scope and applies ancestor search only to a single unqualified segment. A successful `Lookup.object` identifies an existing object. Consumers must check `object < object_count` before reading it with `object_at`. `references::resolve_reference` follows package name references using their retained declaration scopes.

Declaration `insert` creates a fresh object identity, so it must not implement interpreter Store. The [integer method executor](../interpreter/execution/PORT.md) updates the existing checked slot with `object_set`, preserving the `Object.has_next`/`next` package linkage and alias identity. Method bodies retain only flags and a `Span`; the execution caller must supply matching immutable bytes, validate the unit and span bounds, and establish its own frame, argument and work policies. The executor accepts a separate method-definition observation table for original declaration scope; `load_with_definitions` now captures it at declaration and rolls back namespace and observations together. The [single-source pipeline](../pipeline/PORT.md) owns the source bytes and executes against that snapshot. This package supplies no unit-to-input registry, method-local teardown, runtime object conversion, or execution service.

## Pin behavior, validation differences and specification

The grammar review uses the [primary ACPI 6.5 Errata A AML grammar](https://uefi.org/specs/ACPI/6.5_A/20_AML_Specification.html), sections 20.2.2–20.2.5. Names and ancestor lookup were checked against [ACPI 6.6 section 5.3](https://uefi.org/specs/ACPI/6.6/05_ACPI_Software_Programming_Model.html#acpi-namespace). These sources define wire grammar and namespace rules; the bounded profile is separate.

| Behavior | Pinned evidence | This adaptation |
|---|---|---|
| PkgLength reserved bits / invalid envelope | `mod.rs::pkglength` ignores bits 4–5 in extended form; consumers subtract lengths | Rejects reserved bits, lengths shorter than their encoding and enclosing-bound crossings |
| MultiName count zero | `mod.rs::namestring` accepts the empty loop | Rejects zero count; NullName has its own encoding |
| Buffer initializer exceeds declared length | `mod.rs` Buffer retirement takes a shortened destination but the whole initializer source for `copy_from_slice` | Stores declared length and complete initializer span; [owned-byte materialization](byte-storage.PORT.md) uses max(declared size, initializer length), copies the complete initializer and zero pads, per ACPI 6.6 §19.6.10 |
| Extra Package elements | `mod.rs` Package retirement asserts end; VarPackage completion subtracts supplied count from declared count | Recoverable `BadEncoding` |
| Alias collision | `namespace.rs::create_alias` inserts before returning NameCollision | Checks collision before mutation |
| Rebinding names / opening levels | `namespace.rs::insert` replaces; `add_level` creates/reopens and preserves existing kind | Preserved as explicit pin-compatible namespace operations, even though ACPI 6.6 describes load-time name collisions as fatal |
| Scope declaration | `mod.rs` Scope resolves against current scope then calls `add_level` | Preserves that pin behavior, including root Scope; this is not a claim of full spec Scope interpreter semantics |
| Missing root object | `get_level_for_path` asserts that path is not root | Recoverable missing object |
| Strings / Mutex / External metadata | Pin strings use UTF-8 conversion; sync byte and External fields are permissive | ASCII strings, reserved mutex sync bits and External type/argument metadata checked before installation |
| Lazy reference limits | Pin `MAX_NAME_PATH_INDIRECTIONS` is 8 and exhaustion yields NameResolutionLoop | Caller-selected budget through 64; repeated IDs produce ReferenceCycle, budget exhaustion WorkLimit |
| Full namespace constructor | Pin installs host mutex and Windows identity behavior | Only explicit pure scope seeding; host decisions pending |

## Verification and update audit

Run `python3 tools/ports/acpi/aml/audit.py`, then `python3 tools/ports/acpi/aml/check.py --omega /tmp/cathedral-omega-eaa7993/release/omega`. `fixtures.py` regenerates independently authored bytes and expectations. Each case has a unique mutation **inside its expected behavior**, and the negative must compute failure 1 and be rejected by `requires result == 0`.

Validation is semantic evaluation, not native execution, Omega/Rust ABI agreement or execution of an AML interpreter. The harness records the compiler hash. Source hashes, license hashes and metadata-only upstream test provenance are audited separately. No firmware dump, `uacpi_examples.rs`, external global-lock example, or firmware-derived package fixture was copied.

When changing the pin: review license and manifest changes, hash and review every AML source/test delta, reclassify all changed anchors, review opcode ranges and each validation difference above, regenerate original fixtures only after semantic review, and rerun both audits and body controls. An inventory snapshot alone cannot authorize a completion claim.

## Canonical object references

[Reference kernels](object-references.PORT.md) add six semantic reference cases,
full and transparent unwrap, validated stable-ID allocation, sibling-preserving
payload copy and complete package-chain selection. The 160 checked pairs, five
const pairs and 116 public immutable Rust observations pass; all 146 existing
parser/executor/pipeline/field regression pairs also passed at that milestone.
Those receipts retain their original model hashes.

[Owned byte storage](byte-storage.PORT.md) now provides canonical Source/Owned
String and Buffer cases, byte copies, indexed fields and affine Program ownership.
Its 138 storage/composition pairs, 306 existing regression pairs and eight const
pairs passed at the storage milestone. [Object queries](object-queries.PORT.md)
add mixed name/reference resolution, numeric type reporting and validated size
queries, with fresh namespace-guard regression evidence. Generic opcode
integration remains pending.

[Canonical conversion preflight](object-conversions.PORT.md) adapts direct stored
Integer/String/Buffer values and bounded numeric BufferFields into detached
semantic results. Its 207 checked pairs, three constant pairs and 166 actual
public Rust observations cover storage validation, width normalization and named
String policies. Reference resolution, opcode retirement and target mutation
remain pending. Wider reads are supplied separately below. The standalone primary
implicit numeric and String formatting helpers do not change this adapter's
explicit conversion policy.

[Primary implicit conversion dispatch](implicit-conversions.PORT.md) selects all
nine Integer/String/Buffer conversions for direct canonical object IDs. Its 239
checked pairs and three constant pairs verify complete backing admission, exact
semantic result cases, width rules and zeroed output tails. These detached results
have no existing target extent; reference/Field resolution, named Buffer resizing,
object installation and opcode integration remain pending.

[Detached BufferField reads](buffer-field-values.PORT.md) complete the bounded
read_buffer_field data algorithm, with 281 checked pairs, three constant pairs
and 136 actual public Object observations. The helper validates complete backing
and field bounds and returns an Integer or a zero-tailed Buffer through 2048 bits.
The primary width test uses 32/64 bits, correcting the pin's byte-width comparison.
Outer reference resolution, opcode integration and the existing conversion
adapter's wider-field policy remain separate work.

[Direct object comparisons](object-comparison.PORT.md) select the right operand's
primary implicit conversion from the left operand's type, then compare normalized
Integers or complete String/Buffer bytes. All 278 checked pairs and three constant
pairs pass, including unsigned ordering and full storage validation. Reference
and field evaluation, logical truth operators and opcode retirement remain open.

[Direct basic-data Concatenate](object-concat.PORT.md) composes all Integer/String/Buffer
pairings with primary right-hand conversion, little-endian integer encoding and
complete admission before combined capacity checks. All 314 checked pairs and
three constant pairs pass. Other-object descriptions, reference/field policies,
object installation and opcode retirement remain pending; aggregate counts are
unchanged.

[Atomic BufferField byte writes](buffer-field-writes.PORT.md) stage already-converted payloads,
validate the full field and backing, and publish only after String encoding checks.
All 189 whole-store behavior/control pairs, three representative constant pairs
and 89 actual public Object method observations pass. One bounded source anchor
closes; source conversion, target handling and Store execution remain pending.

[Direct Buffer/String Mid](object-mid.PORT.md) validates complete canonical backing,
preserves the source type and slices without overflowing index + requested length.
All 315 checked pairs and three constant pairs pass, including malformed tails
before empty selection and full zero output tails. Parameter evaluation,
reference policy, target writes and opcode retirement remain pending; aggregate
source counts are unchanged.

[Direct Buffer ToString](object-to-string.PORT.md) admits complete backing before selecting the
ASCII prefix ending at NUL or the requested maximum. All 195 checked pairs and
three constant pairs pass. The 44 public Rust observations retain 33 agreements
and 11 documented NUL/UTF-8 differences. Reference evaluation, target writes and
opcode execution remain pending; aggregate source counts are unchanged.

[Explicit numeric String composition](object-numeric-strings.PORT.md) handles direct Integer, String
and Buffer sources for decimal/hexadecimal formatting. All 255 checked pairs,
three constant pairs and 102 actual public opcode observations pass; the public
record retains 100 String results and two pinned literal-construction panics.
Explicit formatting and width/capacity differences are documented. Operand
resolution, target writes and opcode retirement remain pending; counts are unchanged.

[Direct String-name lookup](string-lookup.PORT.md) joins canonical String admission, textual
ASL name parsing and scoped namespace search, returning an object ID and path.
All 97 checked pairs and three constant pairs pass, including scope validation,
error precedence and all initialized path segments. Target evaluation, reference
policy and full DerefOf execution remain pending; aggregate counts are unchanged.

[Positive-extent Buffer preparation](buffer-target-values.PORT.md) prepares Integer/nonempty String bytes for a caller-supplied
Buffer extent, preserving that extent through truncation and zero padding. All
204 checked pairs, three constant pairs and 54 public replacement observations
pass; the public record distinguishes seven agreements, 25 differences and 22
excluded-policy observations. Zero extent, empty String and Buffer sources remain
outside this profile. Target provenance, mutation and Store execution are pending;
aggregate source counts are unchanged.

[Direct logical results](object-logic.PORT.md) compose primary Integer truth conversion and
left-directed relational comparison over canonical basic values. All 346 checked
pairs and three constant pairs pass, including exact 32/64-bit Boolean results,
full right admission and malformed-tail errors. Operand/reference evaluation and
context retirement remain pending; aggregate source counts are unchanged.

[Direct object descriptions](object-descriptions.PORT.md) supply the eleven represented nonbasic
Concatenate labels without reading or validating object payloads. All 115 checked
pairs, three constant pairs and eleven static pinned-label audits pass; receipts
bind the final execution root and generated build text. General Concatenate
dispatch and opcode execution remain pending; aggregate counts are unchanged.

[Direct arithmetic results](object-maths.PORT.md) admit canonical Integer/String/Buffer
operands before width-normalized mathematics. All 506 checked pairs and three
constant pairs pass, including separate quotient/remainder results and semantic
conversion, divide-by-zero and BCD failures. Target writes, operand/reference
evaluation and opcode retirement remain pending; aggregate counts are unchanged.

[Package Index construction](package-index.PORT.md) validates the advertised member chain and
allocates one fresh RefOf wrapper preserving the selected element's identity.
All 65 complete-store checked pairs, three bounded constant pairs and 19 public
Index observations pass. Source evaluation, target Store and opcode retirement
remain pending; aggregate counts are unchanged.

[Direct BufferField source writes](buffer-field-store.PORT.md) compose Integer/Buffer/String
admission with atomic backing updates. All 226 complete-store checked pairs,
three representative constant pairs and 116 public Store observations pass.
The public record retains ten width differences and 24 String-source panics;
shared source/backing identity is checked safely in Omega. Target/reference
policy and opcode retirement remain pending; aggregate counts are unchanged.

[Expanded canonical byte Index validation](byte-index.PORT.md) checks the existing byte_storage constructor without adding a second implementation.
All 117 complete-store checked pairs, three bounded constant pairs and 44 actual
public Index observations pass, covering two-slot allocation, failure preservation,
transparent references and fresh field/reference identities. Source evaluation,
target Store and opcode retirement remain pending; source counts are unchanged.

[Description-aware Concatenate](object-concat-described.PORT.md) composes the eleven represented nonbasic labels with canonical basic-data conversion.
All 472 checked pairs, three constant pairs and 52 actual public opcode observations
pass. Integer plus a description remains explicitly outside this bounded profile;
reference/field evaluation, target application and retirement remain pending.
Aggregate source counts are unchanged.

[Atomic named-value Store](named-value-store.PORT.md) admits existing direct Integer/String/Buffer destinations, prepares canonical conversion,
and publishes only after success. All 305 whole-store checked pairs, three constant
pairs and 297 actual public Store observations pass. Destination identity and links
are retained; full-value byte storage is reset or replaced atomically. The bounded
Buffer exclusions remain partial; subsequent executor integration is described
below. These component receipts retain their original scope and source counts.

[Generic method execution](../interpreter/execution/generic.PORT.md) transports preloaded Integer/String/Buffer/Package/reference data through methods,
Return, Store and CopyObject using stable ObjectStore IDs and private bindings.
At checkpoint `c4a8b03`, canonical replay passed 55 generic, 79 unchanged integer and 22 unchanged pipeline
pairs, plus 103 boundary pairs and one constant-evaluator pair. Argument writes
preserve the primary binding rules; byte copies own their backing, while Package
children remain explicitly shallow. Dynamic literals, conversion opcodes, general
reference/field evaluation and services remain pending. ACPI-005 stays partial;
aggregate source counts are unchanged.

[Scalar named Store](named-value-store-scalar.PORT.md) adds current-store destination admission and an inline Integer entry without
a temporary object allocation. All 504 checked pairs and six constant pairs pass,
including the unchanged 305 object-source cases and complete-store failure
preservation. Existing target types and positive Buffer extents are retained.
Generic named-target routing is implemented in the subsequent
[executor integration](../interpreter/execution/named-store.PORT.md);
the scalar receipts retain their original scope and source counts are unchanged.

[Named Store executor integration](../interpreter/execution/named-store.PORT.md) composes the canonical conversion kernels into Store and arithmetic named targets,
preserving destination types and atomic failure behavior. Named Store expressions
now contribute converted stored data, following ACPI 6.6 §19.6.132. All 255
checked behavior/control pairs pass: 44 bytecode, 30 complete-state bridge, 25
target-dispatch, 55 generic, 79 integer and 22 pipeline pairs. CopyObject and
Local/Arg bindings retain their existing contracts. Broader bytecodes, field
evaluation and resource accounting remain pending; ACPI-005 and aggregate source
anchors stay open.

[Equal-extent named Buffer Store](named-buffer-store.PORT.md) admits Buffer sources when both logical extents match, including zero, self-store
and source padding. All 97 new and 504 retained complete-state behavior/control
pairs pass in the recorded isolated worktree, with 12 public Rust observations.
Those exact production and fixture hashes match the integrated source at
`ea4557d`; this is retained worktree evidence, not a canonical-path rerun.
Unequal Buffer extents need a narrow compatibility choice between resize and
fixed-extent behavior; this does not block other ACPI-005 implementation work.
Aggregate source anchors remain pending.

[Returned object graph quotas](result-graph.PORT.md) apply one shared object/byte budget at the public Program boundary. Nested byte
backing and Package member chains are validated without evaluating references,
resolving names or copying the graph. The 49 pure-kernel and 27 Program behavior/
control pairs pass in the recorded isolated worktree. Rejected results expose no
value while preserving prior execution effects and diagnostics. This is partial
ACPI-006 evidence; reclamation, deep copy accounting and other resource work
remain open. No upstream source anchor is promoted by this original composition.
