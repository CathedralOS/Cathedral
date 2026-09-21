# AML declaration capture and single-source execution

This package connects the bounded static loader to the bounded generic method
executor. It adds no hardware access, firmware invocation, physical mapping or
production boot import. Full ACPI-005/006 interpreter coverage remains pending.

## Source and licensing

The upstream reference is rust-osdev/acpi at
`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, MIT OR Apache-2.0, copyright 2018
Isaac Woods. The existing [AML port](../aml/PORT.md), its inventory, and
`tools/ports/acpi/aml/provenance.json` retain the source and licensing audit.
This adapter and its synthetic test inputs are original Cathedral code. Its
method declaration semantics reuse the translated loader in `aml/loader.omg`;
execution remains the partial translation documented by
[the executor](../interpreter/execution/generic.PORT.md). No upstream source symbol is
newly classified as fully translated solely because this adapter exists.

## Declaration capture

`aml::model::MethodDefinition` is shared by the loader and executor. It contains
an inert observation: presence, stable object ID, original resolved method path,
flags and retained body span. The path is the initial method scope expected by
the existing executor, including the method name.

`load_with_definitions` records that observation at successful Method declaration
insertion, using the newly allocated object ID. It never reconstructs history
from an alias or the current spelling of a namespace entry. Alias retains its
existing object ID. Redeclaration allocates a fresh ID; original-name rebinding
does not erase the old observation. Objects are not reclaimed in this bounded
namespace profile.

Five supplementary [public Rust pipeline observations](../../../../tools/ports/acpi/aml-public-execution/README.md#declaration-and-alias-witnesses)
exercise these encoded declarations through the pinned loader and evaluator.
Addition, nested calls and method redeclaration agree. Both cross-scope alias
cases return 22 in the pin, while this component's declaration-scope observation
returns 11; the difference also survives original-name rebinding. The record
preserves that distinction and makes no compatibility claim for those two cases.

Before publishing an insertion, the loader checks the ID is below 64 and its
observation slot is unoccupied. An occupied slot produces `InvalidState`; it
never silently overwrites a prior observation. A failed parse, exhausted budget
or later capacity failure returns both the original namespace and original
observation table. The original `load` signature and `LoadState` layout remain
compatible; that API discards observations and cannot recover them later.

The low-level observed loader takes ordinary namespace/observation values. Its
caller must preserve the immutable initialized source for every retained span,
including across incremental loads. A source unit integer alone does not prove
byte identity. It supplies no multi-unit source registry.

## Owned source wrapper

`prepare_program` takes an initialized `[u8; 1024]` by value, length, unit and
loader budgets. It retains that source snapshot, the ObjectStore and observations
in `Program`. `run_program` accepts no replacement input buffer: it executes
against the retained snapshot and mutates the retained ObjectStore. Changing the
caller's original input after preparation cannot replace this snapshot.

A failed preparation produces an unloaded program which execution rejects.
There is no incremental reload or source replacement API in this wrapper. Calls
into a different retained unit are rejected by the executor; multi-unit
retention and dispatch remain pending. `Program` and observations are ordinary
public semantic records, not unforgeable certificates: clients preserving a
prepared program must preserve its source snapshot and metadata. The executor
checks observed object ID, flags and span against the live Method payload.

## Limits and boundaries

The existing parser limits remain: 1024 initialized bytes, 32 namespace entries,
64 objects and fixed parsing budgets. The observation table has 64 slots indexed
by object ID. The executor retains four frames, bounded expression/block stacks
and a caller work budget capped at 1024. These are Cathedral policy bounds,
not ACPI maxima. The pipeline does not expand opcode support, dynamic namespace
semantics, field access, synchronization or handler behavior. Execution effects
are not rolled back on method failure; only static load is transactional.

## Verification stages

**Current generic replay:** the unchanged 22 pipeline scenario/control pairs pass
against the current ObjectStore execution bridge. See the source-bound
[generic manifest](../../../../tools/ports/acpi/interpreter/generic-execution/manifest.json).

**Historical declaration milestone:** all 22 pipeline cases and 22 body controls pass on the final source
(380.560 seconds, maximum 75,688 evaluator units, zero filesystem attempts).
That milestone also passed 27 parser, 79 executor and 18 field-parser pairs.
The existing load-method/load-rollback and frame-admission/MAX-offset constant
proofs each pass their changed-body controls. Historical full const records
retain their original hashes; current full regressions use the distinct checked
interpreter stage.

`tools/ports/acpi/pipeline/check.py` checks authored package bodies with the pinned
Omega compiler libraries and executes the original Omega bodies through the
checked interpreter. Each scenario has a changed expected-value condition inside
its assertion body; the positive must return zero and the changed body must
return one without interpreter errors. The harness has a separate 10-million
interpreter-step ceiling, which does not alter the source-level parser or method
budgets. This is neither native execution nor a firmware test.

`--parser-regression` reuses all 27 existing parser assertion bodies and their
exact original mutations. Their imports/helpers are retained; only the test
entry spelling changes to select each body. The existing 79 executor scenarios
are rerun using their own harness. Representative original parser and frame
fixtures additionally use constant evaluation and checked requirements.

Exact outcomes, source hashes, compiler revision and harness hash are retained
in the companion verification records. Commands and remaining evidence are in
[the test README](../../../../tools/ports/acpi/pipeline/README.md).

[Returned object graph quotas](../aml/result-graph.PORT.md) apply one shared object/byte budget at the public Program boundary. Nested byte
backing and Package member chains are validated without evaluating references,
resolving names or copying the graph. The 49 pure-kernel, 27 Program and three supplemental behavior/control pairs
pass in the recorded isolated worktree. Rejected results expose no
value while preserving prior execution effects and diagnostics. This is partial
ACPI-006 evidence; reclamation, deep copy accounting and other resource work
remain open. No upstream source anchor is promoted by this original composition.
