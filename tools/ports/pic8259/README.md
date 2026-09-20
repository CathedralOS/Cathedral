# PIC port checks

Run `python3 tools/ports/pic8259/check.py [path/to/omega]` from Cathedral root.
The default compiler is `../Omega/target/release/omega`. Requires Python3, Git,
Rust and the exact pinned `reference_code/rust-osdev/pic8259` checkout.

The runner audits source coverage and numeric vectors, then compiles pinned
Rust controller methods with recording port replacements. It executes the
actual translated Omega helpers through compile-time semantic evaluation.
`negative.omg` changes the expected first initialization command from 0x11 to 0x12
inside the test body while preserving the final assertion: it must evaluate
failure 1 and reject. The source-check fixture never issues port I/O.

`check_existing.py` separately checks the existing core PIC file byte for byte
in a modern isolated package, importing the real facts. Its source regression
checks cover ordered writes and PortIo/Pending acknowledgement spelling; these
are explicitly source checks, not replacements claiming compiler artifact
contract evidence. The older initialization/root canaries currently stop at
an external-symlink package rule before compilation.

[PORT.md](../../../source/libraries/pic8259/PORT.md) records exact mappings,
intentional provider policy differences, test scope and results. No test grants
PortIo, acknowledges a live interrupt, or executes privileged hardware code.
