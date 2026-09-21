# Scratch admission measurements

These measurements describe development experiments, not release proof. Each
receipt names its exact source snapshot; earlier snapshots are retained under
`/tmp/cathedral-aml-generic-execution-*-20260920`. No native execution is claimed.

| Snapshot / fixture | Result | Elapsed |
| --- | --- | ---: |
| argument-action / argument policy and mutation | 12 checked pairs pass | 254 s |
| argument-action / read-only classification | 9 checked pairs pass | 234 s |
| argument-action / scalar/ID frame construction | 9 checked pairs pass | 9.6 s |
| target-action / complete target policy and dispatcher | 25 checked pairs pass | 131 s |
| target-action / bridge, original assertions | 3 checked pairs pass | 609 s |
| target-action / byte bridge, scalar assertions | 1 checked pair passes | 420 s |
| target-action / retirement | Stopped during source admission; no runtime result | 913 s |
| integer-copy / copy parity with full-store expected state | 19 checked pairs pass | 79 s |
| integer-copy / integer target bridge | 5 checked pairs pass | 119 s |
| target-action / matched byte bridge, borrowed-check boundary | 1 checked pair passes | 423 s |
| integer-copy / retirement | Stopped during source admission; no runtime result | about 15 min |
| integer-copy / minimal method entry | Stopped during source admission; no runtime result | about 18 min |
| binding-plan / prospective binding versus actual copied_binding | 64 checked pairs pass | 67.55 s |
| prepared-bridge / repeated binding prediction | 64 checked pairs pass | 67.05 s |
| prepared-bridge / bridge, original three scenarios | 3 checked pairs pass | 557.51 s |
| prepared-bridge / full Frame comparison helper | 8 checked pairs pass | 9.44 s |
| prepared-bridge / byte-copy kernel constant evaluation | Positive and changed-body rejection pass | 111.67 s |

Different scenario counts prevent treating the bridge timings as a controlled
speedup measurement. The borrowed-check fixture keeps the same single
positive/control scenario and changes only the setup/check function boundary.
Its 423-second result versus 420 seconds provides no material improvement.

Frozen source-admission samples repeatedly show write-frame/result-origin
analysis. They are measurements of compiler work, not evidence of incorrect
execution or a language blocker. Failed/stopped attempts have no success claim.
All runners are isolated and pinned; no Omega source or shared runner is edited.

The historical prepared-bridge snapshot did not complete retirement or public
method entry. Current proof is bound separately to the outcome-output revision.

The first full-state bridge fixture reached array-bound/ranking diagnostics in
its comparison helper. Those loops now use guarded item reads and direct bounded
recursion, and the helper passes eight independent physical-field controls. The
restarted equivalent compact bridge fixture is a separate receipt; the diagnostic
receipt is retained and does not count as a production or runtime failure.

Prepared-bridge retirement stopped after 904.14 seconds and
minimal method entry after 903.00 seconds, both during source
admission with unchanged sources and no execution result. Exact source, fixtures,
stop receipts and samples are retained in `/tmp/cathedral-generic-prepared-bridge-admission`.
The separate outcome-output revision changes only the internal mutation helper
to write a semantic `ExecutionOutcome` through an output borrow; its public API
and copy policy are unchanged. The completion receipts below describe its measured effect.

Outcome-output bridge passes the identical three positive/control pairs in
164.591 seconds versus prepared-bridge's 557.508 seconds. The only production
change is internal `mutate_selected(..., outcome:&mut ExecutionOutcome)` returning
unit rather than returning the nominal outcome. Public APIs, semantic-case values,
copy policy and mutation kernels are unchanged. No numeric error encoding is used.
Actual Return/CopyObject retirement passes two pairs in 213.30 seconds;
complete Frame/ObjectStore atomicity passes fourteen pairs in 183.70 seconds.
Minimal method entry passes its original two payload/count pairs in 494.96
seconds. A stricter four-pair fixture independently checks returned Operand
cases and numeric payloads; all four pairs pass in 552.96 seconds. The original
79 integer scenarios and their 79 changed-body controls pass unchanged in
624.80 seconds. All 22 pipeline scenarios and controls pass in 836.55 seconds. Fifteen
transactional external-argument pairs pass in 45.98 seconds; six decoder cursor
and failed-contribution pairs also pass. All 55 generic scenarios and controls pass in 962.73 seconds without
batching or changing the original expectations. These are checked-interpreter results, not native execution.

The full generic and pipeline runs completed; their source-admission samples
are diagnostic timing evidence only. No source change or fixture compaction was
needed after the outcome-output revision. All current proof is enumerated in
`manifest.json`; earlier snapshot successes and stopped runs are historical.

## Canonical replay

The reviewed nineteen production files were published byte-identically. Final
replay uses explicit canonical execution-root and generated-build bindings.

| Receipt | Pairs | Seconds | Stage |
| --- | ---: | ---: | --- |
| `verification.json` | 55 | 890.47 | checked interpreter |
| `integer-regression.json` | 79 | 568.23 | checked interpreter |
| `pipeline-regression.json` | 22 | 771.34 | checked interpreter |
| `arguments-verification.json` | 15 | 39.71 | checked interpreter |
| `decoder-verification.json` | 6 | 100.19 | checked interpreter |
| `entry-minimal-strict-verification.json` | 4 | 495.37 | checked interpreter |
| `bridge-atomicity-verification.json` | 14 | 182.64 | checked interpreter |
| `binding-prediction-verification.json` | 64 | 67.25 | checked interpreter |
| `kernel-const-verification.json` | 1 | 84.89 | constant evaluator |

All canonical replay receipts pass. Historical measurements above remain tied
to their archived scratch sources; they are not rewritten as canonical results.
