# AML field syntax checks

```sh
python3 tools/ports/acpi/aml/fields/audit.py
python3 tools/ports/acpi/aml/fields/check.py --omega /tmp/cathedral-omega-eaa7993/release/omega
```

The checker evaluates independently authored metadata cases, then changes one expected behavior inside each body and requires the failure result 1. It records the compiler hash and a source hash covering both parent AML files and the new child package. Source changes during a run invalidate that run.

`fixtures.py` regenerates the original cases. `source-check.omg` imports the complete child package for a separate source check. `provenance.json` and the package inventory bind the exact Rust pin. The source package's `PORT.md` explains pending expression spans, resource policy and runtime boundaries. There is no native ABI or field I/O success claim.

`coverage.json` maps each field form and pending execution boundary. `compiler-notes.md` and `reproduce-field-comparison.py` retain the observed unsigned record-operand comparison mismatch and the checked scalar source form used by the parser.

`verification.json` preserves the historical constant-evaluation baseline: 18 positive semantic cases, 18 rejected body mutations, the 19-file source check, compiler/closure hashes, and per-fixture hashes. The parent AML source remained unchanged during that historical run.

After the parent added method declaration capture, `checked-verification.json` records all 18 unchanged assertion bodies and 18 original mutations executed through the checked interpreter against the current parent/child closure. Run `python3 tools/ports/acpi/aml/fields/check_interpreted.py`, then `python3 tools/ports/acpi/aml/fields/verify_interpreted.py`. This consumer requires the exact pinned runner built by the execution/pipeline harness; its binary hash is checked. Current regression execution is distinct from the historical const proofs and from native execution.
