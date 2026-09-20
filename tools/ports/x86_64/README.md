# Full x86_64 source inventory

`python3 tools/ports/x86_64/inventory.py` verifies the exact pinned checkout,
all 41 tracked Rust files, all module classifications and local comparison
anchors, the shared source inventory, enum/payload-field/macro supplements,
and 85 upstream test/proof scenarios plus four integration entry roots. Missing or mutated upstream source fails.

`--write` regenerates artifacts only after reviewing the explicit module catalog
and symbol classification rules. Pending inventory entries are **reviewed future
implementation work**, not unclassified modules. The checker compares full
reproduced artifacts, so missing source/target rows fail instead of passing a
partial coverage claim.

The shared lexical scanner is deliberately not a Rust compiler. Supplements
cover implicit enum variants and payload fields; whole-file hashes bind private
fields, macro bodies, cfg branches and every unexpanded detail. One format-string
false anchor is explicitly rejected as non-code. Macro expansion and trait API
semantics remain source-review work, not purported semantic-parser output.

[PORT.md](../../../source/libraries/x86_64/PORT.md),
[MODULES.md](../../../source/libraries/x86_64/MODULES.md), and
[RECONCILIATION.md](../../../source/libraries/x86_64/RECONCILIATION.md) record scope,
reuse constraints, authority decisions and bounded implementation order.
No inventory success establishes Omega compilation, test execution, native ABI
or hardware access.

`python3 tools/ports/x86_64/check_catalog.py [omega]` source-checks the u8 port
control and verifies the exact u16-port and invlpg rejection diagnostics. These
are bounded compiler capability probes; no instruction is executed.
