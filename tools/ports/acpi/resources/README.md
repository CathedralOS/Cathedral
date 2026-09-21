# Resource parser checks

Run `python3 tools/ports/acpi/resources/check_interpreted.py --record tools/ports/acpi/resources/checked-verification.json` from Cathedral. This checks the authored suite and dependencies once, then executes every positive body and changed-body control through the pinned checked interpreter. The retained runner/lock build from clean Omega revision eaa7993 in an isolated `/tmp` target directory. `--case NAME` selects a pair.

The separate `check.py` uses constant semantic evaluation via `omega --check` and can also select cases. Four representative pairs retain this additional evidence in `constant-verification.json`; the full suite is measured by the checked interpreter. Its default compiler is `/tmp/cathedral-omega-eaa7993/release/omega`; override with `--omega`. Neither route publishes native artifacts or exercises firmware.

`generate.py` builds/runs the actual pinned public Rust resource parser with
`cargo +nightly-2026-09-04 --locked`, normalizes supported resource fields,
records errors/panics, and generates original synthetic Omega cases. Pass
`--check` to demand exact checked-in generated files and observations. The
independent oracle is `corpus.py`; no Omega implementation is scraped into it.
Rust observations are in `observations.json`. Behavioral differences from the pin
are explicit in the package PORT document, including unsafe pin slice handling,
reserved fields, optional source bytes, defined unsupported tags and EndTag policy.

`map_inventory.py --check` and `tools/ports/inventory.py check` bind the full
upstream `resource.rs`, with all unfinished resource families still pending.
There are no raw firmware samples, AML execution handlers, physical mappings or
production graph changes in this harness.
