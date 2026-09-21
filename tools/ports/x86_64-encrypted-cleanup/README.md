# Explicit-profile cleanup checks

The corpus contains 384 complete detached trees in 11 fresh-process encryption
profiles. It checks 181 actual pinned `MappedPageTable::clean_up_addr_range`
calls and 203 exact adapted private recursive cleanup executions. Every final
nonzero word and the complete ordered retirement callback sequence is compared
against the finite cursor oracle. No table is installed and the callbacks retain
all allocations for final inspection.

The generated 424 Omega fixtures evaluate each cursor/branch step of those
complete-tree observations. Six additional fixtures cover malformed pages,
capture mismatches, retained effects from an earlier successful step, forged
active cursors, recursive-index validation and maximum-budget resumption.
Nine controls change expected conditions inside those bodies. The unchanged
success contracts or checked runtime wrappers must observe failure `1`.

```sh
python3 tools/ports/x86_64-encrypted-cleanup/check.py --host-only
python3 tools/ports/x86_64-encrypted-cleanup/check.py --omega /path/to/omega
python3 tools/ports/x86_64-encrypted-cleanup/check_interpreted.py
```

`check.py` checks all generators and upstream anchors, checks that retained
range-advance geometry still matches `cleanup_ranges`, builds the actual pinned
Rust crate with memory encryption and Step enabled, and runs every profile in a
separate process. Without `--host-only` it also checks one current
profile-sensitive constant body and its body-mutating negative control.
Rust uses the installed `nightly-2026-09-04` toolchain because this exact pin's
Step implementation requires the newer trait methods.

`check_interpreted.py` uses Cathedral's existing ACPI checked-runner API consumer.
It source-checks the full selected package once, then executes all 430 unchanged
fixture bodies and nine changed bodies through Omega's checked interpreter.
Its temporary `Suite` methods preserve authored fixture bodies; only the constant
success wrapper is replaced by a receiver entry returning the same result.
`--match TEXT` runs selected smoke fixtures without replacing the full record.
The default record hashes the selected source closure, all owned harness inputs
and the shared runner source/lock before and after execution. It is checked-stage
semantic evidence, not native execution, layout measurement or hardware evidence.

`generate_reference.py` extracts the exact pinned recursive private body. It
substitutes owned registry lookup for recursive pointer resolution, threads that
registry through recursive calls, and reaches the private virtual-address step
through the public `Step::forward_checked` delegate after verifying its exact
body. It does not construct a `RecursivePageTable` or claim to call its public API.
The mapped reference calls the actual public API with distinct stable initialized
allocations and a checked frame-ID registry. Frame zero is an ordinary numeric
registry key, never a null host pointer. The root is excluded from that registry.

Raw words enter actual pinned entries through
`set_addr(PhysAddr::zero(), PageTableFlags::from_bits_retain(word))`. This preserves
encryption-only occupancy, including old configured bits. Profiles cover disabled,
encrypted bits 0/7/12/21/30/47/51, shared bits 47/63, and encrypted 47 followed by
shared 48. The low/flag-bit stress profiles describe source arithmetic, not CPU
configuration admission. Configuration occurs before all table construction.

The inventory and [port contract](../../../source/libraries/x86_64/encrypted-cleanup.PORT.md)
identify canonical types, ownership boundaries and the exact upstream pin. On a
pin update, review both complete cleanup bodies, PTE frame/flag/zero methods,
configuration semantics, the extraction substitutions and retained range geometry
before regenerating the corpus and rerunning every positive and control.
