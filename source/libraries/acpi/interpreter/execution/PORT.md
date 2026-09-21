# Bounded AML integer method execution — partial ACPI-005

## Scope and status

`cathedral-acpi-execution` executes a bounded integer subset of real AML method
bytes using the sibling parser's retained `Span`, `Path`, `Namespace`, `Value`
and stable object IDs. It depends on `cathedral-acpi-aml` and the previously
verified `cathedral-acpi-interpreter` integer helpers. No production build imports
this package. This is a partial ACPI-005 milestone; packages, generic references,
fields and complete value semantics remain pending. It does not complete ACPI-006.

The public entry is `engine::run_method(input, length, unit, space, definitions, method_path,
arguments, count, size, budget)`. Input is a borrowed initialized `[u8;1024]`, with
an explicit logical length and retained unit identity. The caller supplies a
constructed namespace and retained method-definition observations, and selects
32/64-bit integer width from the definition block revision. No physical pointer or table mapping is accepted. Every method
span must belong to that unit and lie inside its initialized bytes. Multi-unit
source ownership/dispatch remains pending.

`definitions` is a borrowed `[MethodDefinition;64]` indexed by stable object ID.
The shared observation type now lives in `aml::model`.
Each present observation records that ID, original absolute method-name path
(the initial execution scope), flags and retained source span. The caller must capture it when declaring the
method and retain it independently of subsequent names/aliases/rebinding. The
executor checks ID, flags and the complete live Method span against it, validates
the scope/span, and uses its scope for initial and nested calls. Missing or
mismatched observations return MethodDefinition. Current entry paths and alias
flags never establish definition scope. These are inert caller-supplied semantic
observations, not unforgeable provenance or authority tokens. The
[single-source pipeline](../../pipeline/PORT.md) now captures them at declaration
and retains its own initialized source snapshot; the low-level entry still
requires caller-retained source and observations.
A caller that supplies fabricated definition observations is outside this API
contract; the executor cannot reconstruct historical scope from a bare namespace.

## Pin and licensing

Derivative translation of `rust-osdev/acpi` 6.1.1 at
`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`; original copyright 2018 Isaac Woods,
modified under `MIT OR Apache-2.0`. The parent [fixture/license audit](../../PORT.md#pin-licensing-and-exceptions),
[retained licenses](../../../../../licenses/rust-osdev/acpi/) and
[root notice](../../../../../THIRD_PARTY_NOTICES.md) were reviewed before copying.
Only crate-owned ordinary ASL tests and audited source are used. No firmware dump,
external uACPI/global-lock sample or unpinned corpus is copied.

[Inventory](inventory.json) snapshots every lexical anchor and pinned file hash
for `src/aml/mod.rs`, `object.rs` and `namespace.rs`. Generic upstream functions
remain pending when this package implements only integer/control suboperations.
No omission erases later interpreter work. The distinct helper/parser inventories
retain their own completed anchors; this narrow inventory does not override them.

## Source and behavior map

| Pinned concept | Omega implementation | Actual scope |
| --- | --- | --- |
| `MethodContext`, `new_from_method`, method invocation | `frames.omg`, `engine.omg`, `runtime_model.omg` | Seven argument/eight local slots, checked span/arity, nested integer calls, explicit return/fallthrough. |
| `OpInFlight`, `Argument`, operand contribution | `execution_model.omg`, `operands.omg`, `decode_execution.omg` | Finite expression stack, integer operands, up to two targets; generic operand/value forms pending. |
| `Block`, `BlockKind`, executor control arms | `control.omg`, `retire.omg` | If/Else, While, Break, Continue, Return, Noop. No table/scope/package declaration execution. |
| `ResolveBehaviour`, namespace lookup | `execution_names.omg`, parser APIs | Integer reads and existing name targets; lexical parent search and aliases. Generic references and declaration binding pending. |
| `do_binary_maths`, `do_unary_maths`, `do_logical_op`, BCD | `operator_specs.omg`, `retire.omg`, integer helper dependency | Actual integer op retirement and target writes, including Divide's two results. Generic Object conversions/comparisons pending. |
| `do_store`, `do_copy_object` | `targets.omg` | Integer/Uninitialized local/argument/existing named targets. Store mutates the existing ID and preserves object linkage; no insert/rebind. Other object/reference behavior pending. |

The decoder accepts Zero/One/Ones and byte/word/dword/qword constants, integer
name/argument/local values, method calls, Add/Subtract/Multiply/Divide/Mod,
shifts, bitwise operations, bit searches, logical/comparison operations,
Increment/Decrement, FromBCD/ToBCD, Store/CopyObject and the controls above.
Unsupported encodings return an explicit result; there is no successful stub.
`Revision`, conversion bytecodes, RefOf/DerefOf/Index, package/string/buffer
execution and fields remain unsupported even where separate pure helpers exist.

Primary references: [ACPI 6.6 AML grammar](https://uefi.org/specs/ACPI/6.6/20_AML_Specification.html)
and [ASL operator definitions](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html).
These define encoding and behavior, not ABI or grant authority.

## Profile, effects and deviations

- Input capacity is 1024 initialized bytes; namespace capacity is 32 entries and
  64 objects inherited from the parser. This executor never grows/rebinds either.
- At most four active method frames, sixteen pending expressions and eight
  control blocks per frame. Caller fuel is 0–1024 execution turns shared by all
  calls/loops. Each active dispatch turn consumes one; bounded parsing/lookup
  work inside it is not charged as separate turns. Zero fuel returns WorkLimit;
  a request over the supported cap returns Capacity. Depth has its own outcome.
- Arguments are integer values; locals and unused arguments begin Uninitialized.
  Method fallthrough returns an uninitialized slot. A void call as a statement
  is permitted; using its result as an operand rejects. The pin internally has
  eight argument slots; this profile exposes the seven AML argument slots.
- Calls retain their source span and flags and require a matching definition
  observation for the original scope, including calls through aliases or names
  rebound with `namespace::bind`. An observation survives original-name removal
  or replacement because it belongs to the stable object ID. Serialized methods stop with
  UnresolvedSynchronization; no mutex, sync level or method lock is fabricated.
  Namespace declarations/local-name cleanup inside methods are not implemented.
- Frame span comparisons stage start/end/unit as typed scalar locals. A direct
  record-field ordering with a `u64::MAX` start produced the wrong rejection in
  the pinned evaluator; the canonical maximum-start/end cases guard this source
  form. No Omega source is changed. See the [related minimal comparison probe](../../../../../tools/ports/acpi/aml/fields/reproduce-field-comparison.py).
- Integer width, wrapping, bitwise Not, bit positions, shift/BCD failures follow
  the documented helper profile and primary corrections. Divide parses/stores
  remainder first, quotient second, following AML grammar; the pin assigns these
  targets in the opposite order. Integer CopyObject remains a narrow value copy.
- Existing aliases keep identity across Store. No object is reallocated. Slot
  writes preserve `Object.has_next/next`; arbitrary shared references and object
  type replacement are future work. Null targets discard arithmetic results;
  Store/CopyObject/Increment require a real target.
- A frame caches at most sixteen immutable byte offsets to resolved entry/object
  IDs. Hits recheck namespace/entry/object bounds and identity. Values are always
  reread. This is valid because bytes/scope and namespace bindings cannot change
  in this profile; a future declaration/rebinding executor must invalidate it.
- Namespace writes are **not transactional**. Earlier successful stores remain
  after a later error or fuel exhaustion. Fault offset is the current dispatch
  cursor, not a promise of the exact offending source token.
- If validates the adjacent Else envelope before selecting a branch. Truncated,
  malformed or out-of-scope controls reject. This bounded profile is not a full
  AML compatibility/error-code specification.

## Service boundary and integration

OperationRegion values produce UnresolvedRegion when consumed. Serialized calls
produce UnresolvedSynchronization. Known Timer/Sleep/Stall/Notify/synchronization,
Debug/Breakpoint/Fatal service operations stop with UnresolvedService before
operand evaluation; no live callback or pretend success is installed. Unknown
operations return UnsupportedOpcode. Field declarations/field descriptors remain
pending integration with the parser's independent field-metadata child package.

No field access, mapping, I/O, physical address conversion, scheduler, timer,
mutex/event, PCI configuration or firmware invocation exists here. The parent
[adapter specification](../../ADAPTER.md) requires independent grants and their
bounds/lifetimes before any future host service can run.

## Test translation and verification

The fixtures hand-encode AML using the primary grammar. They execute actual Omega
bodies, not a Python behavioral model. Existing namespaces/method spans are seeded
using the parser's public data model. This verifies that representation seam but
is not yet a loader-to-executor pipeline test, an iASL compilation or an external
interpreter differential test.

`logical_not.asl` supplies its complete MAIN body. `method.asl` supplies FOO and
its invocation/store; synthetic MAIN wraps the top-level tail. `incdec.asl` and
all four `while.asl` statement scenarios seed their Name declarations and execute
statements inside a synthetic method. Added return/read assertions inspect results.
They are translated scenarios, not execution of complete ASL definition blocks.
Case metadata distinguishes those from original malformed/boundary scenarios.

**Current constant-evaluator stage passed:** `check_frame_const.py` proves valid
observed method admission and rejection of a `u64::MAX` start through the real
`observed_frame`/`new_frame` bodies. Changing the expected rejection inside the
body produces `TEST_RESULT == 1` and the required failing contract. Compiler:
clean Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`, binary SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
The separate whole-bytecode const runner is optional; its earlier Return/Add and
While successes preceded the definition-observation refinement and are not a
fresh const proof of the final executor.

**Final checked-interpreter stage passed:** all 79 scenarios and 79 changed-body
controls pass on the fixed source, including both maximum span offsets,
cross-scope method aliases, original-name replacement/removal, a non-alias `bind`
entry and missing definition observations. The complete retained-package run took
280.577 seconds after the shared declaration-observation type move. The largest observed evaluator usage was 71,787 units;
every run reports zero filesystem operation attempts.
[Retained evidence](../../../../../tools/ports/acpi/interpreter/execution/verification.json)
binds current source/fixture hashes, all selected machine outcomes, the dependency
lock and exact runner identity. `verify_record.py` verifies this correspondence.

The Cathedral test harness uses the pinned manager's source inspection pipeline
and `checked_interpreter::interpret_entry` on the retained checked bodies. Its
separate 10,000,000 evaluator-step ceiling is test machinery; the AML executor's
caller fuel remains unchanged. This is a distinct stage from constant requires
proofs and native execution. Rust harness SHA-256:
`e6d0aee6b4dddbbf34a60cffbe8f158643cc5c4d100f9e20f0481f75f52890be`, built with
`rustc 1.100.0-nightly (a69a63265 2026-09-03)` from the clean Omega revision above.
The retained Cargo lock reproduces its dependency selection offline.

Omega native ABI/artifacts, iASL/external interpreter comparison, firmware/hardware
and production integration are not run. Full generic values/references, packages,
fields, conversions and dynamic namespace behavior,
multi-unit source lifetime management and external service policy remain pending.
