# Direct named-value Store checks

Status: tested at the final repository path: 305 checked pairs, three constant pairs and 297 public Rust observations. See the
[port record](../../../../../source/libraries/acpi/aml/named-value-store.PORT.md)
for exact admission, exclusions and destination-block publication policy.

```sh
python3 tools/ports/acpi/aml/named-value-store/inventory.py --check
python3 tools/ports/acpi/aml/named-value-store/check.py
python3 tools/ports/acpi/aml/named-value-store/check.py --verify-record
```

The 305 actual checked behavior/control pairs compare the complete canonical
ObjectStore: every object case/payload/link, every entry/Path slot, block metadata
and all 16,384 byte cells. The independent oracle applies primary numeric/string
conversion and positive-extent fitting using Python. Controls change expected
kind, ID, links, byte tails or error cases rather than changing only a final flag.

Three representative const pairs use the same production call and compare all
object/entry metadata plus byte-block sentinels, retaining the full byte scan in
the checked stage. The driver checkpoints completed checked evidence before
running constants; receipt verification still requires both stages. Concurrent
batches use unique temporary directories and deterministic result ordering.

Records bind the exact execution root and literal build text/hash, 18 used
source/build files, driver/fixtures/cases, runner recipe sources, generated bodies
and immutable compiler/runner hashes. Inputs are checked before/after execution;
the shared runner is never rebuilt. Wall time and checked-batch elapsed sum are
reported separately. Source-backed input is immutable and no existing production
module is edited by this slice.

Parent-owned reference.py/reference.README.md and the combined verifier retain
separate actual public Rust observations. The pure checks here do not fabricate
ObjectToken, perform hardware access or claim native execution. Aggregate Store
and replacement anchors remain pending despite this completed bounded component.
