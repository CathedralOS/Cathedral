# Positive-extent Buffer preparation evidence

The [port record](../../../../../source/libraries/acpi/aml/buffer-target-values.PORT.md)
defines the partial preparation profile and explicit destination-policy exclusions.
Final repository-path verification passed all 204 checked pairs, three constant
pairs and 54 public observations; retained receipts match current inputs.

From the repository root:

```sh
python3 tools/ports/acpi/aml/buffer-target-values/inventory.py --check
python3 tools/ports/acpi/aml/buffer-target-values/check.py
python3 tools/ports/acpi/aml/buffer-target-values/reference.py
python3 tools/ports/acpi/aml/buffer-target-values/verify_record.py
```

204 actual checked behavior/control pairs check complete 256-byte outputs or exact
semantic failures. Three further constant-expression pairs require the actual
helper's body to return zero and reject changed expected data. Large deterministic
alphabet fixtures use a shared initializer, avoiding hundreds of independent
array-write statements while preserving every-byte assertions. No expectation is
computed by the implementation under test. Driver batches run in three independent
temporary builds, retain deterministic order and bind 17 used production/build
files, three fixture/driver inputs, shared runner sources, generated programs,
execution roots and pinned binary hashes. The shared runner is never rebuilt.

54 actual public Rust Object::replace_with_implicit_casting calls operate on local
owned values. The raw method has no IntegerSize argument: 32-bit fixture inputs
are normalized before the call, and the pin still returns eight raw bytes. Seven
results match, 25 differ, and 22 cover excluded profiles without comparison claims.
The host fixture never instantiates an Interpreter or ObjectToken and uses an
isolated Cargo build target. It records all 27 pinned source/manifest/license
hashes and reproduces the complete retained receipt. `--write` deliberately
refreshes the public receipt.

verify_record.py checks current hashes, generated selections, all observed 0/1
results and constant diagnostics, the real execution root, probe inputs, exact
pinned checkout contents and public classifications. It is a receipt audit utility,
not a retroactively claimed executed fixture input. Host binary survival is not
required to validate the retained source-bound observations.

Zero extent Bounds, valid empty String Empty and Buffer-source UnsupportedValue
are local exclusions of this partial API. They are not normative Store failures.
No target presence/provenance, mutation, installed object, provider authority,
native Omega result or completion of aggregate Store/replace is claimed.
