# Recursive cleanup witnesses

Run `python3 tools/ports/x86_64-recursive-cleanup/check.py --jobs 4` from Cathedral.
The default is the verified eaa7993 compiler in `/tmp`; override with `--omega`.
`--host-only` checks generated source freshness and the pinned Rust mirror only.
`--case NAME` selects an Omega fixture; each positive has a changed expected behavior
inside its body that must evaluate to 1 and fail the unchanged assertion contract.

`generate_reference.py` extracts the exact nested private cleanup routine from
x86_64 cc35c876d3badb57df54a66e22f7768a52be95f2. It replaces recursive pointer
resolution with an owned stable allocation registry and passes that registry through
the recursion. Public `Step::forward_checked` reaches the pin's identical private
forward helper. Assertions bind each textual substitution. This is a local private
body mirror, not a call to a public `RecursivePageTable` API.

`probe.rs.in` retains initialized table allocations until all observations finish.
Ordinary links are acyclic and use distinct owned tables. The root is absent from
the resolver registry, so accidentally following its recursive self-link fails.
The callback only records retirement, preserving allocations for final inspection.
No live recursive mapper, CR3 instruction, allocator custody or DMA is involved.
The probe uses `cargo +nightly-2026-09-04` with the pinned crate's `step_trait`
feature; `Cargo.lock` records dependencies.

`generate.py` independently models bounded detached steps, records every step,
and checks their composed final nonzero table words and retirement order against
20 whole-tree Rust witnesses. Its 29 Omega step fixtures evaluate the actual
recursive cursor and shared branch/range machines. The snapshot's `other_nonzero`
observation always includes the self-link for ordinary root slots. Nine further
fixtures cover invalid inputs, forged state, zero/high-bit budgets, capture error,
and actual begin/step/resume composition; two dedicated u64MAX fixtures reproduce
and guard the scalar-staging fix. Numeric result checks establish no native ABI.

`map_inventory.py --check` and the shared inventory checker bind the complete
recursive mapper source file, not just selected copied functions. See the adjacent
PORT document and verification record for exact scope and evidence.
