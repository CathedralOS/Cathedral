# Equal-extent named Buffer Store checks

The [port record](../../../../../source/libraries/acpi/aml/named-buffer-store.PORT.md)
defines the supported equal-length 0..256 profile and the separate compatibility
decision for differing Buffer lengths. These tools call the canonical
`named_value_store::store_value`, never a test implementation.

`fixtures.py` reuses the complete store comparator and fixture renderer from the
scalar milestone. `cases.json` contains 97 additional scenarios. The 504-row
`regression-cases.json` changes only the two former Buffer self-store expectations
from unsupported to successful; the old fixture files/receipts remain intact.
Every scenario has a negative control and checks the full ObjectStore.

Both suites pass at `/private/tmp/cathedral-acpi-buffer-store`: 97 new pairs and
504 regression pairs, in `buffer-verification.json` and
`regression-verification.json`. The 12 public Rust observations pass in
`reference-verification.json`. These receipts retain their isolated execution
root; replay at another path produces separate evidence.

```sh
python3 tools/ports/acpi/aml/named-buffer-store/check.py --group buffer
python3 tools/ports/acpi/aml/named-buffer-store/check.py --group regression
python3 tools/ports/acpi/aml/named-buffer-store/check.py --group buffer --verify-record
python3 tools/ports/acpi/aml/named-buffer-store/check.py --group regression --verify-record
python3 tools/ports/acpi/aml/named-buffer-store/inventory.py --check --checkout reference_code/rust-osdev/acpi
python3 tools/ports/acpi/aml/named-buffer-store/reference.py --checkout reference_code/rust-osdev/acpi
python3 tools/ports/acpi/aml/named-buffer-store/reference.py --verify-record --checkout reference_code/rust-osdev/acpi
```

The checked runner at
`/tmp/cathedral-acpi-generic-checked/release/cathedral-acpi-checked-runner` was built
from the clean `eaa7993a23623cd8fabf45350340479c5c9c7879` Omega checkout and the
existing checked-runner source/Cargo lock; see `toolchain.json`. Each recorded
batch binds exact machine selections, result output, generated source and
source/binary hashes before and after execution. `--match` selects a named
subset; subset receipts cannot verify as complete suites. `--workers 1|2|3`
controls concurrent independent batches. Default is two workers and ten pairs
per batch, with evaluator fuel fixed at 10,000,000.

`reference.py` builds the unchanged public-object probe from
`../buffer-target-values/reference.rs` with its exact Cargo lock and calls the
actual pinned Rust method. It records six equal-extent matches plus six excluded
unequal-extent observations. Source files and licenses are checked against the
pin before building. Its receipt is separate from Omega execution evidence.
