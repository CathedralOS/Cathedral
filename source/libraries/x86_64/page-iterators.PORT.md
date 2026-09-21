# Detached virtual page iterator transitions

Status: tested through actual Omega checked-interpreter execution. This slice adds the mutable cursor behavior omitted by the earlier
bounded [page range selection](pages.PORT.md) policy. Existing APIs remain separate.

Modified from `src/structures/paging/page.rs` in rust-osdev/x86_64 0.15.5 at
`cc35c876d3badb57df54a66e22f7768a52be95f2`, with retained
[MIT OR Apache-2.0 licenses](../../../licenses/rust-osdev/x86_64/).
The full PageRange/Inclusive methods, Page arithmetic/Step implementation and
current Omega data, machine and boundary contracts were consulted. The
[full-file inventory](page-iterators-inventory.json) maps all eight next/nth/back
anchors; other geometry and representation algorithms remain in existing slices.

`page_iterators::step(start,end,size,inclusive,index,reverse)` reuses
`encrypted_frames::IteratorResult` and `addresses::NumberResult`. It creates no
duplicate result type. Index zero has the same transition as direct next or
next_back; other indices preserve nth/nth_back order. Supported sizes are 4 KiB,
2 MiB and 1 GiB under the pinned canonical 48-bit, 64-bit-usize profile.

Both endpoints must be canonical aligned page starts. Unsupported size or
malformed ordinary input produces `failed=true` with the original cursor values.
Empty/reversed intervals produce no selection and `failed=false`. Gap-spanning
canonical endpoints are admitted here. Length uses raw Page subtraction rather
than dense canonical distance, exactly as the pinned iterator does.

A valid indexed operation first updates the selected cursor, then executes its
successor/predecessor update. Yield happens only after that second update succeeds.
For a requested jump across the canonical gap, the pinned dense Step distance
identifies the last low page or first high page; the cursor first moves there,
then ordinary arithmetic fails when attempting the next step into the gap. The
result retains the completed boundary update and reports failure without a
selection. It does not silently jump the gap using Step arithmetic.

An overshooting index first consumes the last remaining indexed selection, then
attempts one more next/back operation. Its earlier cursor changes survive a later
failure. Inclusive iteration at the maximum page decrements the end instead of
overflowing start; inclusive reverse iteration at zero increments start instead
of underflowing end. A singleton at a canonical-gap boundary can therefore fail
before yielding, unlike the existing bounded `pages::range_at` policy.

The implementation reuses existing validated geometry, raw arithmetic and dense
distance helpers. The gap-aware guards and endpoint-count bound prevent wrapped
index multiplication on admitted iterator paths. Added ordinary-input rejection
does not fabricate invalid unchecked Rust values. The 32-bit usize profile,
pointer/iterator trait objects, native ABI, Kani universal proofs and live page
or mapping authority remain outside this component.

The reference performs 1,614 actual public iterator operations and twelve actual
public typed-input rejections, across all three sizes. It records success,
exhaustion, panic and both cursor fields after each operation. Four additional
Omega cases reject unsupported geometries. Of the public operations, 180 are
direct next/next_back calls retained separately from nth(0)/nth_back(0), checking
the index-zero equivalence against actual public methods. No copied private
body is used.
The [harness README](../../../tools/ports/x86_64-page-iterators/README.md) describes
compact checked execution, representative constant checks, body controls and
the source-bound records. Compiler revision is
`eaa7993a23623cd8fabf45350340479c5c9c7879`; no native or hardware execution is claimed.

The full command `python3 tools/ports/x86_64-page-iterators/check.py` passed
1,630 rows in 51 positive groups and 51 changed-body controls in 236.772 seconds.
All positives returned zero, all controls returned one, and no execution error
or filesystem attempt occurred. Maximum interpreter fuel was 34,543 of the
10,000,000 ceiling. The [checked execution record](../../../tools/ports/x86_64-page-iterators/checked-verification.json)
binds 27 unchanged source/harness inputs and the generated suite.

The optional six-group `check.py --const` route is not claimed as completed.
A 32-row gap-group attempt was stopped during compiler write-frame/origin
analysis without a source diagnostic. The separate direct
[constant fixture](../../../tools/ports/x86_64-page-iterators/check_const.py)
uses public Rust observation 352: a gap-crossing inclusive nth retains the
last low canonical page before its next step fails. Its control changes that
expected cursor value while keeping the success contract unchanged. This
standalone fixture and final record verifier are included in the final checked
run's source snapshot; the full suite was rerun after their addition.

`python3 tools/ports/x86_64-page-iterators/check_const.py` passed both the
direct positive and the changed-cursor control in 338.905 seconds. The positive
source check succeeded; the control computed one and failed the unchanged
`requires value==0` contract with `1 == 0`. The
[constant record](../../../tools/ports/x86_64-page-iterators/constant-verification.json)
binds the same 27 final input hashes.

`python3 tools/ports/x86_64-page-iterators/verify_record.py` passed against both
current records, regenerating suite hashes and checking all input hashes, binary
hashes and exact positive/control counts. Python syntax, JSON/JSONL, whitespace
and local Markdown link checks also pass. These are finite semantic observations,
not universal proofs or native code-generation evidence.
