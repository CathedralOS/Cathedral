# Machine protocol verification

Run `python3 tools/ports/uefi-machine/check.py` from the repository root.
It requires the pinned reading-room checkout, Cargo, and the installed Rust
`x86_64-unknown-uefi` target. Locked dependencies compile into a temporary
folder. The tool checks source coverage and Rust-target representation agreement;
it never labels its output as an Omega ABI observation.

`generate.py` is a restricted exact-pin transcription reproducer. It rewrites
the raw declarations, plans, inventories, vectors, schema and Rust probe. Review
all diffs and rerun the checker and Omega source checks after regeneration.
See [machine.PORT.md](../../../source/contracts/uefi/raw/machine.PORT.md) for
scope, license, primary references, verification and unresolved native seams.
