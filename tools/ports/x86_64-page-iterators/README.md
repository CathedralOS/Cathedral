# Detached virtual page iterator transitions

`generate_inputs.py` creates 1,626 public pinned Rust observations across 4 KiB,
2 MiB and 1 GiB pages, exclusive/inclusive ranges, direct next/next_back and
nth/nth_back calls. Of these, 1,614 execute actual iterator operations and twelve
check rejected public typed-address/page construction. Four extra Omega rows
reject unsupported numeric page sizes.

The reference catches iterator panics and reads both actual public cursor fields
after the call. No private-body mirror, pointer or live table is involved.
Inputs cover reversed/empty ranges, low/high canonical boundaries, gap-spanning
ranges, maximum pages, overshooting indices and both directions. Fresh default
virtual-address arithmetic is independent of memory-encryption configuration.

```sh
python3 tools/ports/x86_64-page-iterators/check.py --host-only
python3 tools/ports/x86_64-page-iterators/check.py
python3 tools/ports/x86_64-page-iterators/check_const.py
python3 tools/ports/x86_64-page-iterators/verify_record.py
```

The default check verifies all generators, reruns actual Rust calls, then uses
the pinned Cathedral ACPI checked runner to source-check and execute the authored
Omega bodies. Compact groups contain up to 32 initialized rows and share the
comparison/iteration functions. Every group has a body control that reverses its
first row's expected failure flag; the unchanged computation must return 1.
`--match group_000` runs selected smoke evidence without replacing full records.
`check_const.py` evaluates one direct gap-crossing observation under a
`requires value==0` contract. Its changed expected cursor makes the actual body
return one, rejected by the unchanged contract with `1 == 0`. The optional
`check.py --const` route offers six compact groups but is not part of the retained
completed evidence; its larger gap fixture was stopped during compiler analysis.
`verify_record.py` checks both retained stages against the current complete
selected input closure, generated suite hashes, binary hashes and exact positive/
control counts without rerunning Omega or the public Rust observations.

The checked and constant records bind exact selected dependency/harness hashes,
compiler/runner hashes, generated-suite hashes and full output. The source
closure includes the unchanged `encrypted_frames::IteratorResult` declaration
used by this slice. The virtual machines do not invoke encryption operations.
No record demonstrates native code generation, hardware access or a universal
proof. The [port report](../../../source/libraries/x86_64/page-iterators.PORT.md)
records the actual completed stages and the immutable upstream source inventory.

For a pin update, inspect the full PageRange/Inclusive methods and their ordinary
arithmetic versus dense Step helpers, update generated inputs if needed, produce
fresh `reference.jsonl` using the Cargo command in `check.py`, regenerate groups
and inventory, then rerun positives and controls. Read-only source drift checks
use each generator's `--check` option.
