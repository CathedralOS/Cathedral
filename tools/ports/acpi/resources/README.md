# Resource parser checks

Run `python3 tools/ports/acpi/resources/check_batched.py --jobs 2 --batch-size 40`
from Cathedral. This checks disjoint suites and executes every positive and
changed-body control through the pinned checked interpreter. It verifies the
exact corpus union and shared current source hashes before writing
`checked-verification.json`. Individual batch records include all observed
results and fixture hashes. The runner/lock build from clean Omega revision
eaa7993 in an isolated `/tmp` target directory. `check_interpreted.py --case NAME`
selects pairs directly; omitting `--case` selects a single entire-corpus suite.

The separate `check.py` uses constant semantic evaluation via `omega --check`.
The current representative run is:

```
python3 tools/ports/acpi/resources/check.py --jobs 1 --case gpio-pins-vendor --case i2c-ten-bit --case template-connections --case irq-default --record tools/ports/acpi/resources/constant-verification.json
```

Its default compiler is `/tmp/cathedral-omega-eaa7993/release/omega`; override
with `--omega`. Neither route publishes native artifacts or exercises firmware.

`generate.py` builds/runs the actual pinned public Rust resource parser with
`cargo +nightly-2026-09-04 --locked`, normalizes supported resource fields,
records errors/panics, and generates original synthetic Omega cases. Pass
`--check` to demand exact checked-in generated files and observations. The
independent oracle is `corpus.py`; no Omega implementation is scraped into it.
GPIO/I2C fixtures include nonzero origins, exact buffer-end descriptors, pin
padding, No Pin, vendor bytes, empty source names, flag alternatives, malformed
extents, reserved fields and source termination. Public pin differences remain
visible in `observations.json`; they are documented in the package PORT document.

`map_inventory.py --check` and `tools/ports/inventory.py check` bind the entire
upstream `resource.rs`. All families implemented by that pin now have bounded
mappings; other ACPI families remain explicit Unsupported. This is not completion
of general AML resource acquisition or controller support.

After the above current-source runs, `record.py --write` verifies every checked
pair, constant result, source/fixture hash and actual public reference observation
before updating the release manifest. `record.py` verifies the retained release.
Historical first-slice records live under `history/first-slice-4b95485/`; their
hashes apply only to that commit, not the current GPIO/I2C extension.

No raw firmware samples, AML execution handlers, physical mappings or production
graph changes are part of this harness.

The dependency audit includes `acpi/headers.omg`, transitively imported through
`bytes.omg`. Initial run hashes covered only nine/ten paths;
`source-closure-audit.json` transparently supplements that unchanged header. The
final manifest distinguishes original run subsets from the complete eleven-file
Cathedral closure. Future runs hash the full closure before and after execution.
