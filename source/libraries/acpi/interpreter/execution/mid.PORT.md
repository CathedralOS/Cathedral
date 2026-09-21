# Mid opcode execution

Status: **tested in nine focused retirement pairs; full verification in progress**. This isolated extension
adds opcode `0x9e` with three operands and one target. The retained corpus has
74 Program scenarios and 42 complete-store/frame retirement scenarios, plus
199 unchanged integer, generic, pipeline and ToInteger regression scenarios.
The nine selected pairs passed with their changed-expectation controls; full
corpus coverage is not yet established.

`mid_execution.omg` adapts rust-osdev/acpi
[`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, `src/aml/mod.rs:2137`](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/mod.rs#L2137),
`do_mid`, copyright 2018 Isaac Woods, MIT OR Apache-2.0. `operator_specs.omg`
and `retire.omg` supply operand shape and retirement dispatch. The
[inventory](mid-inventory.json), [notices](../../../../../THIRD_PARTY_NOTICES.md)
and [licenses](../../../../../licenses/rust-osdev/acpi/) retain the source map.
Whole interpreter anchors and ACPI-005 remain pending.

## Behavior and composition

[ACPI 6.6 §19.6.86](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#mid-extract-portion-of-buffer-or-string)
defines zero-based slicing, empty out-of-range results and truncation at the
source end. Buffer/String inputs use the unchanged canonical
[`object_mid::extract`](../../aml/object-mid.PORT.md). All backing is checked
before slicing, including malformed tails on an empty request. Results retain
the source type, and every inactive byte is initialized to zero.

The primary [conversion rules](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#implicit-source-operand-conversion)
select Buffer for an Integer source and implicit Integer conversion for index
and length. The adapter normalizes scalar values to the active 32/64-bit size;
String numeric operands use the canonical implicit hexadecimal-prefix policy.
Transparent Named/Local/Arg carriers follow the existing 64-inspection resolver.
Explicit RefOf/Index sources remain unsupported. Index conversion precedes
length conversion, then source extraction, then target handling.

The result is one fresh, owned Buffer or String object. Optional target writes
use existing Store conversion and binding rules. A named destination retains its
fixed type; Local/Arg replacement and argument-reference redirection retain their
existing contracts. The expression contributes the independent sliced object,
including when the named target's converted value has another type. Null omits
the target write; Debug and region targets retain their unresolved outcomes.

Target classification and existing destination bounds are checked before the
fresh result allocation, so a stale identity cannot become newly valid.
One staged ObjectStore/Frame contains allocation, target conversion and parent
contribution. Failure publishes none of those changes, including exhausted
allocation and a malformed/full parent operation. Existing statements and operand
evaluation effects precede this retirement boundary and are retained. A successful
result consumes one object slot; a new Local/Arg cell may consume another. The
existing 64-object, 256-byte, method, fuel and result-graph quotas still apply.
No temporary source object is needed for inline Integer conversion.

This code supplies no live field provider, synchronization or hardware effects.
The synchronous decoder's existing source-admission boundaries remain in force;
a canonical BufferField helper does not by itself enable BufferField bytecode
reads. This component remains outside production roots.

## Evidence and pin differences

[Owned test tooling](../../../../../tools/ports/acpi/interpreter/mid-execution/README.md)
records source, generated packages, exact entries, runner identity and actual
observations. Changed-expectation controls cover complete Frame/ObjectStore
state in retirement tests, including inactive metadata and bytes. New cases
exercise nested results, self-targets, target conversion, implicit numeric
operands, both Integer widths, references, empty/extreme bounds, allocation and
failure preservation. Existing regression bodies remain unchanged except their
receiver names and the already established bounded assertion controls.

The retained [focused receipt](../../../../../tools/ports/acpi/interpreter/mid-execution/focused-verification.json)
binds 141 source/tool inputs and the exact runner binary. Its nine pairs cover
Null, named Integer conversion, fresh Local binding, argument RefOf redirection,
four originally invalid destination identities and full-parent failure. Complete
authored/dependency checking and execution passed in 617.596 seconds, with maximum
fuel 525,283. This selected run does not establish the entire 315-pair corpus.

The unchanged public Rust harness observed 64 encoded cases with zero forbidden
service calls: 33 exact value/state agreements and 31 separately retained
nonagreements. Fifteen Integer-source cases are rejected by the pin; four implicit
numeric-operand cases are rejected; four excessive-length cases panic; four named
conversion cases have different values. Four remaining error-profile cases retain
their actual error/result observations. Ten cases are explicitly omitted because
of canonical store edits or the unresolved Debug boundary. The retained receipt
verifies exact AML, source hashes and rebuilt binary identity. No private Rust
implementation is mirrored and no firmware or device access occurs.

No new constant-evaluator, native Omega or hardware result is claimed.
