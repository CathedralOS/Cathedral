# Program result graph boundary fixtures

The 27 original behavior/control pairs use real synthetic AML and the canonical
loader, Program and method executor. They exercise default validation, explicit
object/byte quotas, nested malformed data, shared/opaque references, inert forward
names and Package padding, private result allocation, and observable writes made
before a result is rejected. Oversized quota requests must fail before execution;
inline Integer and void results consume no allocated result-object quota.

```sh
python3 tools/ports/acpi/pipeline/result-graph/fixtures.py
python3 tools/ports/acpi/pipeline/result-graph/check.py --record tools/ports/acpi/pipeline/result-graph/verification.json
python3 tools/ports/acpi/pipeline/result-graph/check.py --verify tools/ports/acpi/pipeline/result-graph/verification.json
```

`--runner` selects an already built binary; `--match` narrows cases for diagnosis.
Positive cases assert result state, preserved identity, final object count and the
method's observable write. Controls change the expected written value. Receipts
bind actual generated code/build text, all dependency bodies and fixture inputs,
runner hash, exact selected entry points and execution root. No native execution,
firmware access or equivalence to a Rust resource quota is claimed.

All 27 pairs pass in the recorded isolated worktree. Its exact-input receipt
remains evidence for that source snapshot and execution root.

A separate three-pair supplement reaches an additional four-byte Buffer through
RefOf and Index carriers. Exact object/byte quotas admit it; either quota one
short rejects it. Each witness also executes an independent copy through the
underlying engine, requiring the Program boundary to preserve exactly the same
step count and fault offset, including rejected results. It has its own input
snapshot and does not change the 27-pair fixture.

```sh
python3 tools/ports/acpi/pipeline/result-graph/supplemental/witnesses.py
python3 tools/ports/acpi/pipeline/result-graph/supplemental/check.py --record tools/ports/acpi/pipeline/result-graph/supplemental/verification.json
python3 tools/ports/acpi/pipeline/result-graph/supplemental/check.py --verify tools/ports/acpi/pipeline/result-graph/supplemental/verification.json
```

All three supplemental behavior/control pairs pass with an exact-input receipt
in the same isolated worktree, bringing graph coverage to 79 pairs overall.
