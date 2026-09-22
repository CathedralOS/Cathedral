# Executor composition evidence

Status: **draft composition; checked integration execution pending**. This
isolated branch combines the Field-write retirement and pipeline, Concatenate,
qualified ToString, logical-opcode and completed Mid sources. Component receipts
retain their original source roots and scope. No component receipt proves the
combined source, and generation or source audits do not execute Omega.

The composition preserves the component implementations. `provenance.py`
reconstructs the complete ACPI Omega source tree from immutable component
commits, checks exact file sets and bytes, and derives the two shared dispatch
files through explicit insertions. It also verifies the borrowed fixture/tool
sources against their owning commits. The generated provenance record describes
source composition only; source inventory anchors and aggregate counts remain
pending.

## Authored coverage

The full selection has **1,091 positive/control pairs**. The component rows and
legacy regressions are selected once each:

| Group | Pairs | Scope |
| --- | ---: | --- |
| Mixed AML | 16 | Nested and sequential logical, ToString, Concatenate and Mid operations at both Integer widths |
| Expanded Frame | 32 | 26 comparator controls and six actual retirement preservation/rollback rows |
| Composed Field writes | 12 | Six actual AML/provider sequences at both Integer widths |
| Logical AML / retirement | 221 / 138 | Primary conversion, operand order, exact Boolean and whole-state publication |
| ToString AML / retirement | 66 / 79 | Source/Length conversion, owned String publication and target replacement |
| Concatenate | 110 | 86 direct retirement and 24 actual AML rows |
| Field-write retirement | 34 | Complete Frame/Runtime continuation state |
| Field-write pipeline | 68 | 48 actual AML/provider rows and 20 rejection rows |
| Mid AML / retirement | 74 / 42 | Owned byte results and target publication |
| Integer / generic / pipeline / ToInteger | 79 / 55 / 22 / 43 | Retained executor regressions |

The 60 new rows address composition. Mixed AML checks exact result identities,
allocation counts, result bytes and selected retained statement effects. The
named-target rows additionally assert the exact destination tag/owner, extent
and all 256 backing bytes, including after the later logical failure. Field
write compositions compare the full canonical store, every new allocation,
all 512 synthetic provider memory bytes, the complete request/response trace,
result/transfer source identities and issued/accepted/read/write/payload counts.
They include acknowledged effects retained after a later provider failure and
result-byte admission failure after all writes were acknowledged.

Field-write support adds `Frame.field_writes` and `Frame.deferred_write` to the
older execution model. `frame_coverage.py` extends recognized legacy Frame
comparators with those members without editing historical fixtures. It accepts
an already extended comparator only after checking its structure, recursive
Operand/Target helpers and all seven deferred-write payload coordinates.
Unknown or partial comparators fail generation. Dedicated symmetric controls
exercise both union discriminants, all payload coordinates, all Operand/Target
variants and inactive second-target/value fields. Six retirement rows carry a
nondefault pending write through logical, ToString and Mid success/rollback.

Logical and mixed 64-bit true controls use zero as their changed expectation.
The original borrowed scalar generator's increment would exceed `u64::MAX`.
The logical component retains the original mixed receipt and the renderer-only
fix audit; its production code and positive bodies were unchanged by that fix.

The mixed-target backing helper uses `snapshot` as its local name. A
[retained original/renamed probe](diagnostics/block-local/) demonstrates that
the pinned parser rejects `block` in that expression position. Correcting the
identifier changes neither fixture expectations nor production; the probe
does not establish full integration execution.

## Running and verifying

```sh
python3 tools/ports/acpi/interpreter/executor-integration/provenance.py --record /tmp/executor-source-audit.json
python3 tools/ports/acpi/interpreter/executor-integration/preflight.py --record /tmp/executor-render-audit.json
python3 tools/ports/acpi/interpreter/executor-integration/check.py --batch-size 1000 --workers 1 --record /tmp/executor-checked.json
python3 tools/ports/acpi/interpreter/executor-integration/verify_record.py /tmp/executor-checked.json --require-binaries --require-complete
```

`--group` selects groups; `--match` accepts exact comma-separated names. A
selected receipt covers only those rows. Every batch checks the authored and
dependency bodies before running both positive and changed-expectation control
entries. The checker binds all ACPI Omega inputs, borrowed/local generators,
case manifests, generated modules, selections, build/driver text and the pinned
runner. The verifier reconstructs those bindings and every observed result.
The full 1,091-pair claim additionally requires all 16 groups in their declared
order; `--require-complete` enforces that condition. `scope=full` on a deliberately
narrowed group list covers that list only.

Provider traffic is a deterministic caller-supplied fixture. Live provider,
Field-read continuation, Bank/Index Field execution, native Omega and hardware
execution remain outside this composition. A passing source audit alone does
not advance any of those boundaries.
