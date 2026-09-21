# Bounded Program resource profile

Status: ACPI-006 complete for the current single-source bounded Program profile.
This records tested resource confinement, not complete AML opcode support.
ACPI-004 and ACPI-005 remain open. These are Cathedral policy limits, not ACPI
maxima, compiler evaluator fuel, or grants of scheduler/device authority.

| Resource | Enforced boundary | Recorded evidence |
| --- | --- | --- |
| Input and names | `prepare_program` owns one initialized 1024-byte snapshot; loaders/executors reject oversized lengths, budgets and mismatched method source units/spans. Paths hold at most 16 segments. | Pipeline input capacity/maximum, source ownership/mismatch; executor inverted/truncated/MAX spans; parser name capacity. |
| Namespace and objects | 32 namespace entries including root; 64 stable objects; 64 byte blocks of 256 bytes. Allocation and binding reject capacity. External arguments stage allocation transactionally. | Loader capacity/rollback, last object slot; generic copy allocation failures; external argument late-capacity/backing failures; complete-state Store bridges. |
| Depth and traversal | Eight parser Package/Scope frames, four method frames, eight control blocks, 16 pending expressions; reference/package walks inspect at most 64 IDs. | Parser values/load depth and reference cycles; executor recursive/block/expression depth; shared/cyclic/full-arena result graphs. |
| Method work | One Runtime counter shared across nested calls, with caller budget at most 1024 turns. Per-turn lookup, object and byte helpers are independently bounded. Loading and value parsing have separate bounded budgets. | Executor zero/oversized/loop budgets and Store-before-exhaustion; nested-call implementation; loader/value exhaustion and rollback. |
| Returned graphs | Program admits at most 64 distinct reachable IDs and 16,384 logical String/Buffer bytes, charging each byte-object ID once. Each backing block remains at most 256 bytes. Oversized requests reject before execution/allocation. | 49 graph-kernel, 27 Program and three supplemental behavior/control pairs: exact/short/zero/MAX quotas, reference edges, malformed nested backing, shared/cyclic graphs, earlier effects and exact diagnostics. |
| Exhaustion and unsupported behavior | Explicit capacity, depth, work-limit and unsupported/unresolved outcomes; graph rejection clears exposed result data while retaining earlier execution effects. | Loader rollback, unsupported opcodes/objects, regions/services, malformed packages, and graph rejection fixtures. |

The [Program API](PORT.md) owns source custody and applies the returned-graph
boundary. Public low-level `engine::run_method` and `run_operands` retain input,
storage, frame, traversal and work bounds, but do not recursively admit every
returned descendant or apply the Program graph quotas. They return inline
scalars or stable IDs in caller-owned storage. Their callers retain source and
definition custody and must use `result_graph::check_graph` before treating a
returned graph as admitted export data, or use Program instead.

Objects are not reclaimed: repeated calls eventually return Capacity at 64
allocated slots. Current Package copies retain bounded child IDs and do not
perform recursive expansion. Cyclic results are finite identity graphs, not
recursively serialized values. Earlier writes can survive method exhaustion;
execution rollback is a different contract. Reclamation, deep copying, dynamic
declarations, Load/Unload and broader field/service execution remain separate
implementation work. Their current exclusions do not bypass these bounds.

Evidence retains its original source identity. The historical generic checkpoint
`c4a8b03` verifies 181 committed input files, 259 checked pairs and one constant
pair through `tools/ports/acpi/interpreter/generic-execution/history/verify_checkpoint.py`.
The [graph receipts](../aml/result-graph.PORT.md) record their isolated worktree
and tested source histories (`1607011`, `5774cea`); graph/Program implementations
were retained during integration. The combined canonical Store suites passed
58 actual bytecode and 38 complete Frame/ObjectStore behavior/control pairs with
the graph boundary enabled and equal/zero-length Buffer support installed.
Their separate `integrated-*-verification.json` receipts under
`tools/ports/acpi/interpreter/named-store-execution/` reproduce exact inputs,
generated bodies, selections and runner hash. They supplement the original
255-pair named Store checkpoint `a75f0cc`; they do not relabel older results.

Checked interpretation, constant evaluation and native execution are distinct
stages. No native execution, hardware/firmware access, or production boot
integration is claimed by this resource milestone.
