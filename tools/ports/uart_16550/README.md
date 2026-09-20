# UART audit developer fixtures

See [`PORT.md`](../../../source/drivers/uart_16550/PORT.md) for scope, pin,
licensing, source map, deviations and explicit I/O boundaries.

From Cathedral root:

```sh
python3 tools/ports/inventory.py check --checkout reference_code/rust-osdev/uart_16550 --require-transcribed source/drivers/uart_16550/inventory.json
python3 tools/ports/uart_16550/measure.py
python3 tools/ports/uart_16550/check.py --omega /tmp/cathedral-omega-eaa7993/release/omega
OMEGA_BIN=/tmp/cathedral-omega-eaa7993/release/omega tools/uart-16550-facts-canary/run.sh
cargo test --quiet --manifest-path reference_code/rust-osdev/uart_16550/Cargo.toml --lib
```

`measure.py` compiles the exact pinned Rust source through Cargo.lock and checks
168 fact/ordinal values against the Omega source and committed vectors. `--write`
regenerates the vector file only after review. `facts.json` maps existing fact
names as well as additions. The Divisor enum's ordinals are deliberately not
interpreted as baud divisors.

`check.py` prints the compiler hash, evaluates actual Omega helper/test bodies,
and then changes one expected baud value inside the test body. The second
compilation must reject computed result1. This is semantic evaluation, not
native execution. The source root also loads retained error/baud/ordinal data.
No fixture accesses live hardware or constructs firmware/PortIo/MMIO authority.

`map_inventory.py` applies the reviewed mappings to the existing exact-pin
snapshot. Changing the upstream pin requires a fresh snapshot and review of
all mappings, omissions, values, algorithms, tests and licensing. Source tests
are hand-authored derivatives; the lexical inventory is not a semantic parser.
