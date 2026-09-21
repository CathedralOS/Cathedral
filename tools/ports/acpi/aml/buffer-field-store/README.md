# Direct BufferField source Store checks

The [port record](../../../../../source/libraries/acpi/aml/buffer-field-store.PORT.md)
describes source admission, atomic backing publication and the direct-ID boundary.
The 226 original cases use independent Python bit arithmetic and compare complete
canonical stores after actual checked Omega calls. Every case has a changed-body
control altering expected preserved storage. Three constant pairs separately check
representative backing results and reject changed expectations.

Run from the repository root:

```sh
python3 tools/ports/acpi/aml/buffer-field-store/inventory.py --check
python3 tools/ports/acpi/aml/buffer-field-store/check.py
python3 tools/ports/acpi/aml/buffer-field-store/check.py --verify-record
python3 tools/ports/acpi/aml/buffer-field-store/verify_record.py
```

The driver uses the pinned compiler and canonical checked runner declared in
check.py and never rebuilds the shared binary. The
[runner recipe](../../interpreter/execution/README.md) documents its build.
Three independent processes use distinct temporary source/build directories and
deterministic receipt order. The receipt binds the exact used source/build
closure, fixture/driver/store-comparison inputs, two runner recipe files,
execution root/build hash and binaries. Verification checks selected names,
exact expected/observed outcomes, generated source hashes and constant diagnostics.

Focused successful scratch checks preceded the complete final-path replay.
An initial imported-machine transition with a borrowed local payload admitted
but produced a checked-interpreter error before a successful write. The final
implementation carries a detached payload into a local state and calls the
existing writer ordinarily; the same fixtures pass. That diagnostic formulation
is retained at /tmp/cathedral-buffer-field-store-direct-transition-diagnostic and
is not counted as successful evidence or a language blocker.

No native Omega or hardware execution, generic Store dispatch, reference/field
source evaluation or opcode retirement is claimed by the checked corpus.

The separate [public probe instructions](reference.README.md) reproduce 116
actual Rust Store observations. The combined verifier checks every fixture/AML
body, exact public observations and classification totals, final root/build text,
current probe/production mapping hashes and all 27 pinned upstream hashes.
