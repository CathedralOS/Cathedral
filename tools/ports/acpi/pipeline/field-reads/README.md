# Resumable normal Field read fixtures

Original synthetic AML exercises actual Program preparation and retained method
execution. Initialized arrays supply native read responses; no callback reaches
hardware. Every scenario has a changed-expectation control inside its Omega body.

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
