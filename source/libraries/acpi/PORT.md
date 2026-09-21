# ACPI source inventory and partition (ACPI-000)

## Scope and status

**Stage: inventoried, 2026-09-20.** This milestone covers the complete pinned
Rust corpus and test-scenario metadata. It does not claim an Omega translation,
compiled parser, interpreted AML, or hardware integration. The cumulative source map now records 589 translated table/topology/AML/helper anchors, two omitted Rust formatting hooks and 990
pending anchors. Inventory completion and translation completion are different
milestones. The bounded ACPI-001 slice is separately
[tested by Omega semantic evaluation](headers.PORT.md): 65 original scenarios
and three body-mutating negative controls pass; native and firmware execution
remain untested.

The implemented ACPI-001 slice covers initialized-byte RSDP/SDT headers,
checksums and RSDT/XSDT entries within its explicit 4096-byte input profile.
Fixed tables and topology have separate scope/verification reports in
[fixed.PORT.md](fixed.PORT.md) and [topology.PORT.md](topology.PORT.md).
Those slices pass 291 and 77 original Omega semantic scenarios respectively,
each with three body-mutating controls. The [AML helper slice](interpreter/PORT.md)
passes 150 integer/byte scenarios and three body controls. The [static AML layer](aml/PORT.md)
passes 27 semantic cases and 27 body controls for bounded parsing, declarations
and namespace behavior. [Field declaration metadata](aml/fields/PORT.md) adds
18 cases and 18 body controls for all three declaration families and five
FieldList forms. The [integer executor](interpreter/execution/PORT.md) passes
79 checked-interpreter pairs; [declaration capture and owned-source execution](pipeline/PORT.md)
adds 22 pipeline pairs and current parser/field regressions.
The supplementary [public interpreter probe](../../../tools/ports/acpi/aml-public-execution/README.md)
records 49 finite Rust loader/evaluator observations, including 24 successful
value/state agreements. Corresponding errors, pin differences and thirty
explicitly excluded fixture rows remain separately recorded.
The [resource parser](resources/PORT.md) adds 210 checked-interpreter pairs, four
constant-evaluation pairs and 187 actual public Rust observations, including
64 normalized supported-result agreements. It checks bounded envelopes, all
pin-supported descriptor families including GPIO/I²C, aggregate dispatch and
strict template termination/checksum. Other defined families retain explicit
unsupported spans. Runtime fields and complete object/context behavior remain
pending. No count implies full AML compatibility.
No production build root imports this directory. The libraries charter governs
pure algorithms; firmware/physical-address facts convey no mapping authority.

## Pin, licensing and exceptions

- [rust-osdev/acpi](https://github.com/rust-osdev/acpi), exact commit
  `257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, package version 6.1.1.
- Checkout: repository-root `reference_code/rust-osdev/acpi`.
- Manifest spells `MIT/Apache-2.0`; README explicitly grants the dual license.
  Preserve `MIT OR Apache-2.0` for derivative translations and modified-file
  notices. Root licenses retain `Copyright (c) 2018 Isaac Woods`.
- Exact pin/working bytes and existing retained license copies are checked by
  `tools/ports/acpi/inventory.py`; license/Cargo/README SHA-256 evidence is in
  `test-scenarios.json`.
- Retained texts: [licenses/rust-osdev/acpi](../../../licenses/rust-osdev/acpi/).
  Root notice: [THIRD_PARTY_NOTICES.md](../../../THIRD_PARTY_NOTICES.md).

The crate grant is not treated as a blanket grant for external fixtures:

| Input | Audit finding / handling |
| --- | --- |
| `tests/pc-bios_acpi-dsdt.asl` | Disassembled firmware with Intel ACPICA disassembler notice; firmware origin/license unresolved. Hash/scenario metadata only; no fixture import or translation. |
| `tests/uacpi_examples.rs` | Explicit adaptations from the uACPI README; exact original revision/license notices need separate review before copying those scenarios. |
| `tests/global_lock.rs::uacpi_global_lock_test` | Explicit adaptation of uACPI `tests/test-cases/global-lock.asl`; separate original provenance review required. Other tests in that file remain separately listed. |
| External uACPI suite | Not present or pinned by this checkout; no suite/source license or execution result inferred. |
| `tools/no-alloc-check` | README credits an external zulinx86 example; implementation would require that provenance review before translation. Only metadata inventoried. |
| Remaining crate-authored tests | Root grant reviewed; no separate external origin identified in the audit. This is not a claim to have proved provenance beyond the available source notices. |

Some tools lack their own Cargo license field; metadata inventory does not
copy those implementations or settle external-example licensing. Actual
translation must retain the origin of every imported dependency or fixture.

## Exhaustive maps

- [inventory.json](inventory.json): all **52 Rust files, 1,581 lexical anchors**
  under `src/`, `tests/`, `tools/`, including the excluded no-allocation tool.
  Every anchor has partition labels, queue tasks and an implementation status.
  The table/topology/static-AML/helper slices overlay only translated anchors; their narrow omissions do
  not remove future work from this complete corpus.
- [partitions.json](partitions.json): per-file SHA-256, categories, queue tasks,
  and observed allocation/concurrency/unsafe/host-I/O indicators.
- [test-scenarios.json](test-scenarios.json): **66 Rust tests, three ignored**,
  **19 ASL/AML assets**, hashes, named ASL methods and provenance exceptions.

The lexical index binds complete file hashes but does not expand macros or
pretend to be a Rust semantic parser. Symbol partition labels describe work;
a file can contain multiple kinds. Indicators are review aids, not inferred
proofs that a function is pure or safe.

## Partition and ownership

| Source surface | Work partition | Destination / constraints |
| --- | --- | --- |
| `rsdp.rs`, `sdt/mod.rs`, `sdt/*.rs` | Byte/table facts and table parsing | ACPI-001/002; initialized bounded bytes, explicit little endian and structural lengths; no reference cast into untrusted input |
| `lib.rs::AcpiTables`, `AcpiTable` | Discovery plus physical handler boundary | Split entry decoding/selection from mapping and table custody; addresses remain data |
| `address.rs` | GAS representation/validation plus `MappedGas` access | Retain region-space/access-width facts; do not translate raw volatile mapping into ambient access |
| `registers.rs` | Fixed-register operations and status/control bits | Facts can translate independently; read/write effects, event enabling and sleep control are later boundaries |
| `platform/{interrupt,numa,pci}.rs` | Platform topology | ACPI-003 inert CPU/controller/NUMA/PCI-region descriptions, without controller/MMIO authority |
| `platform/mod.rs` | Topology plus platform control | Split pure descriptions from event initialization, ACPI mode transition and AP wakeup mailbox writes |
| `aml/mod.rs` cursor/opcode/package/name parsing | AML syntax | ACPI-004 bounded deterministic input and namespace construction |
| `aml/mod.rs` interpreter/method operations | AML execution | ACPI-005 arithmetic/conversions/control flow/packages/fields; ACPI-006 work/recursion/output limits |
| `aml/namespace.rs` | Namespace plus compatibility policy | Separate lookup/normalization from `_OS`/`_OSI` claims; upstream OS impersonation is not a Cathedral default |
| `aml/object.rs` | Value algorithms plus shared mutable object ownership | Preserve semantics through explicit bounded storage/ownership; no unchecked translation of `Arc<UnsafeCell>` or the global `ObjectToken` convention |
| `aml/resource.rs`, `aml/pci_routing.rs` | Resource decoding and routing results | Syntax/byte facts separate from `_PRT` evaluation and later authority grants |
| `aml/op_region.rs`, `Handler` | Handler callbacks | ACPI-007 named physical/MMIO/port/PCI/time/wait/mutex/debug/fatal boundaries; none installed by parsing |
| `tools/` and test infrastructure | Host-only test adapters/compiler runners | Never target code. `iasl`, filesystem/process execution, real dump access and external test suites need separate harness treatment |

## Allocator, concurrency and policy assumptions

Default Cargo features enable `alloc` and `aml`. The no-allocation core can
find/enumerate raw tables, while platform extraction uses allocator-parametric
vectors and defaults to `Global`. AML adds vectors, strings, BTreeMap, Box,
Arc and SmallVec. A translation must select explicit capacities/allocators and
exhaustion outcomes; neither parser input nor firmware supplies allocation rights.

The interpreter coordinates namespace/region maps with spinlocks and a single
object mutation token, uses atomic event/global-lock state, and has explicit
unsafe Send/Sync assertions. `Handler` owns reentrant mutex operations and
infinite-wait timeout 0xffff. The pin documents that balanced method-exit mutex
release is not enforced. These assumptions require a Cathedral ownership/work
model; copying Rust marker traits cannot establish it.

`MAX_NAME_PATH_INDIRECTIONS = 8` bounds one name-resolution chain, not all AML
work. Loops, recursion, namespace growth, object allocation and returned data
require separate budgets, now supplied by the bounded Program profile
([ACPI-006 resource audit](pipeline/resource-limits.PORT.md)). The upstream test README names several
external tests that can hang; they must become timeout/exhaustion scenarios,
not be silently deleted or counted as passing.

Compatibility policy is separately inventoried: `_OS` identifies Windows NT,
`_OSI` answers a hardcoded capability/OS list, and `AcpiQuirks::ignore_xsdt`
changes table selection. Those decisions must remain visible to later Cathedral
adapter review rather than entering generic parsing as hidden defaults.

## Test partitions and evidence limits

The 66 Rust test records include unit bit-copy/conversion/resource tests,
namespace lookup, fields/index/bank access, operation-region mocks, global-lock
concurrency, packages/references, malformed opcode handling and host helper
checks. Three are explicitly ignored upstream: reference rebind semantics,
increment/decrement behavior, and buffer-field implicit conversion. Ignored
means unexecuted coverage, not a passing compatibility guarantee.

ASL scenarios cover buffers/strings, integer conversions, methods/scopes,
packages, field creation, control flow, thermal/power objects and multi-table
loading. Their content is not copied into this milestone. Future tests must
retain valid/malformed cases and actual expected observations; script exit zero
alone is insufficient. In particular, the upstream uACPI adapter returns success
for unsupported `resource-tests`; that path must never become positive coverage.
The no-alloc-check executable is build-only by design and must not be run.

No upstream Rust tests, ASL compiler, AML interpreter, Omega parser, simulator
or hardware checks were run for this inventory milestone. This is not an Omega
language blocker. Missing future implementation remains ordinary queue work.

## Verification

From Cathedral root:

```sh
python3 tools/ports/acpi/inventory.py --check
python3 tools/ports/inventory.py check --checkout reference_code/rust-osdev/acpi source/libraries/acpi/inventory.json
```

Both pass against the exact pin and current source bytes. The first checks the
partition/scenario metadata and license copies; the second checks Rust anchors
and hashes. `--require-transcribed` intentionally fails while implementation
is pending. No compiler binary/revision is attributed to this metadata audit.

[Canonical AML reference kernels](aml/object-references.PORT.md) add full and
transparent unwrap and six reference kinds, with stable-ID allocation, shallow
payload copy and complete package-chain validation. Their 160 checked pairs,
five const pairs and 116 actual public immutable Rust observations pass, alongside
146 existing parser/executor/pipeline/field regression pairs. Generic opcode
execution remains pending.

[Owned AML byte storage](aml/byte-storage.PORT.md) adds explicit Source/Owned
String and Buffer cases, a detached 64×256-byte arena, atomic byte copies and
field operations, stable Index identities and affine Program ownership. Its
138 checked pairs, 306 existing regression pairs, eight const pairs and 21
labelled Rust observations pass. The current regression receipts are under
`tools/ports/acpi/aml/owned-bytes/regressions/`; earlier model-dependent receipts
remain historical with their original hashes. Generic opcode dispatch remains
pending. The source-map count is unchanged: these are partial Object/interpreter
components and an already translated bit-copy dependency.

[Canonical object queries](aml/object-queries.PORT.md) add a single-budget mixed
name/reference resolver, numeric ObjectType and validated String/Buffer/Package
SizeOf components. The query suite passes 96 checked pairs, 306 existing
regression pairs, three const pairs and 44 public Rust observations (41 numeric agreements, two explicit errors and
one caught load panic). Namespace get/bind guards now stage IDs and counts as
u64 locals to reject malformed maximum IDs on the pinned evaluator. Generic
query opcode retirement and absent payload kinds remain pending.

[Detached PCI interrupt routing](pci_routing/PORT.md) translates the pinned routing
file's pure behavior into strict package decoding, first-match selection,
source-scoped link lookup, inert `_CRS` requests and physical SourceIndex resource
selection. The 117 checked pairs validate all 32 route slots, exact request paths
and all 4096 resource bytes. Shared unsigned-count guard fixes also pass fresh
96 query and 27 parser pairs; earlier receipts retain their original hashes. The separate public
Rust probe retains its 101 observations and documented pin differences. AML
method evaluation and interrupt installation remain caller boundaries. No live
access or completed generic AML routing service is implied.

[ASL textual names](aml/name-text.PORT.md) adds bounded parse/format conversion
into canonical paths, with lowercase folding, underscore padding and explicit
prefix/segment grammar. All 150 checked pairs, three const pairs and 84 actual
public Rust observations pass. The tests include full initialized outputs and
35 canonical guard regressions. Six new source anchors are translated; two Rust
formatting hooks are deliberately omitted. Namespace lookup and generic DerefOf
execution remain separate work.

[Method/device metadata](aml/object-metadata.PORT.md) translates ten pure
MethodFlags/DeviceStatus anchors. All 448 actual public Rust observations agree
with Omega across 56 checked pairs and one constant pair. Decoding retains each
status bit independently; synchronization, firmware validity and enumeration
policy remain caller responsibilities.

[Byte literal preflight](interpreter/execution/byte-literals.PORT.md) adds
StringPrefix and constant-size Buffer admission using the canonical parser and
byte storage. It enforces active-block bounds and source-unit identity, normalizes
BufferSize to the frame integer width, and publishes no value or cursor on failure.
All 75 checked pairs, three constant pairs and 47 actual public Rust observations
are retained against the current used source closure. Dynamic BufferSize,
Package and generic executor dispatch remain pending; the aggregate source-map
counts are unchanged.

[Namespace-level removal](aml/namespace-removal.PORT.md) adds transactional
subtree entry removal while preserving a same-path object, external aliases and
stable object storage. Its 44 checked pairs, two constant pairs and 12 actual
public Rust observations pass, including full 32-entry and depth-16 cases.
Malformed flat namespaces fail before mutation; no-op requests preserve all slots.
Object reclamation and AML Unload remain separate lifecycle work.

[Normal Field access geometry](field_access/PORT.md) adds complete aligned
region-relative footprints, initialized chunk plans, read-shape metadata and
scalar extraction/update arithmetic. Its 451 checked pairs, three constant pairs
and 362 actual public synthetic-memory observations retain source-bound evidence.
Requested GlobalLock and Preserve-read requirements remain explicit; plans convey
no access permission. Runtime/provider integration remains pending, so aggregate
source-map counts are unchanged.

[Detached Bank/Index sequencing](field_protocol/PORT.md) adds independently
validated register geometry, aligned Index selectors and ordered logical
read/write actions. Its 204 checked pairs, three constant pairs and 60 public
Rust observations pass with exact retained inputs. The public comparison keeps
32 explicit differences; selector overflow rejects before a recipe and lock
requirements remain unmet. Payload transfers, namespace installation,
synchronization and provider/opcode execution remain pending. ACPI-005 and
aggregate source-map counts are unchanged.

[Implicit String-to-Integer conversion](interpreter/implicit-integer.PORT.md) adds
the primary Table 19.7 hexadecimal prefix rule, with 630 checked pairs and three
constant pairs. This pure helper does not change upstream anchor counts or finish
implicit operand/target conversion dispatch.

[Primary Integer/Buffer-to-String conversion](interpreter/implicit-strings.PORT.md)
adds fixed-width Integer text and spaced byte pairs, with 334 checked pairs and
three const pairs. Aggregate Store/conversion anchors remain pending.

[Canonical object conversions](aml/object-conversions.PORT.md) add direct
ObjectStore preflight with 207 checked pairs, three constant pairs, 100 actual
public Object calls and 66 actual explicit Interpreter observations. All four
aggregate source anchors remain pending: reference/context/target integration
and wider BufferField conversion are not claimed.

[Detached normal Field read assembly](field_values/PORT.md) recomputes geometry
and constructs complete Integer/Buffer results from supplied numeric words. All
301 checked pairs, three const pairs and 259 actual public Rust reads pass.
Outputs retain unmet lock requirements and exact zero tails. Region access and
object/evaluator integration remain pending; aggregate source counts are unchanged.

[Primary implicit conversion dispatch](aml/implicit-conversions.PORT.md) composes
the canonical helpers for all nine direct Integer/String/Buffer combinations.
All 239 checked pairs and three constant pairs pass. Results are detached and
fully initialized; existing-target extent policy, reference/Field resolution and
Store execution remain pending. Aggregate source counts are unchanged.

[Detached BufferField reads](aml/buffer-field-values.PORT.md) return Integer or
Buffer from canonical Source/Owned backing using the Definition Block's bit width.
All 281 checked pairs, three constant pairs and 136 actual public Object calls
pass. One source anchor is translated; strict complete-field bounds and the
pinned byte-versus-bit shape correction are documented. Runtime field dispatch
and conversion integration remain pending.

[Detached normal Field write assembly](field_writes/PORT.md) turns already-converted,
exactly field-sized bits into complete native-width numeric write records. All
454 checked pairs, three constant pairs and 126 actual public Rust probes pass.
Geometry, payload/count checks, Preserve input requirements and unmet lock metadata
are retained. Source conversion/repeated writes, provider access, Bank/Index and
Store/evaluator integration remain pending; aggregate source counts are unchanged.

[Selected normal Field write chunks](field_writes/chunks.PORT.md) expose one
native-word recipe while sharing the bulk extraction/merge implementation. The
separate source-bound corpus passes 175 chunk pairs and the 454 original bulk
pairs, with 126 public Rust observations replayed. This supports later per-chunk Preserve sequencing; provider execution and
Store retirement remain pending.

[Direct object comparisons](aml/object-comparison.PORT.md) compose primary
right-hand conversion with unsigned Integer and lexical String/Buffer ordering.
The 278 checked pairs and three constant pairs verify all type combinations and
complete source admission. Reference/field evaluation, logical truth operations
and runtime integration remain pending; aggregate source counts are unchanged.

[Field source sequencing](field_sources/PORT.md) prepares one normalized Integer payload,
ordered Buffer pieces, or individual String characters for a normal Field.
All 315 checked pairs, three constant pairs and 38 public Rust observations pass;
the observations retain the pin's single-pass and String-rejection differences.

Empty-source and terminator choices are documented primary-profile interpretations.
Repeated execution, provider/lock access and Store integration remain pending;
aggregate source counts are unchanged.

[Inline Integer Field payloads](field_sources/inline.PORT.md)
expose the shared Integer conversion without a temporary namespace object. All
168 scalar pairs and 315 original object-source pairs pass with exact source and
binary verification. Interpreter Field writes and provider integration remain pending.

[Direct basic-data Concatenate](aml/object-concat.PORT.md) composes all Integer/String/Buffer
pairings with primary right-hand conversion, little-endian integer encoding and
complete admission before combined capacity checks. All 314 checked pairs and
three constant pairs pass. Other-object descriptions, reference/field policies,
object installation and opcode retirement remain pending; aggregate counts are
unchanged.

[Atomic BufferField byte writes](aml/buffer-field-writes.PORT.md) stage already-converted payloads,
validate the full field and backing, and publish only after String encoding checks.
All 189 whole-store behavior/control pairs, three representative constant pairs
and 89 actual public Object method observations pass. One bounded source anchor
closes; source conversion, target handling and Store execution remain pending.

[Direct Buffer/String Mid](aml/object-mid.PORT.md) validates complete canonical backing,
preserves the source type and slices without overflowing index + requested length.
All 315 checked pairs and three constant pairs pass, including malformed tails
before empty selection and full zero output tails. Parameter evaluation,
reference policy, target writes and opcode retirement remain pending; aggregate
source counts are unchanged.

[Mid opcode execution](interpreter/execution/mid.PORT.md) adds scalar/byte operand conversion, independent owned
expression results and staged target publication. Nine focused retirement pairs
pass; its full 116 new and 199 retained regression pairs remain under checked
verification. The 64 public Rust observations
retain 33 value/state agreements and explicit pin differences. This remains
a partial ACPI-005 component.

[Direct Buffer ToString](aml/object-to-string.PORT.md) admits complete backing before selecting the
ASCII prefix ending at NUL or the requested maximum. All 195 checked pairs and
three constant pairs pass. The 44 public Rust observations retain 33 agreements
and 11 documented NUL/UTF-8 differences. Reference evaluation, target writes and
opcode execution remain pending; aggregate source counts are unchanged.

[Explicit numeric String composition](aml/object-numeric-strings.PORT.md) handles direct Integer, String
and Buffer sources for decimal/hexadecimal formatting. All 255 checked pairs,
three constant pairs and 102 actual public opcode observations pass; the public
record retains 100 String results and two pinned literal-construction panics.
Explicit formatting and width/capacity differences are documented. Operand
resolution, target writes and opcode retirement remain pending; counts are unchanged.

[Direct String-name lookup](aml/string-lookup.PORT.md) joins canonical String admission, textual
ASL name parsing and scoped namespace search, returning an object ID and path.
All 97 checked pairs and three constant pairs pass, including scope validation,
error precedence and all initialized path segments. Target evaluation, reference
policy and full DerefOf execution remain pending; aggregate counts are unchanged.

[Positive-extent Buffer preparation](aml/buffer-target-values.PORT.md) prepares Integer/nonempty String bytes for a caller-supplied
Buffer extent, preserving that extent through truncation and zero padding. All
204 checked pairs, three constant pairs and 54 public replacement observations
pass; the public record distinguishes seven agreements, 25 differences and 22
excluded-policy observations. Zero extent, empty String and Buffer sources remain
outside this profile. Target provenance, mutation and Store execution are pending;
aggregate source counts are unchanged.

[Direct logical results](aml/object-logic.PORT.md) compose primary Integer truth conversion and
left-directed relational comparison over canonical basic values. All 346 checked
pairs and three constant pairs pass, including exact 32/64-bit Boolean results,
full right admission and malformed-tail errors. Operand/reference evaluation and
context retirement remain pending; aggregate source counts are unchanged.

[Direct object descriptions](aml/object-descriptions.PORT.md) supply the eleven represented nonbasic
Concatenate labels without reading or validating object payloads. All 115 checked
pairs, three constant pairs and eleven static pinned-label audits pass; receipts
bind the final execution root and generated build text. General Concatenate
dispatch and opcode execution remain pending; aggregate counts are unchanged.

[Direct arithmetic results](aml/object-maths.PORT.md) admit canonical Integer/String/Buffer
operands before width-normalized mathematics. All 506 checked pairs and three
constant pairs pass, including separate quotient/remainder results and semantic
conversion, divide-by-zero and BCD failures. Target writes, operand/reference
evaluation and opcode retirement remain pending; aggregate counts are unchanged.

[Package Index construction](aml/package-index.PORT.md) validates the advertised member chain and
allocates one fresh RefOf wrapper preserving the selected element's identity.
All 65 complete-store checked pairs, three bounded constant pairs and 19 public
Index observations pass. Source evaluation, target Store and opcode retirement
remain pending; aggregate counts are unchanged.

[Direct BufferField source writes](aml/buffer-field-store.PORT.md) compose Integer/Buffer/String
admission with atomic backing updates. All 226 complete-store checked pairs,
three representative constant pairs and 116 public Store observations pass.
The public record retains ten width differences and 24 String-source panics;
shared source/backing identity is checked safely in Omega. Target/reference
policy and opcode retirement remain pending; aggregate counts are unchanged.

[Expanded canonical byte Index validation](aml/byte-index.PORT.md) checks the existing byte_storage constructor without adding a second implementation.
All 117 complete-store checked pairs, three bounded constant pairs and 44 actual
public Index observations pass, covering two-slot allocation, failure preservation,
transparent references and fresh field/reference identities. Source evaluation,
target Store and opcode retirement remain pending; source counts are unchanged.

[Description-aware Concatenate](aml/object-concat-described.PORT.md) composes the eleven represented nonbasic labels with canonical basic-data conversion.
All 472 checked pairs, three constant pairs and 52 actual public opcode observations
pass. Integer plus a description remains explicitly outside this bounded profile;
reference/field evaluation, target application and retirement remain pending.
Aggregate source counts are unchanged.

[Atomic named-value Store](aml/named-value-store.PORT.md) admits existing direct Integer/String/Buffer destinations, prepares canonical conversion,
and publishes only after success. All 305 whole-store checked pairs, three constant
pairs and 297 actual public Store observations pass. Destination identity and links
are retained; full-value byte storage is reset or replaced atomically. The bounded
Buffer exclusions remain partial; subsequent executor integration is described
below. These component receipts retain their original scope and source counts.

[Generic method execution](interpreter/execution/generic.PORT.md) transports preloaded Integer/String/Buffer/Package/reference data through methods,
Return, Store and CopyObject using stable ObjectStore IDs and private bindings.
At checkpoint `c4a8b03`, canonical replay passed 55 generic, 79 unchanged integer and 22 unchanged pipeline
pairs, plus 103 boundary pairs and one constant-evaluator pair. Argument writes
preserve the primary binding rules; byte copies own their backing, while Package
children remain explicitly shallow. Dynamic literals, conversion opcodes, general
reference/field evaluation and services remain pending. ACPI-005 stays partial;
aggregate source counts are unchanged.

[Scalar named Store](aml/named-value-store-scalar.PORT.md) adds current-store destination admission and an inline Integer entry without
a temporary object allocation. All 504 checked pairs and six constant pairs pass,
including the unchanged 305 object-source cases and complete-store failure
preservation. Existing target types and positive Buffer extents are retained.
Generic named-target routing is implemented in the subsequent
[executor integration](interpreter/execution/named-store.PORT.md);
the scalar receipts retain their original scope and source counts are unchanged.

[Named Store executor integration](interpreter/execution/named-store.PORT.md) composes the canonical conversion kernels into Store and arithmetic named targets,
preserving destination types and atomic failure behavior. Named Store expressions
now contribute converted stored data, following ACPI 6.6 §19.6.132. All 255
checked behavior/control pairs pass: 44 bytecode, 30 complete-state bridge, 25
target-dispatch, 55 generic, 79 integer and 22 pipeline pairs. CopyObject and
Local/Arg bindings retain their existing contracts. Broader bytecodes, field
evaluation remain pending; ACPI-005 and aggregate source
anchors stay open.

[Equal-extent named Buffer Store](aml/named-buffer-store.PORT.md) admits Buffer sources when both logical extents match, including zero, self-store
and source padding. All 97 new and 504 retained complete-state behavior/control
pairs pass in the recorded isolated worktree, with 12 public Rust observations.
Those exact production and fixture hashes match the integrated source at
`ea4557d`; this is retained worktree evidence, not a canonical-path rerun.
Unequal Buffer extents need a narrow compatibility choice between resize and
fixed-extent behavior; this does not block other ACPI-005 implementation work.
Aggregate source anchors remain pending.

[Zero-length Buffer destination conversion](aml/zero-buffer-store.PORT.md) retains zero extent for Integer and fully admitted String sources. The 86
focused checked behavior/control pairs and six public Rust observations remain
bound to their isolated worktree. Their production and fixture hashes match the
integrated source at `3a74b52`. Positive-target empty String precedence and
unequal Buffer extents remain the two narrow compatibility decisions; other
ACPI-005 work continues.

[Returned object graph quotas](aml/result-graph.PORT.md) share one object/byte budget across the public Program result, validate nested
backing and preserve reference identity. The recorded isolated worktree passed
49 pure-kernel, 27 Program and three supplemental behavior/control pairs. Result
rejection preserves prior execution effects and exact step/fault diagnostics.
ACPI-006 is complete for the current single-source bounded Program profile;
[the resource audit](pipeline/resource-limits.PORT.md) records limits, evidence
and the distinct low-level engine contract. The combined canonical Store suites
passed all 58 bytecode and 38 complete-state bridge pairs with graph validation
and equal/zero-length Buffer support installed.

[ObjectType execution](interpreter/execution/object-type.PORT.md) inspects names,
Local/Arg bindings and Debug through the canonical metadata helper, including pure Scope
lookup and method noninvocation. Its isolated checkpoint `5bbaf37` passed 53 bytecode
and 12 complete-state decoder pairs, with 37 public Rust observations. [Explicit
ToInteger execution](interpreter/execution/to-integer.PORT.md) converts through the
canonical kernel and replaces the destination type while preserving target binding
rules. Its source-bound checkpoints passed 43 bytecode and 25 complete-state retirement
pairs. General reference constructors, field evaluation and the wider interpreter remain
open; these milestones do not complete ACPI-005.

The [canonical interaction suite](../../../tools/ports/acpi/interpreter/component-integration/README.md) passed 14 additional behavior/control pairs at both Integer widths with
normal Field installation, ObjectType and ToInteger combined. It covers type
replacement, Field metadata without region access, Field-target rejection,
Owned Local sources and seeded RefOf/Index argument redirection. Component
receipts retain their original source identities.
