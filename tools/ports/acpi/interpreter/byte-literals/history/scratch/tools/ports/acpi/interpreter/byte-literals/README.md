# Standalone byte literal preflight proposal

Everything here is scratch-only. The sole new Omega implementation is
`source/libraries/acpi/interpreter/execution/byte_literals.omg`. No executor,
canonical model, parser, existing helper or build source is modified. The copied
canonical snapshot is checked by `canonical-baseline.json` at the scratch root.
The future executor decides contribution, cursor advancement, budgets and target
publication. These tests exercise preflight itself, not future runtime wiring.

Reproduce from this directory:

```
python3 fixtures.py --check
python3 inventory.py --check
python3 reference.py --write
python3 check.py --record verification.json
python3 check_const.py
python3 verify_record.py
```

The current scratch tools explicitly point to the Cathedral checkout at
`/Users/zcanann/Documents/projects/Cathedral` for the unchanged canonical Rust
runner source/lock, pinned upstream checkout and public interpreter probe. They
build the checked runner ONLY under `/tmp/cathedral-acpi-byte-literal-runner`.
No shared `/tmp/cathedral-acpi-execution-checked` binary is rebuilt. The public
probe uses canonical source and its existing `/tmp/cathedral-acpi-public-execution`
target. No copied Rust source is substituted into a shared target.

`check.py` uses batches of10 actual authored bodies and their changed-assertion
controls. Every success verifies canonical Source type/span/size, cursor, logical
length and all256 bytes including zero tails through canonical `read_bytes`.
Every error verifies its exact outcome/offset, Uninitialized payload and next0.
The fixtures cover all integer/pkglength widths, active-frame and snapshot bounds,
64-bit maximum metadata, 32-bit normalization, capacity, malformed and unsupported
terms. Three representative positive/negative constant-expression pairs separately
exercise ordinary String, normalized Buffer and maximum-offset rejection.

The source snapshot hashes conservatively include every copied ACPI `.omg` file,
all suite Python files/cases, and canonical runner source/lock. It is a superset of
actual imported bodies, not a claim that every copied module is tested. Historical
exploratory logs outside this directory are not final evidence. Native execution,
ABI, live hardware, dynamic size evaluation, Package and future executor integration
are not tested here.

`reference.py` uses actual pinned public load_table/evaluate calls:47 observations,
including32 same-value agreements,5 caught panics and4 returned errors. Remaining
values expose the documented capacity profile, UTF8 admission, RevisionOp value and
dynamic-size profile differences. Pathological giant allocation requests are not
sent to upstream; only Omega preflight receives those bounded-validation cases.
Each observation records its exact AML bytes, public result and forbidden-handler
counter; these are not private Rust algorithm mirrors.

Before eventual publication, root must authorize and rebase the copied current
names/namespace guards, then rerun against the final canonical dependency snapshot.
The current scratch baseline predates those independently authored guard updates.
