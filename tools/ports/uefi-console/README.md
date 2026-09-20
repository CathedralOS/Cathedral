# Console/image/device-path upstream probes

These are host tools for the [raw port record](../../../source/contracts/uefi/raw/console.PORT.md).
The Rust probes do not execute Omega or firmware. A separate Omega semantic
fixture executes the pure helpers. Run from the Cathedral root:

```sh
python3 tools/ports/inventory.py check source/contracts/uefi/raw/console-inventory.json --checkout reference_code/rust-osdev/uefi-rs
python3 tools/ports/uefi-console/measure.py
python3 tools/ports/uefi-console/check_transcription.py
python3 tools/ports/vectors.py source/contracts/uefi/raw/console.vectors.json
cargo test --manifest-path tools/ports/uefi-console/Cargo.toml
```

`measure.py` requires the exact optional upstream checkout, Cargo and the installed
`x86_64-unknown-uefi` Rust target. It measures native Rust values, then generates
`src/lib.rs` assertions for every numeric value and GUID byte and compiles them
for UEFI x64. A host success alone cannot satisfy the target check. The checked-in
vector file must match; `--write` changes it only after both steps pass.

`check_transcription.py` independently calculates the required C/packed geometry
from the Omega fields, compares every field/size/alignment and constant to the
Rust-target vectors, and checks complete field mappings in `schema.json`.
It also checks all 114 authored policies against the same vectors. It does
not claim plain data has that home layout; selected consumers encounter the
compiler limits recorded in console.PORT.md.

`generate.py` reproduces the reviewed translation, inventory, probe and schema
from the pinned source. It is a restricted translator for these exact seven
files, not a general Rust parser. It verifies source hashes via `inventory.py`;
review generated changes and run all checks after regeneration. It preserves
original Rust pointer/function signatures as comments, while source fields carry
inert `addr`. Generated packed nodes retain fixed prefixes; runtime tail fields
are explicit seams. Do not use it to update an upstream pin incidentally.

`src/tests.rs` runs two upstream behavior checks: exhaustive 16-bit device-node
length decoding and the zero graphics-mode constructor. `fixtures.omg` retains
corresponding Omega cases plus malformed/boundary length cases for the local
prefix-bounds helper. `main.omg` calls them through a compile-time constant and
requires the computed result to be zero. This genuinely evaluates helper bodies:

```sh
../Omega/target/release/omega --check tools/ports/uefi-console/main.omg
```

`negative.omg` adds one to the evaluated result and must fail. Separate
`layout_type_only.omg`, `layout_projection.omg`, and `layout_probe.omg` distinguish
selected-layout checking from public-field projection and full fixed-corpus
consumer limitations. These source fixtures have an isolated application build;
they do not call firmware or depend on a native host console provider.

No code here acquires a protocol, dereferences an address, or invokes firmware.
