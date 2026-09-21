# Returned object graph fixtures

The 49 original behavior/control pairs call the canonical graph validator and
check exact failure kinds or object/byte/visited observations. Expected graph
counts are authored independently of the Omega traversal. Controls alter the
expected visited bitmap or failure kind inside the executed assertion body.

```sh
python3 tools/ports/acpi/aml/result-graph/fixtures.py
python3 tools/ports/acpi/aml/result-graph/check.py --record tools/ports/acpi/aml/result-graph/verification.json
python3 tools/ports/acpi/aml/result-graph/check.py --verify tools/ports/acpi/aml/result-graph/verification.json
```

`--runner` selects an existing checked interpreter. `--match` narrows cases for
diagnosis. The receipt records the exact execution root, generated source/build
hashes, production and fixture hashes, selected entry points, immutable runner
hash, observations and unchanged-input result. The driver never rebuilds or
replaces a live binary. Checked interpretation is separate from native execution.
All 49 pairs pass in the recorded isolated worktree. The receipt remains
evidence for that exact root and source snapshot.
