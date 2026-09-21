# Standalone byte literal preflight proposal

The sole new Omega implementation is
`source/libraries/acpi/interpreter/execution/byte_literals.omg`. This published
helper has no executor dispatch integration. It does not change the canonical
model, parser, other helpers or package builds. The future executor decides
contribution, cursor advancement, budgets and target publication. These tests
exercise preflight itself, not future runtime wiring.

Reproduce from this directory:

```
python3 fixtures.py --check
python3 inventory.py --check
python3 reference.py --write
python3 check.py --record verification.json
python3 check_const.py
python3 verify_record.py
```

Tools resolve canonical Cathedral and sibling Omega from their repository paths.
They build the unchanged canonical checked runner ONLY under
`/tmp/cathedral-acpi-byte-literal-runner`; no shared execution runner is rebuilt.
The public probe uses its canonical source and existing
`/tmp/cathedral-acpi-public-execution` target.

`check.py` uses batches of10 actual authored bodies and their changed-assertion
controls. Every success verifies canonical Source type/span/size, cursor, logical
length and all256 bytes including zero tails through canonical `read_bytes`.
Every error verifies its exact outcome/offset, Uninitialized payload and next0.
The fixtures cover all integer/pkglength widths, active-frame and snapshot bounds,
64-bit maximum metadata, 32-bit normalization, capacity, malformed and unsupported
terms. Three representative positive/negative constant-expression pairs separately
exercise ordinary String, normalized Buffer and maximum-offset rejection.

The source snapshot hashes include the exact14-file transitive Omega import and
package-build closure, all suite Python files/cases, and canonical runner
source/lock. Native execution, ABI, live hardware, dynamic size evaluation,
Package and future executor integration are not tested here.

`reference.py` uses actual pinned public load_table/evaluate calls:47 observations,
including32 same-value agreements,5 caught panics and4 returned errors. Remaining
values expose the documented capacity profile, UTF8 admission, RevisionOp value and
dynamic-size profile differences. Pathological giant allocation requests are not
sent to upstream; only Omega preflight receives those bounded-validation cases.
Each observation records its exact AML bytes, public result and forbidden-handler
counter; these are not private Rust algorithm mirrors.

`history/scratch` preserves the original75+3+47 scratch receipts and artifacts,
including the older names/namespace bytes and original manifest. Current receipts
are freshly generated against the canonical guard updates at Cathedral7629bb8;
the verifier checks both current dependencies and unchanged historical artifacts.
