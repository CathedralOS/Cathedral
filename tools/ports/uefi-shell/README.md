# Pinned shell protocol probe

Run `python3 tools/ports/uefi-machine/check.py --slice shell` for pinned-source,
transcription and Rust UEFI-target representation checks. The shared exact-pin
reproducer is `python3 tools/ports/uefi-machine/generate.py --slice shell`.
It rewrites only this slice. Both commands need Cargo, the installed Rust
x86_64-unknown-uefi target, and the exact upstream reading-room checkout.

See [shell.PORT.md](../../../source/contracts/uefi/raw/shell.PORT.md) for the
complete source map, licensing, test commands and confirmed reflection limit.
Rust ABI measurements are not Omega layout observations or firmware execution.
