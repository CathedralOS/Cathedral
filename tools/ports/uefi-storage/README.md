# UEFI storage corpus checks

`python3 tools/ports/uefi-storage/check.py` verifies the exact pinned source
inventory, source-to-Omega field mapping, signature metadata, constants, GUIDs,
requested layout policies and vectors. It builds a small no_std Rust probe
against the optional upstream checkout for `x86_64-unknown-uefi`; 657 actual
upstream size/alignment/offset/constant values must match. No firmware runs.

Requirements: Python 3, Cargo with edition 2024, the matching Rust UEFI x64
standard-library target, and the exact reading-room source pin. Install the
standard target with `rustup target add x86_64-unknown-uefi` if needed. Missing
source/target/dependencies fail. Cargo.lock fixes dependencies; build output
uses temporary storage. Artifact hashes identify one run, and may differ when
compiler/debug paths change; numeric measurements are deterministic.

`python3 tools/ports/uefi-storage/semantic-check.py` executes the Omega pure
constant fixture by semantic evaluation, then creates a temporary negative
control that must fail its computed success assertion. It defaults to the
sibling Omega release compiler; `--omega /path/to/omega` selects another binary.

`source-shapes.json` is checked source mapping metadata. It preserves exact
Rust types and intentional Omega adaptations, and is validated against pinned
upstream declarations before use. It is not an alternate ABI authority. The
Rust probe measures the actual upstream records and constants independently.

See [storage.PORT.md](../../../source/contracts/uefi/raw/storage.PORT.md) for
complete coverage, deviations, source status, and remaining layout/authority
seams. Neither passing check proves an Omega-native foreign representation,
variable-tail access, a device action, or Cathedral integration.
