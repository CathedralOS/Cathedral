# Named Store in generic method execution

Status: tested through checked interpretation at checkpoint `a75f0cc`. All 255
behavior/control pairs pass with exact source/fixture receipts. ACPI-005 remains open. No native
execution, live region access, or production boot integration.

This slice composes the existing canonical named-value Store kernels into the
actual generic executor. It adapts `src/aml/mod.rs:2406 do_store` and
`src/aml/object.rs:317 replace_with_implicit_casting` from rust-osdev/acpi
`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, preserving MIT OR Apache-2.0 and
[retained notices](../../../../../THIRD_PARTY_NOTICES.md).

Primary behavior follows ACPI 6.6
[implicit result conversion](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#implicit-result-object-conversion)
and [storing/copying rules](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#rules-for-storing-and-copying-objects).
The pin's raw byte transmutation, Buffer resizing, and argument-binding
differences remain documented comparison evidence, not normative behavior.

## Composition

`TargetAction::StoreNamed` distinguishes fixed-type publication from `CopyTo`.
Integer, String and Buffer named destinations retain their existing types.
The generic and scalar bridges pass the active four/eight-byte IntegerSize into
application. Arithmetic result targets, including Add and both Divide outputs,
therefore use the same named conversion path as Store. CopyObject still replaces
the destination payload; Local/Arg and explicit-reference Arg targets retain their
existing copy/binding rules. An Uninitialized named destination retains the
existing first-assignment copy path.

Store expression publication follows [ACPI 6.6 §19.6.132](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#store-store-an-object): a successful named Store contributes the converted
stored data. Integer results are normalized inline; String/Buffer results retain
the destination's stable object identity under the existing no-eager-cloning
contract. This corrects the earlier source-contributing retirement path and is
an explicit primary-spec correction. The pinned `do_store` helper returns its
unconverted source, but its Store opcode path discards that helper result; this
is not evidence of a conforming Store expression. CopyObject keeps its existing
source result, and arithmetic operators keep their own numeric result contracts.

`named_store::store_named_operand` re-admits the current destination, then
selects canonical `store_integer` or `store_value`. Integer intermediates allocate
no object. Transparent Named/Local/Arg source carriers resolve under the existing
64-inspection bound. Explicit RefOf/Index sources keep their reference identity
and fail basic-data conversion. This does not add reference bytecodes or live
field evaluation. Namespace aliases already select stable object IDs; no second
namespace or object representation is added.

The adapter preserves all nine ConversionFailure distinctions. Five additional
ExecutionOutcome cases represent Bounds, Empty, Encoding, Overflow and
ReferenceCycle. The other four map directly to existing matching outcomes.
Destination-first failure precedence applies at named application. The surrounding
executor may inspect/normalize a source earlier, and its existing decode errors
remain unchanged.

Publication still belongs to the canonical atomic Store kernel. Failed application
changes neither ObjectStore nor frame bindings. Success preserves the destination
ID, sibling links, namespace metadata and every unrelated object. Existing source
snapshots and byte backing are checked before publication, including self-Store.
This is per-write atomicity; earlier AML statements and a successful first Divide
write are not rolled back by a later failure.

## Explicit remaining work

The original checkpoint admitted positive Buffer extents and Integer/nonempty
String sources. The subsequent [equal-extent Buffer slice](../../aml/named-buffer-store.PORT.md)
adds same-length Buffer copying, including zero. Unequal Buffer extents need the
narrow compatibility decision documented there. The [zero-length destination extension](../../aml/zero-buffer-store.PORT.md)
admits Integer and fully validated String conversion into empty Buffers. Empty
String conversion into a positive-length Buffer remains a separate, documented
precedence decision.
Other source conversions, dynamic literals, Field/BufferField target dispatch,
RefOf/DerefOf/Index bytecodes and complete package cloning remain ordinary
implementation work. The bounded Program resource profile is recorded separately
in [ACPI-006 evidence](../../pipeline/resource-limits.PORT.md). No aggregate source anchor or task checkbox
is promoted to complete by this integration.

## Verification

The new fixture driver lives under
`tools/ports/acpi/interpreter/named-store-execution/`. It runs synthetic AML
through the actual loader, Program and method executor, then separately checks
complete ObjectStore and Frame preservation at the write bridges. Positive/control
pairs change expected bytes, outcomes, numbers, or an inactive frame cache slot;
the production computation is unchanged. Historical public Rust observations
remain those of the canonical named-value Store and generic execution ports.
This slice adds no new claim of public Rust execution.

Earlier generic receipts remain source-bound to checkpoint `c4a8b03`; run
`generic-execution/history/verify_checkpoint.py` for their historical integrity.
They are not relabeled as current-source execution.

The canonical-checkout checkpoint passed 44 new bytecode pairs, 30 complete
Frame/ObjectStore bridge pairs, 25 focused target-dispatch pairs, 55 retained
generic pairs, 79 retained integer pairs and 22 retained pipeline pairs: 255
positive scenarios plus 255 negative controls. All six receipts bind unchanged
source hashes and the same audited Omega `eaa7993` checked runner. The bytecode
and bridge receipts additionally reproduce exact generated source, build text
and selected entry points with `check.py --verify`. These are checked-interpreter
results, not constant evaluation, native execution or hardware results.

Checkpoint `a75f0cc` retains the exact tested inputs before the independent
upstream BankField/IndexField package was merged. Subsequent receipts must retain
their own source identity; these six results are not relabeled as execution of
later code. Use `tools/ports/acpi/interpreter/named-store-execution/history/verify_checkpoint.py`
with `--source-ref a75f0cc` to reproduce their historical integrity.

The subsequent integrated checkpoint passed 58 bytecode and 38 complete-state
bridge pairs with equal/zero-length Buffer support and Program graph validation
enabled. Both new receipts passed exact input/body/build/selection verification.
They add explicit destination identity checks for named String/Buffer results,
equal/self/empty copies, zero-length conversion and malformed-source preservation.
The original six receipts remain bound to `a75f0cc`.
