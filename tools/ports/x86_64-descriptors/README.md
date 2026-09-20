# Pure descriptor evidence

[Port record](../../../source/drivers/facts/x86_descriptors.PORT.md) contains exact
scope, source map, provenance, representation and compiler limitations.

```sh
python3 tools/ports/x86_64-descriptors/check.py --omega /tmp/cathedral-omega-eaa7993/release/omega
```

`--host-only` runs generated freshness, pin/anchor coverage, 32 actual Rust host
observations and UEFI-x64 assertions, four Rust tests and the unchanged existing
IST pair audit. It requires the Rust `x86_64-unknown-uefi` target. The full runner
also executes actual Omega helper bodies with four body-mutated negative
controls and checks local equivalent schemas using the actual layout policies.
No native ABI or CPU execution is claimed.

`generate.py` emits facts, requested policies, schema and full byte codecs;
`generate_inventory.py` emits the bounded source map;
`generate_reference_tests.py` extracts the actual pinned TSS encoder and bitmap
comparisons with numeric/byte observation inputs; `generate_fixtures.py` emits
independent literal expectations. All four accept read-only `--check`.
`measure.py` executes pinned values and compares committed vectors; `--write`
updates expected vectors after review.

The failed consumers are separate, reproducible evidence:

```sh
/tmp/cathedral-omega-eaa7993/release/omega --check tools/ports/x86_64-descriptors/layout_projection.omg
/tmp/cathedral-omega-eaa7993/release/omega --check tools/ports/x86_64-descriptors/tss_layout_projection.omg
/tmp/cathedral-omega-eaa7993/release/omega --check tools/ports/x86_64-descriptors/legacy_ist_constant_probe.omg
```

They respectively expose imported plan-laid field visibility, packed-array
alignment and legacy imported aggregate-constant leaf correspondence. The
passing `layout_local_projection.omg` is deliberately a local equivalent schema
check; it does not claim the failing imported access path is usable.

The original profile/stack canary can be compared before/after the copy-only IST
fact correction with the modern package wrapper:

```sh
python3 tools/ports/x86_64-descriptors/check_existing.py /tmp/cathedral-omega-eaa7993/release/omega --baseline
python3 tools/ports/x86_64-descriptors/check_existing.py /tmp/cathedral-omega-eaa7993/release/omega
```

`--baseline` substitutes IST source from Cathedral `cbbf9b6c9c9365728c24fcedafe6cb6fdd3bfdda` in a temporary facts package. At the
reviewed baseline it yields four constant-copy diagnostics and one pre-existing
vector-range proof diagnostic; the copy-only edit leaves just that same range
diagnostic. This comparison does not claim the original core profile passes.
