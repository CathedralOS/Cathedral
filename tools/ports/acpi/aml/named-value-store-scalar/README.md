# Scalar named Store checks

Canonical-path validation passed and is frozen: 504 checked pairs and six constant pairs, with exact current hashes verified. The 504 checked pairs comprise 305 unchanged object-source regressions, 143 scalar cases, 53 direct admission cases and three object-source re-admission cases. Six representative constant pairs exercise the original object-source controls and one scalar conversion per destination kind.

```sh
python3 tools/ports/acpi/aml/named-value-store-scalar/check.py
python3 tools/ports/acpi/aml/named-value-store-scalar/check.py --verify-record
python3 tools/ports/acpi/aml/named-value-store-scalar/inventory.py --check --checkout /Users/zcanann/Documents/projects/Cathedral/reference_code/rust-osdev/acpi
```

The baseline generator and cases are retained byte-for-byte in baseline_fixtures.py/baseline_cases.json; cases() verifies that the first 305 scenarios remain identical. Full checked comparisons inspect all semantic payloads, links, namespace metadata and 16,384 arena bytes. Const comparisons use all object/entry metadata and first/last byte sentinels. Controls alter expected outcomes or actual expected stored data, rather than changing only a final flag. Earlier admission is observed and checked before malformed mutation, then both writers must reject the changed current state.

verification.json is complete only after checked and const stages pass. It binds the current execution root (including equality to ROOT), literal build text/hash, current and baseline fixtures, production/helper closure, generated bodies and immutable compiler/runner hashes. Three workers use independent temporary package directories and deterministic ordered receipt assembly. The source and tools stay unchanged during a frozen run; wall time and checked-batch elapsed sum are distinct. No shared runner is rebuilt.

The original smoke.json remains only in the isolated scratch tree and records an earlier eight-pair fixture revision; source bytes remained unchanged, but final current-hash evidence must come from verification.json. Earlier published public Rust named Store observations remain historical, not newly executed by this scalar tool. The source PORT/inventory explain the partial upstream mapping and pending generic integration.

[Historical pre-scalar inputs](history/README.md) retain the exact old named305/public297 closures from c4a8b03. Run `python3 tools/ports/acpi/aml/named-value-store-scalar/history/verify.py` to check all 61 archived files without reinterpreting historical receipts as current-source execution.
