# Resumable normal Field read fixtures

Original synthetic AML exercises actual Program preparation and retained method
execution. Initialized arrays supply native read responses; no callback reaches
hardware. Every scenario has a changed-expectation control inside its Omega body.

This is an incomplete draft. The retained 213 ordinary execution/bridge pairs
passed; the first Field-read pair fails interpretation after checked compilation.
See [STATUS.md](STATUS.md) for the original and decoder-experiment failures.
The authored 70-pair read corpus is not a passing behavior claim.

Run against the clean audited Omega source and its retained checked runner:

```sh
python3 tools/ports/acpi/pipeline/field-reads/check.py \
  --runner /path/to/cathedral-acpi-checked-runner \
  --omega-source /path/to/clean/eaa7993 \
  --record /path/to/read-verification.json
python3 tools/ports/acpi/pipeline/field-reads/check.py \
  --verify /path/to/read-verification.json
```

`--match` selects comma-separated name substrings. Omitting `--record` writes no
receipt. Verification re-renders exact fixture bodies/build recipes and checks
current source/fixture hashes and each selected result. A receipt for a temporary
worktree is not relabeled as final-path execution.

The fixture API also exposes IMPORTS, HELPERS, cases and render for a combined
multi-module checked package. Give its receiver a unique name such as
FieldReadSuite. Existing historical receipts remain unchanged.

The [implementation contract](../../../../../source/libraries/acpi/pipeline/field-reads.PORT.md)
describes the supported normal profile, total-fuel versus quantum limits,
correlation obligations, failure retention and explicit absence of live authority.

`regressions.py` is the byte-identical harness used for the separate regression
receipt. Its historical execution root and generated build paths are retained:

```sh
python3 tools/ports/acpi/pipeline/field-reads/regressions.py \
  --root /tmp/cathedral-field-reads-current \
  --verify tools/ports/acpi/pipeline/field-reads/evidence/regressions213.json
```
