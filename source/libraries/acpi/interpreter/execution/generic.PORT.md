# Generic execution: initial preloaded-data profile

Status: reviewed and published bounded component; canonical replay passed.
No boot/runtime production integration, native execution, firmware access or
complete AML claim.

This component adapts `src/aml/mod.rs` at acpi
`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5` (MIT OR Apache-2.0), particularly
MethodContext/OpInFlight, argument evaluation, Return, do_copy_object and do_store.
Original copyright and licenses remain those recorded by the parent AML port.
ACPI 6.6 [method calling convention](https://uefi.org/specs/ACPI/6.6/05_ACPI_Software_Programming_Model.html#method-calling-convention) and [storing/copying rules](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#rules-for-storing-and-copying-objects) define the primary execution profile.
Public pin observations remain comparison evidence, including disagreements.

## Representation and lifetime

`Operand` is Uninitialized, Integer(number), or Object(stable ID). Canonical
Values remain in ObjectStore. This is execution metadata, not another object
model. `Binding` distinguishes uninitialized/scalar private values, shared incoming
argument identities, and materialized writable cells. Integer intermediates remain
inline; the fixed Runtime no longer embeds large canonical Value unions. Owned String/Buffer values are admitted only through
their containing ObjectStore identity. Byte copies create/update a block owned
by the destination ID. Copying only an Owned locator is never a byte copy.

Frame creation and stable-ID entry allocate no object slots. Copying bytes, a
Package payload, or Reference metadata into an unmaterialized local/argument
allocates one private cell; later replacements preserve that cell ID and
object sibling links. Ordinary argument assignment replaces its binding, leaving
the caller's incoming object intact. Explicit-reference argument assignment
selects the referenced cell and preserves the argument reference. Reads and
writable target selection are separate operations. Read-only target classification
returns semantic NoWrite, Replace(binding), CopyTo(ID, retained binding),
StoreNamed(ID), or Failure cases. A shallow dispatcher applies the selected
action through the canonical copy or named-conversion kernel. These cases are inert policy decisions: they do not bypass
object bounds, destination admission, or source validation, and are not writable
capabilities.

The bridge computes prospective binding metadata from the classified action,
normalized operand and pre-mutation object count. These inputs must come from
the same store and selected integer width; the helper does not perform admission
or grant access. Mutation still runs the existing validation/copy kernel. Only
its scalar success outcome permits publication of the prospective binding.
Failed mutation publishes no binding. The metadata mapping is checked against
actual `copied_binding` results for scalar/identity normalization, opaque versus
transparent references, private cells, bytes and resource failures.

All IDs are relative to the supplied Program/ObjectStore. They grant no authority
and carry no cross-program provenance claim. The affine Program retains one
immutable initialized source array and its mutable ObjectStore. Allocated cells
are not reclaimed/reused in this initial bounded profile, so long-lived repeated
calls can exhaust the 64-object arena. Garbage collection, generation reuse and
general dynamic-declaration teardown are pending work.

## Scope

Generic transport, calls, Return, and CopyObject support preloaded Integer,
String, Buffer, Package and admitted Reference data. Local/Arg Store replacement
uses the same data-copy mechanics. Named Store now composes the canonical basic-data conversion kernels through
[the named Store integration](named-store.PORT.md). Integer/String/Buffer targets
retain their type; Uninitialized destinations use the existing copy path.
The bounded Buffer exclusions remain explicit in that port record. CopyObject may
replace an ordinary named data object's type, while keeping its identity.
Method, permanent field and service destinations reject. MethodDefinition
observations remain untouched; this component cannot make a copied method
executable.

String/Buffer copy is deep at the byte payload. Package copy retains the existing
linked child IDs. That shallow package policy matches the pin's Object clone but
is not advertised as a complete ACPI deep package-copy implementation. Package
admission validates the complete advertised chain before accepting it; values
inside children remain inert and are not eagerly evaluated.

RefOf/DerefOf/Index bytecode, dynamic String/Buffer/Package literals, method-local
Name declarations, general operand conversion, conversion opcodes, services,
region accesses and synchronization remain outside this initial dispatch slice.
Existing lexical NameReference resolution is not confused with object-reference
unwrapping; mixed resolution is a separately owned helper.

The existing `Slot` type is retained solely as an integer result projection for
legacy consumers. Generic execution uses Operand/Binding exclusively.
`ExecutionResult.has_value` and `.operand` represent generic returns; `.value`
is meaningful as the integer projection. No-return remains distinguishable from
an uninitialized value used as an operand.

## Entry and result contracts

`run_method` preserves canonical Value arguments through a transactional entry
adapter outside Runtime. Method/source/count/fuel admission happens first. The
adapter validates every argument against the original store, stages any new
private slots in a copy, and publishes only after all arguments succeed. Integer
arguments allocate nothing; non-Integer values without existing identity consume
one stable slot each. Transparent references preserve their original selected ID,
including intermediate wrappers, after validating the resolved payload.
`run_operands` accepts stable object IDs without entry allocation. A standalone
Owned byte locator is rejected because it does not identify its containing slot. The Program bridge supplies its retained
source, store and declaration-time method observations; it does not reconstruct
definition scope from a later alias lookup.

Arg source reads automatically dereference RefOf/Index values; Local source
reads preserve those references. This differs from writable Arg selection, which
retains the Arg reference binding while copying to its referent. The source-read
rule is exercised by the explicit-reference Store/CopyObject call cases.

A generic return preserves its selected object identity until consumed, except
that Integer carriers are normalized to the selected four/eight-byte width.
Incoming objects are shared without eager cloning. Local and ordinary Arg writes
replace the frame binding; explicit-reference Arg writes copy into the selected
referent without a named-target conversion, for both Store and CopyObject.
The separately retained public pin observations show different plain-argument
alias behavior and are not treated as the primary calling convention.

## Failure and resource policy

ObjectStore: 64 stable object slots, 32 namespace entries, 256 initialized bytes
per owned byte block. Existing execution bounds remain four frames, eight blocks,
16 pending operations, seven arguments, eight locals and at most 1024 caller-fuel
turns. Reference walks and package validation inspect at most 64 objects; byte
operations inspect at most 256 bytes. These are Cathedral profile limits, not
ACPI maxima.

Each replacement stages source validation/bytes before committing destination
payload and backing. Integer-source writes use the same destination validation
and plain-payload publication as generic copying; the already-normalized scalar
requires no byte-source staging. Source/destination self-copy is supported. Failure does not
undo earlier AML statements. No general execution transaction or rollback is
claimed. External argument failure leaves the entire original store unchanged and starts
no execution frame. Once entry succeeds its private argument slots remain allocated
under the same no-reuse policy, including after a later AML execution error.
Incomplete source units, invalid owner IDs, dangling references,
malformed package chains, unsupported targets and capacity exhaustion reject.

## Source and behavior map

The inventory retains pending dispositions for complete upstream functions;
these are partial algorithm and representation mappings for the stated profile.

| Pinned source anchor | Cathedral component | Boundary |
| --- | --- | --- |
| `mod.rs` MethodContext / OpInFlight / Argument | `execution_model.omg`, `runtime_model.omg`, `frames.omg`, `operands.omg` | Scalar/ID transport, bounded frames and call arguments |
| `mod.rs` evaluate / method argument and Return dispatch | `engine.omg`, `decode_execution.omg`, `retire.omg`, `external_arguments.omg` | Preloaded data, validated definition observations, transactional external admission |
| `mod.rs` do_store / do_copy_object | `generic_target_values.omg`, `generic_targets.omg`, `generic_target_bridge.omg`, `generic_binding_plan.omg` | Primary Arg/Local rules, separate read/write selection, success-only binding publication |
| `object.rs` wrapped references and value cloning | `generic_values.omg` with canonical AML object/storage kernels | Transparent reads, explicit-reference identity, deep byte copies and shallow package payload copies |
| `namespace.rs` identity and lookup behavior | Existing canonical AML namespace and object arena | Reused stable IDs; no new namespace implementation or definition-provenance inference |

`integer_target_values.omg` and `integer_target_bridge.omg` specialize the
already-normalized integer source while sharing destination policy/publication.
`control.omg` and `pipeline/program.omg` adapt the existing control/owned-program
boundaries to canonical Operand and ObjectStore. None of these component mappings
claims complete upstream Store, CopyObject, generic conversion or interpreter
coverage.

## Verification

At checkpoint `c4a8b03`, the canonical source passed eight checked-interpreter receipts covering 259
scenario/control pairs: 55 generic, 79 original integer and 22 original pipeline
cases, plus 15 external-argument admission/rollback, six decoder failures, four
public-entry carrier/value assertions, fourteen complete Frame/ObjectStore
mutation/atomicity cases, and 64 prospective-binding parity cases. Counts overlap
by behavior and are not distinct specification coverage. One actual
constant-evaluator byte-copy positive and changed-body negative control also
passes.

`tools/ports/acpi/interpreter/generic-execution/manifest.json` binds canonical
execution root, complete recorded source snapshot, generated build text/hash,
fixture inputs and current receipts. Its checkpoint history verifier retains those exact committed inputs. The named
Store integration supplies fresh changed-source regression receipts. The original read-only verifier reproduces the
generic/integer/pipeline fixtures and const pair, verifies static focused
fixtures and exact selected/observed result names. The runner uses clean Omega
`eaa7993a23623cd8fabf45350340479c5c9c7879`; compiler SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`,
checked runner SHA-256
`6ae82dae55fb988207049b9d15dc224ac25740f4d918c7fef58899d42b5ab03c`.
Checked interpretation executes actual authored bodies with a ten-million-step
harness ceiling per selected case, separate from the executor's caller limits.
No native or hardware execution is claimed.

Earlier scratch bridge/retirement/entry receipts remain supplemental historical
evidence, not current canonical execution. The complete immutable scratch bundle
and prepublication canonical receipts with their exact historical recorded inputs
are retained under `generic-execution/history/`. Existing parent execution and
pipeline receipts remain historical; they were not rewritten to assert current
source coverage.

The exact pinned source inventory audits three files and 210 symbols. Seven
partial component mappings identify this translation's anchors; no complete
upstream function or whole generic execution coverage is claimed by that audit.
Public Rust comparison uses the root-owned `aml-public-execution` harness, with
trapped host callbacks and no ObjectToken forgery. Primary argument binding
isolation and expression results retain documented differences from the pin.
