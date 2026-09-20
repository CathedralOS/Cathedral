# UEFI table verification

Run `python3 tools/ports/uefi-tables/check.py` from the Cathedral root. It requires
Python 3, Cargo, a Rust toolchain supporting edition 2024, the matching
`x86_64-unknown-uefi` standard library target, and the exact upstream reading-room
checkout. Missing source/target/dependencies fail explicitly. Install the Rust
target with `rustup target add x86_64-unknown-uefi` if needed.

The pinned Cargo lockfile fixes probe dependencies. Build output uses a temporary
directory and is removed afterward. No firmware code runs. The probe compiles
actual upstream types for the UEFI x64 target and inspects static LLVM bytes;
it is independent of hand-entered expected vectors. The script separately
checks the Omega table declaration order, authored offsets, slot ordinals,
GUID fields, and GUID mixed-endian byte expectations. A source inventory check
runs first and binds the exact upstream revisions/hashes.

The script prints an LLVM artifact hash for the particular run. Source paths and
debug metadata can change that hash between runs; the numeric comparisons are
the deterministic assertions. Its result is **upstream Rust ABI agreement**, not
an Omega inspection artifact. No output is labelled `omega-inspection`.

`main.omg` retains a small executable constant/slot fixture. Its raw
module import passes `omega --check`. Run `python3
tools/ports/uefi-tables/semantic-check.py` to execute the pure fixture in Omega
semantic evaluation with a negative control that must reject the exact changed
success condition. Both checks pass; native execution is not claimed. Separate `tables_layouts.omg` checking
fails at the confirmed 32-member layout-reflection limit. This semantic check
does not observe ABI layout. See the complete
[port record](../../../source/contracts/uefi/raw/tables.PORT.md) for provenance,
coverage, intentional omissions, and outstanding Omega evidence.
