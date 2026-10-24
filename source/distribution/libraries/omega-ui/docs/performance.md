# Refresh-rate rendering

Target: complete drag work inside the active monitor's refresh period, then
verify actual presentation. The primary Windows display queried on 2026-10-22
(America/Los_Angeles) reports 3840 x 2160 at 240 Hz: 4.167 ms per frame.
Offscreen rendering costs do not prove displayed FPS or input-to-display latency.

## Current architecture validation: 2026-10-23

The library now uses linked children, a bounded shared frame lifecycle, prepared
paint commands, explicit viewports, font intrinsics, reusable text editing and
viewport-derived list ranges. Windows bitmap/font/edit services live in
`platform/windows`; the gallery consumes them. Themes have one resolver.
This establishes shared ownership and behavior; it does not establish a speedup.

The canonical image is `47e0ed0798d40a627157998df03c29e6f288a007e9b0ba4195ef7a54dba2718d`,
built with optimized compiler `10d0fae11579` (binary SHA256 `2cc2705d3a83...`).
`build/compiler.json` and `oct23-architecture-pipeline.json` bind the producer,
source inputs and image. Three native replays pass all 144 RGB comparisons,
six forced-fallback comparisons and twelve independent corner probes (zero
channel error). Each replay produces all 27 tab captures and preserves labels
outside the launcher. Light/dark, launcher and enlarged corner images were
inspected. Records: `oct23-architecture-replay.json` and its numbered smoke logs.

The final portable suite passes native execution and Terminal Psi, including
capacity/nesting failures, font intrinsics, bounded frame rebuilding, motion
retirement, list ranges and stale paint rejection. The independent Windows
editor/bitmap test and four development-tool tests pass. Records:
`oct23-architecture-library-tests.json` and `oct23-architecture-tools-tests.log`.
One extra EOF blank line was removed after compilation; the pipeline record
verifies the exact preceding source bytes and retains that applicability argument.

The prerequisite compiler cleanup repair has three passing regressions and an
affected-crate check. The crate suite is 8/9: its existing source guard reports
13 diagnostic-string violations in eleven unchanged files. The guard and all
reported files match the baseline exactly; this failure is not counted as green.
The attribution is retained in `oct23-affine-cleanup-validation.json`.

Continuous input plus rendering, milliseconds; median / p95 over 63 frames per
case across all three runs, retaining the slow second run:

| Drag | 1440 x 945 | 3840 x 2160 |
| --- | ---: | ---: |
| Slider | 1.070 / 1.173 | 2.903 / 3.347 |
| Root divider | 2.153 / 2.390 | 6.132 / 21.595 |
| Horizontal divider | 0.902 / 1.175 | 1.819 / 4.356 |
| Pane preview | 0.352 / 0.773 | 0.502 / 2.125 |

All default-size cases fit the 4.167 ms CPU budget. At 4K, root dragging and the
horizontal-divider tail miss it. Root p95 by run is 6.876, 26.668 and 6.665 ms;
none closes the goal. The earlier retained image measured root median/p95
5.577/6.336 ms across three runs, with a later identical-image replay at
6.919/10.185 ms. Keep that baseline and the new slower observations. These
aggregate changes include a compiler repair and are not a controlled attribution
to an individual UI change. `oct23-architecture-render-comparison.json` retains
every current case and its CPU-budget result. Actual presentation FPS and
input-to-photon latency remain unmeasured.

The native gallery build took 897.265 seconds: checking consumed 208.494 seconds
and Terminal production 590.191 seconds (589.101 in lowering). This compile cost
is a remaining workflow problem. The attempted live compiler sample did not
produce a usable profile; no hotspot claim is derived from it.

## Original baseline and experiment

The baseline is commit `0a0d99d`, native image SHA256
`c8f558e7b2386962e109840b7f06e73f63bf3ac21ce10e256691c94732ef92ae`.
Its three native runs measured 4K slider repaint median 26.449 ms, p95 28.495 ms
(90 partial frames, render portion). This misses even a 60 Hz frame budget.
The dock page also forced complete repaints, independently of what moved.

Hypothesis: the first avoidable cost is repainting and copying unchanged pixels.
Retain two-dimensional damage, invalidate the slider's moving pieces and the
dock's old/new painted surfaces, and clip both software writes and native text.
Copy and present only that rectangle. Keep full redraws as exact RGB oracles.
Test root-divider, horizontal-divider and pane-preview drags separately, including
release and drop-target changes. Reject the candidate if any paired image differs.
The 240 Hz render gate remains open until measured; live presentation is a
separate acceptance, not inferred from offscreen timings.

Detailed local records live under ignored `build/validation/`: the baseline is
`refresh-baseline.json`; the new native logs use `refresh-*`. The retained
compiler is the optimized main-based build with the existing descriptor-result
repair described in README.md. No compiler rebuild is implied by these records.

## Research log

2026-10-22: source inspection and primary-source research on the measured Windows
software redraw/presentation path. Queries:

- `site:learn.microsoft.com DwmFlush waits until next present flush synchronize frame`
- `site:learn.microsoft.com high frame rate GDI Direct2D performance dirty rectangles rendering`

Read:

- [DwmFlush](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmflush):
  blocks for outstanding application surface changes to reach presentation.
  Removing the wait alone cannot make 26 ms of drawing fit a 4.167 ms budget.
- [DXGI latency guidance](https://learn.microsoft.com/en-us/windows/uwp/gaming/reduce-latency-with-dxgi-1-3-swap-chains):
  a waitable flip chain schedules input/rendering when the next frame can be
  accepted. This is a possible presentation-owner change, not proof of a win here.
- [Direct2D overview](https://learn.microsoft.com/en-us/windows/win32/direct2d/direct2d-overview):
  a hardware renderer is a possible later path if bounded repainting remains over
  budget. No hardware-backend performance claim is made for this software renderer.

Next comparison: exact partial/full frames and CPU drag costs on the same native
image, with builds and captures outside the timed region.

## First candidate: rejected

Native image `513b8427d0a1f3dd06a1bf939d927deb1b9e3cdd07239c4705db0c22f12b0829` (`refresh-final-smoke-1.log`): 42/46
exact RGB pairs matched; four failed because slider preset selections were not
invalidated. All 18 dock partial/full pairs matched. At 4K the root and horizontal
divider medians remained 37.379 and 32.423 ms; pane-preview median was 5.164 ms.
No speed or completion claim is retained from this candidate.

The next hypothesis removes two demonstrated costs: disjoint changes joined into
a large rectangle, and repeated dock layout/painting on vertically identical
scanlines. Prepare the existing box primitives once, reuse only rows proven flat
by their geometry, and compare against the unchanged scalar full repaint.

The initial row-array implementation was not benchmarked: package checking was
cancelled after roughly 945 CPU seconds without a result. Its compile-cost
attribution is inconclusive. The replacement stores at most 32 rectangles and
34 words of row flags, instead of two 2160-element extent arrays and 2160 flags.
It produces the same disjoint physical regions; overflow uses a bounding union.

2026-10-23: direct primary-document lookup (no search query) for the input path
when a held drag leaves the client area:

- [SetCapture](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setcapture)
- [ReleaseCapture](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-releasecapture)

Read both API contracts. Capture keeps held-button mouse input routed to the
window outside its bounds; release restores normal routing. The current gallery
has no capture call and can fall back to its 16 ms idle poll outside the window.
This is an additional source-level limit, not the measured in-window raster cost.
A capture lifecycle remains a separate input-path repair and needs a real-window
check; no desktop input has been injected for this experiment.

## Prepared rows: correct, budget still missed

2026-10-23 native image `986e59e822a30bf3d42c45b50f75729db188cfdceadfda616c26fe6483ede9d4`: three runs
passed all 46 exact RGB pairs each, including preset changes and all panel pairs.
At 1440 x 945, slider drag median/p95 was 1.532/2.026 ms; root splitter
8.172/14.673 ms, horizontal splitter 3.999/5.015 ms, pane preview 1.246/4.542 ms.
At 4K, slider drag was 11.687/14.145 ms; root splitter 30.502/36.544 ms, horizontal
splitter 14.128/17.285 ms, pane preview 3.720/17.596 ms. Counts, tails and image
identity are in `build/validation/refresh-prepared-render-comparison.json`.
These are layout plus flushed offscreen rendering, including release. They do
not establish displayed FPS. The 240 Hz acceptance remains open.

The native build took 1070.517 s: source checking 698.370 s and Terminal Psi
lowering 271.235 s. This is a separate iteration-cost regression; source and
terminal work dominate that build, not native image publication.

Next hypothesis: cache the existing corner color/coverage samples per radius and
style, then blend the cached samples directly. Keep exact integer coverage and
the scalar full repaint oracle. Reuse a flat dock background only after checking
that no node paints there. Skip opaque slider interior distance calculations.
This is a compound candidate; any measured gain belongs to the complete change,
not to an unmeasured individual kernel. Also replace noncopy prepared rectangles
with scalar copy records and complete the held-pointer capture lifecycle.

2026-10-23: direct primary-document lookups for the exact CPU sample cache:
[CreateDIBSection](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-createdibsection),
[RtlMoveMemory](https://learn.microsoft.com/en-us/windows/win32/devnotes/rtlmovememory),
and [GetDIBits](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-getdibits).
Read their allocation, CPU-access synchronization, overlap and scan-format rules.
The cache bitmap is never selected for GDI drawing; the existing frame bitmap
still flushes GDI before CPU writes. The native fixture proved bounded writes and
reads, including invalid lengths and a null address. Its first failure was a
fixture scanline mismatch: source scan 7 reads the top memory row of its 8-row
bitmap. Direct span reads already contained the expected colors. The corrected
native run exits 0 (`refresh-cache-host-run.log`); a clean rerun is still required.

Direct lookup: [GetCapture](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getcapture)
and [WM_CAPTURECHANGED](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-capturechanged).
Capture loss is delivered to WindowProc; the gallery polls messages rather than
owning that callback. The repair therefore also queries capture ownership during
a held drag and cancels stale active controls without attempting to recapture.
The library's fresh native/interpreter pass is in `refresh-cache-library-both.log`;
the subsequent host addition only queries capture and is unreachable from those
Console-only library checks. Core library sources and the tested program are
unchanged, so their runtime evidence remains applicable.

Before timing the sample-cache candidate, define its primary CPU gate on
continuous input updates: `STREAM_FRAME_US` (slider indices 4..63) and
`PANEL_STREAM_FRAME_US` (indices 4..24), including release. No captures or forced
full repaint intervene between these updates; slider motion advances by 4/5 ms
steps averaging 240 Hz. Retain the paired-image workload and all its slower
measurements as quality and comparison evidence. Neither path measures displayed
FPS. This separates actual drag work from the deliberately disruptive full
repaint oracle, without discarding the earlier valid measurements.

The prepared candidate also regressed full cached-dot median from 8.689 to
16.663 ms at normal size (27.578 to 55.170 ms at 4K), and search edits from
5.962 to 10.892 ms (17.860 to 28.587 ms at 4K). It is not retained as a completed
performance fix. The next candidate removes repeated clip-helper calls from
horizontal spans and disables horizontal clipping metadata for full-width bands.
Timing spread remains recorded; no individual-kernel attribution is claimed.

## Exact sample cache: correct, CPU gate failed

2026-10-23 image `ed2d1324d658f88f5377e7b0e15e947e7aa4d496fea70d87ea5b8a91b6287f3d`: all 138 RGB pairs
passed. Native observations confirm the corner cache, flat-background path and
row reuse were used. Continuous normal-size slider/pane-preview p95 values fit
4.167 ms, but root splitter p95 was 10.177 ms. At 4K, continuous slider p95 was
5.863 ms, root splitter 25.508 ms, horizontal splitter 11.351 ms and pane preview
5.997 ms. Full dot/search work also remains above its original baseline.
The complete data, counters, source identities and failed CPU gate are retained
in `build/validation/refresh-cache-render-comparison.json`. No display-FPS claim.

The next probe replaces large scalar span writes with native copies from bounded
uniform-color rows. It preserves exact RGB and uses the already-tested bounded
bitmap-copy API; small edge blending stays unchanged. Also remove helper-call
chains from hot canvas-dimension lookups and avoid optional clipping calls in
full-width spans. Reject if pixels differ or the continuous CPU budget still fails.

The uniform-row probe uses a CPU-only 3840-by-24 DIB (368,640 bytes) and pins
colors for the prepared frame. Allocation/capacity/copy failure retains scalar
painting. Its native startup check verifies metadata preservation, exact clipped
endpoints, unchanged neighboring pixels, a mismatched-style fallback, and an
empty span. Cache read counters must be observed during the real panel workload.
The core dimension checks cover default, minimum, maximum and oversized metadata,
including the independent 64-by-64 storage mode. These changes are pending runtime
validation; their existence is not evidence of a performance improvement.

Stage medians from all three retained cache-candidate continuous runs isolate
the next target: root-divider layout is 0.047 ms at 1440-by-945 and 0.052 ms at
3840-by-2160, versus raster-plus-flush 8.229 ms and 21.737 ms. Horizontal-divider
layout is 0.036/0.046 ms versus raster-plus-flush 2.915/9.612 ms. Pane-preview
layout is 0.044/0.046 ms versus raster-plus-flush 0.976/1.959 ms. The source logs
are `refresh-cache-smoke-{1,2,3}.log`, stream indices 4..24. These medians are
separate stage distributions, not additive percentile claims. No layout rewrite
is justified by this evidence.

## Uniform native spans: correct, CPU gate failed

2026-10-23 image `a361127c0362920b4b6528cce0f4880b4962982391acc95c53679c6f1152a319`:
all 138 RGB comparisons pass, including the unchanged overlay background. The
native fill self-check passes, and copy counters advance in every dock scenario.
Continuous 1440-by-945 median/p95 costs in milliseconds are slider 1.732/2.590,
root divider 7.621/9.141, horizontal divider 3.760/5.098, preview 1.263/3.733.
At 3840-by-2160 they are 4.883/6.040, 19.247/25.983, 9.115/10.171 and
1.701/4.613. Cached full dots remain 12.896/42.873 ms median at normal/4K,
search 8.677/22.715 ms. These are not acceptance of the refresh goal or a
retained completed performance fix. Full records and source hashes are in
`build/validation/refresh-fill-render-comparison.json`. The compound change does
not isolate individual kernel gains. Native compilation took 1543.915 s:
949.489 s checking and 490.411 s lowering; observed memory reached at least 12 GB.

Next diagnostic: sample only a launched offscreen child to distinguish application
instructions from GDI/driver or native-copy work during continuous drags, without
another rendering rewrite. Diagnostic sampling perturbs execution; its timings
must not replace the untouched benchmark above.

Primary documentation read directly on 2026-10-23 (no search query):
[GetThreadContext](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getthreadcontext),
[AMD64 CONTEXT](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-context),
and [SuspendThread](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-suspendthread).
Use the architecture-specific aligned context and suspend the child thread only
while reading registers, always resuming in a finally block. This is a debugger
sample, never synchronization or control of the owner's interactive application.

## Attribution and next source candidate

Three corrected sampling runs exited 0 (`refresh-fill-profile.json`). In the 4K
root-divider stream, 217/281 samples were in the application image; the rest
were Windows modules or unresolved addresses. Module inventories in two runs
were captured before delayed DLL loading, so unresolved samples remain unresolved.
These perturbed samples include inter-frame logging and do not establish stage
durations. The first sampler attempt hit a terminating-thread race after the
child's PASS; that attempt is rejected, and exit handling was corrected.

The disassembly contains many 64-bit identity moves and forwarding branches.
A temporary byte-level diagnostic threaded 18,436 branches and replaced 122,560
64-bit identity moves; 32-bit self-moves, which zero-extend, were preserved.
All 138 image comparisons passed, but the patched image has no valid producer
realization evidence and is never retained as a product. At 4K its continuous
slider median/p95 was 3.400/4.141 ms, root divider 15.186/21.715 ms, horizontal
divider 6.680/8.038 ms and preview 1.223/3.547 ms. Thus branch overhead matters
but does not close the panel goal. Original image `a361127c...` was restored and
hash-verified (`refresh-branch-diagnostic-patches.json`).

Single-run removal probes deliberately omitted rendering and are not quality or
shipping results. Removing prepared dock painting left 4K root median/p95 at
7.753/12.386 ms. Removing text left 15.172/19.507 ms; removing both left
4.516/5.635 ms. At normal size, removing text changed root median from 7.621 to
3.794 ms; removing both left 0.328 ms. Results are in
`refresh-removal-diagnostics.json` and `refresh-stage-removal-diagnostics.json`.
Interactions and different runs prevent additive stage attribution. Both probes
restored and hash-verified the original image. No compiler repository was edited.

Next source candidate: paint text once through the exact union of all CPU-written
regions, retaining per-strip text if region allocation/combination/selection fails.
The union follows actual rendered strips, including their conservative horizontal
overdraw. Modal clipping restores that union. Also bound normal geometry before
using plain physical arithmetic, keeping the original general arithmetic outside
those bounds. No approximation to colors or coverage is accepted. This compound
candidate must pass 138 exact comparisons, the continuous CPU gate, and unchanged
headless library controls; its individual gains will not be claimed separately.

Primary documentation read directly on 2026-10-23:
[CreateRectRgn](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-createrectrgn),
[CombineRgn](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-combinergn),
and [SelectClipRgn](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-selectcliprgn).
The destination can alias one union input; selection copies the region. The fresh
native fixture verifies a disjoint union, its empty gap, and continued clipping
after deleting the original regions (`refresh-text-host-run.log`, native exit 0).
Scaled pixel controls cover negative coordinates, one-pixel dimensions, clipped
and unclipped rows, and the general path at three canvas sizes. Runtime results
for the new renderer remain pending.

Source inspection also found a region-level gate rejecting the prepared dock path
when a dirty rectangle starts above logical y=138. Pane padding can start it at
y=136, disabling native background fills and row reuse far below the toolbar.
The new candidate applies the y=170 empty-canvas cutoff per scanline and requires
the checked flat background for row reuse. It retains the sidebar, palette,
overflow and scalar-oracle exclusions. This joins the same compound candidate;
no isolated gain is claimed before measurement.

2026-10-23, compiler refresh requested by the owner: fetched remote `main` at
`0dce4dc599e2596206d3ae5ce209729ec3800b6b` (593 commits beyond the shared
checkout). The shared checkout contains overlapping staged and unstaged work,
so it was preserved. A clean managed checkout at the fetched revision built
the release compiler in 5m48s with `mbx build -p omega --release`.
`build/compiler.json` records opt-level 3, an empty working-tree status and SHA256
`aeea407a8a224b719ff9d5de1d3c06104ff025e886cb427e893ff727667dea19`.
Subsequent UI checks use that executable; earlier measurements retain their
original producer identities. Gallery acceptance and new performance results
are pending. The initial main review was cancelled before any diagnostic after
source inspection identified the same missing positive-row guard; its log is
`refresh-main-review.log`. The corrected source is checked separately.

The previous producer refused the text candidate's row-reuse call because its
guard did not establish `py > 0` at `check_flat`
(`refresh-text-gallery-update.log`). That candidate has no gallery runtime
result. The geometry library's native and Terminal runs both exited 0
(`refresh-text-library-both.log`), and the four Python oracle tests passed.
The helper now rejects an interpreter decline even when the compiler process
exits 0, preventing native-only evidence from masquerading as dual execution.

The corrected gallery review with clean main was cancelled after 1,897.481 wall
seconds and 1,815.219 CPU seconds without a semantic verdict
(`refresh-main-review-baseline.json`). Its source hashes and original compiler
are retained; `build/main-latest-clean/omega.exe` preserves the clean executable.
A brief, perturbing 80-sample instruction-pointer diagnostic resolved 27 samples
inside arithmetic `Engine::substituted`, 34 in `memmove`, and the rest mostly in
allocation. Those copy samples have no caller attribution and are not added to
the substitution count (`refresh-main-review-profile.json`).

Source inspection found `next = next.add(&piece)` cloning the entire accumulated
polynomial for every substitution term. A local compiler candidate merges each
piece into the existing accumulator instead, preserving coefficient addition and
zero cancellation. Its exact-arithmetic unit test passed with coefficients larger
than 64 bits. The workflow target is the same gallery review within 300 seconds;
the candidate and native gallery results remain pending. This is a local change
on fetched main, not a claim about unmodified main or published compiler behavior.

The first local compiler candidate, SHA256
`e82a6eb16be0be8117e081599c80c699e7259a9f7799f56480d4b0bfe0a1740f`,
accepted the nonlinear square proof and rejected its false twin with the same
diagnostic as clean main (`refresh-main-proof-controls.json`). It missed the
300-second review limit without a verdict (`refresh-main-accumulation-review.json`).
This establishes neither gallery acceptance nor a speedup. Its patch and a second
brief sample are retained as `refresh-main-accumulation.patch` and
`refresh-main-accumulation-profile.json`.

The next compound candidate also avoids rebuilding a polynomial when none of its
atoms has a substitution. Noncanonical zero powers and coefficients still take
the original normalization path. The existing eight-round limit and exponent
overflow behavior remain. Direct tests cover untouched atoms, matching expansion
and cancellation, zero-power normalization, the round limit, and overflow fallback.
Results for this candidate remain pending.

The second local candidate, SHA256
`81d8b5ab2492726a6788fbe6ee4d69cfba283402ffe3364b47c7e4ff1f71b335`,
passed three direct arithmetic tests and the same positive/reject proof controls.
Its review also exceeded 300 seconds without a verdict
(`refresh-main-substitution-review.json`). A separate quality run was stopped
after 597.613 seconds once caller sampling identified the next copying site
(`refresh-main-quality-review-stop.json`). Neither run establishes gallery
acceptance or a speedup. Its source patch is `refresh-main-substitution.patch`.

Memory-copy diagnostics did not support the large-generated-name hypothesis:
the later sample captured small copies, including 8--21 byte register lengths.
An early sample chose an idle thread, and the next active-thread sample was in
member lookup before the arithmetic phase. Those inconclusive records remain
separate. The memory-read sensor passed a known 8 MiB backslash-buffer self-check;
the virtual-environment launcher and a symbol-name filter initially obscured that
check (`refresh-copy-sampler-selftest.json`,
`refresh-copy-sampler-base-selftest.json`). No atom-name encoding was changed.

Twelve later caller stacks placed substitution below hypothesis installation
during call-requirement checking (`refresh-main-review-stacks.json`). Retained
bounds and newly supplied comparisons are already owned there, but were passed
through the borrowing, cloning API. The next local candidate transfers those
values through the same substitution algorithm. Four direct tests passed,
including retention of an unchanged atom's storage. Release and gallery results
are pending. All premise ordering, normalization rounds and overflow behavior
remain; no proof or consistency check is removed.

2026-10-23 diagnostic research: opened and read Microsoft's
[StackWalk64](https://learn.microsoft.com/en-us/windows/win32/api/dbghelp/nf-dbghelp-stackwalk64),
[STACKFRAME64](https://learn.microsoft.com/en-us/windows/win32/api/dbghelp/ns-dbghelp-stackframe64)
and [ADDRESS64](https://learn.microsoft.com/en-us/windows/win32/api/dbghelp/ns-dbghelp-address64)
documentation to recover callers above instruction-pointer samples. Applied the
AMD64 machine type, flat addresses, a mutable context copy and default process
memory reader. This is diagnostic attribution of this session's compiler only,
not a renderer or display-rate measurement.

The owned-substitution candidate (`03c064...`) was stopped after 1,203.824 wall
seconds without a gallery verdict (`refresh-main-owned-review-stop.json`); its
four arithmetic tests and proof controls passed. This remains inconclusive.
The next candidate deduplicates identical intrinsic integer-carrier comparisons
while preserving the first occurrence, distinct values, operators and bounds.
All 13 checking-crate tests passed (`refresh-main-carrier-tests.log`). The
checking-only field-contract controls preserve their original declarations and
prove the positive/reject distinction with both clean main and the candidate;
the original executable fixture initially needed package setup and is not
counted as a native run (`refresh-main-carrier-focused-controls.json`).

With candidate `ab774c65...`, the headless library project review completed in
70.417 seconds. Its 334 pending metadata decisions comprised 328 schema-only
changes and six filesystem services already present in unchanged standard-library
source (`refresh-main-library-policy-diff.json`). Those exact decisions were
accepted, and `update --offline --resume` published three packages for one target.
This proves project acceptance, not execution or the gallery's review target.
The subsequent native build refused descriptor reconstruction at
`legalization/scalar_graph_input/byte_views.rs:423`, machine 24, operation 1080
(`refresh-main-library-both-sites.log`). Source inspection found the same
runtime-selector exact-address failure addressed by the earlier UI producer.
The new main-based candidate reuses that small repair: unknown coordinates do
not introduce exact scalar facts, and the existing typed write invalidation still
runs. Native/reference replay and gallery measurements remain pending.

The descriptor candidate built in 2m49s (release opt-level 3, SHA256
`cc52e624d068ff83ebe6d8310f0d79dd8cd8eb9a9fe9d1f3ff6345d4f1d5fcef`).
After refreshing acceptance for its changed compiler identity, the unchanged
headless library test reported its authored PASS, native exit 0 and interpreter
exit 0 (`refresh-main-descriptor-library-both.json`). The exact four-file compiler
patch is `refresh-main-descriptor.patch`; this is fetched main plus local repairs,
not unmodified main. The additional claim note could not confirm publication
because its coordination push timed out; isolated work continued without retrying
that uncertain remote mutation. Gallery acceptance and performance remain open.

A new 12-stack diagnostic during the gallery review again reached hypothesis
installation below call-requirement checking
(`refresh-main-descriptor-review-stacks.json`). These are perturbed inclusive
samples, not elapsed-time percentages. The diagnostic initially reused the old
stack filename and overwrote its earlier raw capture; that old raw file is no
longer retained. The correction is recorded in
`refresh-main-stacks-record-correction.json`, and future captures require an
explicit output path. Earlier summary conclusions are not a replacement for
that missing raw capture.

The descriptor candidate's gallery review was stopped after 377.172 wall seconds
and 373.250 CPU seconds without a semantic verdict
(`refresh-main-descriptor-review-stop.json`). It misses the 300-second workflow
target; no gallery speedup or acceptance is claimed. The library-verified compiler
is preserved under `build/main-latest-library-verified/`. A temporary, environment
gated size diagnostic will distinguish large hypothesis collections, polynomial
expansion and long atom names before another optimization is selected.

2026-10-23 diagnostic: candidate `50e723fe...` with temporary size tracing was
stopped at its 180-second limit without a verdict. It reached an incoming batch
of 326 intrinsic comparisons; reported polynomial maxima were one term, power
one, and 796-byte names (`refresh-main-size-trace-review.log`). This does not
establish that larger polynomials never occur later. The next experiment replaces
string-keyed lookups inside difference closure with indexed scratch rows, retaining
the exact atom order, iteration order, first-edge snapshot, diagonal treatment
and contradiction detection. A differential unit oracle will compare complete
matrices, including cycles and coefficients beyond 64 bits, before gallery replay.

Research for this specific path: opened LLVM's official
[PolyBench Floyd-Warshall implementation](https://github.com/llvm/llvm-test-suite/blob/main/SingleSource/Benchmarks/Polybench/medley/floyd-warshall/floyd-warshall.c)
and read the dense indexed `kernel_floyd_warshall` loop. Its ordinary numeric
shortest-path benchmark suggests storage access, not semantic authority for this
exact-integer longest-bound checker. No floating approximation or changed closure
algorithm is proposed; the real gallery result decides whether this path matters.

The indexed-closure candidate passed all 14 checking tests. Its differential
test compared 576 generated matrices twice each (1,152 passes), preserving the
full map and contradiction flag across missing edges, diagonal entries, positive
cycles and large exact coefficients (`refresh-main-indexed-tests.log`). Dense
scratch is limited to 1,024 atoms; larger sparse rosters keep the symbolic walk.
Temporary size tracing is removed. Release and full-gallery results are pending.

The indexed candidate (`8c01bb53...`) completed current-source candidate
compilation in 255.354 seconds, including 209.151 seconds for the gallery.
The complete review still missed 300 seconds and was stopped after 802.598 wall
seconds during historical-source recovery, without project acceptance
(`refresh-main-indexed-review-stop.json`). A second stack capture identified
bounded routing-metadata reads in the old local snapshot scan
(`refresh-main-indexed-policy-stacks.json`). The per-user collection contained
27,832 snapshot directories. Each candidate reopened and reverified absolute
ancestry, although the collection was already retained.

The next repair reads bounded routing metadata relative to that retained parent.
It preserves no-follow child identity, ownership/mode and macOS ACL checks,
verifies the collection before and after the walk, and retains the selected
snapshot's original lock, full custody walk and exact source rehash. No cache
entry, archive or routing index is created or removed. The first replacement
test could not rename an open directory on Windows (sharing violation), so its
portable version supplies a different retained directory identity directly;
the failed setup is retained in `refresh-main-retained-recovery-tests-final.log`.
The six original/focused recovery checks passed before that setup correction.
Final custody tests and gallery replay remain pending.

All 14 selected package-source recovery/custody tests passed after the setup
correction (`refresh-main-retained-custody-tests.log`), with 95 tests outside
the selected scope. They include collection identity, oversized routing metadata,
wrong origins, changed/deleted live sources, corrupted source trees, limits,
publication destinations, lock replacement and forbidden Git records. These are
Windows observations; Unix symlink and macOS ACL cases were not run here.

The retained-parent candidate built in 52.94 seconds (release opt-level 3,
SHA256 `ca193b07bf58507e2d6d819141ea751fc6c3b4628810c49561193a76c1fc93b2`).
Its gallery review completed in 277.358 seconds with exit 3 for pending decisions;
candidate compilation consumed 231.629 seconds. All three historical source
diffs were recovered (two standard-library entries, one library entry, three
gallery entries). This is completed checking and review generation, not accepted
project publication. The 334 per-row metadata deltas exactly matched the already
inspected library proposal, so only those decision tokens were changed to accept
(`refresh-main-retained-gallery-review-decisions.json`). Resume is pending; the
300-second target is not claimed as complete acceptance.

Routing research: the docs.rs pages for pinned `cap-fs-ext`/`cap-std` 4.0.3
were unavailable to the browser. Read the official pinned
[DirExt implementation](https://github.com/bytecodealliance/cap-std/blob/v4.0.3/cap-fs-ext/src/dir_ext.rs)
instead, including its no-follow directory operation and delegation through the
retained file handle. This supports the API choice; repository custody checks
and executable tests establish this integration's behavior.

The resume pass published `build.omg` and `omega.lock` for all three packages
on Windows x86-64 (`refresh-main-retained-gallery-resume.log`, exit 0). It
carried the two dependency packages and rechecked the gallery; candidate
compilation took 198.527 seconds. Initial review plus resume exceeds 300 seconds,
so the complete workflow target remains open despite both source-check passes
finishing below that threshold. The canonical native gallery build is next.

The gallery compiled and published successfully in 590.265 seconds. Source
checking took 179.412 seconds and Terminal production 288.263 seconds, including
287.083 seconds of lowering (`refresh-main-retained-gallery-compile.log`). A
perturbing stack sample located work in copied-payload membership retention
(`refresh-main-retained-native-stacks.json`); it is not a stage-duration measure.

Native image `4f55ed03c893f1a2fc9e7557adb2b324c29b1f79e2fcfd2f82baf148a93aba18`
passed all 138 RGB comparisons, launcher-background checks and required cache/text
path observations over three runs. The 240 Hz CPU gate failed
(`refresh-main-retained-render-comparison.json`). At 4K, continuous median/p95
milliseconds were slider 3.156/4.946, root divider 16.963/21.189, horizontal divider
7.084/8.440 and pane preview 1.378/5.043. Full cached repaint median remained
38.991 ms and search 17.576 ms. These renderer costs exclude compositor delivery;
actual display FPS is unverified. This is a correct native candidate with partial
improvements, not refresh-rate completion.

Next hypothesis: prepared dock painting still scans every recorded box on every
scanline, constructing rectangles even when a box cannot overlap. Index active
commands per row and retain ascending command order. Keep the same drawing
primitives and exact RGB oracles. Reject the candidate if an index self-check or
paired image differs; compare the same continuous drag workload afterward.

The row-index experiment is rejected for production. Native image
`a5d4a4bf1f423ace74fce96a4d8ae9ab244ed4291795e44a1a9f74555d6233c4`
passed all 138 RGB comparisons, all command-index controls, background checks
and required cache/text observations (`refresh-row-index-render-comparison.json`).
Separately viewed the matching 4K root-divider fast/reference images at the same
scale. At 4K, continuous root-divider median/p95 improved to 15.250/17.020 ms,
but slider median increased to 3.526 ms; at 1440x945 root-divider median increased
from 6.446 to 9.960 ms. The three candidate runs ranged from 14.202 to 18.296
seconds, compared with 13.889 to 14.518 seconds for the preceding renderer.
These were sequential candidate batches, not alternating trials, so attribution
is inconclusive. No dependable end-to-end benefit justifies keeping the extra
row index. Its two source files were restored byte-for-byte to the preceding
verified renderer; the diagnostic patch and decision remain under
`build/validation/refresh-row-index-{rejected.patch,decision.json}`.

The row-index review completed in 390.129 seconds and native publication in
625.169 seconds, including 208.039 seconds of source checking and 303.335 seconds
of Terminal production. Both the five-minute workflow target and 240 Hz CPU
target remain open. No display-FPS claim follows from these offscreen runs.

Final compiler checking controls also passed for producer `ca193b07...`:
nonlinear-square and field-contract positives accepted, with their two invalid
controls rejecting with the expected diagnostics
(`refresh-main-retained-focused-controls.json`). The field cases exercise the
original Countdown contracts in checking-only wrappers; they do not claim
native execution of the complete original fixtures. Library native/Terminal
agreement is recorded separately in `refresh-main-retained-library-both.json`.

The restored renderer's final project review published all three packages in
211.594 seconds (`refresh-final-review.json`). This observed review fits the
300-second threshold; it does not make the complete native iteration fit it.
Native rebuild/publication took 596.599 seconds, including 186.953 seconds of
source checking and 300.588 seconds of Terminal production
(`refresh-final-gallery-compile.json` and its log).

The canonical `build/gallery/omega-program.exe` is byte-for-byte identical to
the previously verified renderer (`4f55ed03...`), produced by the same optimized
compiler (`ca193b07...`) from the same recorded source bytes. Its existing
138-pair and timing observations therefore apply without repeating that batch.
An additional fresh launch passed 46 RGB pairs, 27 tab images and preserved
launcher background (`refresh-final-replay.json`). This checks the newly
published artifact and resources. It is not a new display-FPS measurement:
4K root-divider p95 remains the recorded 21.189 ms against a 4.167 ms CPU budget.

The standalone Windows adapter regression also passed with this producer
(`refresh-final-native-host.json`): initial UTF-16 content, selection/replacement,
undo, window destruction, bounded bitmap span reads/writes, invalid arguments,
and disjoint region clipping. Its bitmap assertions are unchanged. The editor
buffer now stays in one owning state throughout its sequential operations.
The earlier drivers were refused during Terminal production with `captured loan
forwarding changes descriptor type`; both failed attempts remain recorded in
`refresh-final-native-host-refusal.json`. This test cleanup does not repair or
qualify that broader compiler limitation. The standalone test is outside the
gallery's dependency closure, so its driver changes leave the verified gallery
image and source inputs unchanged.

2026-10-23: refreshed main to `f7da4c386ebf` and rebased the three retained compiler
repairs cleanly. Release compiler `434b70cf...` built in 288.145 seconds at opt-level
3. The synthesizer now derives native title/value placement from the same header
rectangle used for its icon. Its redundant hard-coded 60 Hz caption is removed.

The real pane-corner oracle is `python tools/check_outline_snapshots.py
build/gallery`. Before the change, cached and uncached frame-20 captures at both
1440x945 and 3840x2160 differ from one fill/stroke composite by 10 channel levels
(allowed 3), while straight strokes match. The old pane paints a rounded body,
rounded header and border separately. The candidate paints each row's fill and
stroke together, with disjoint header/body clips in the prepared painter.

The drag experiment batches consecutive identical scanlines after one destination
range check; varying rows retain the existing painter. Native adapter controls
cover row count, source bounds, missing storage and destination-address wrap.
Research query: `site.learn.microsoft.com RtlMoveMemory routine Length source
destination memory blocks`; read Microsoft's
[RtlMoveMemory contract](https://learn.microsoft.com/en-us/windows/win32/devnotes/rtlmovememory),
which specifies a byte count and permits overlap. The implementation reuses the
existing import. Its aggregate range checks remain outside the copy loop.
Retained: native adapter controls pass, including destination-address wrap;
library tests agree at exit 0 in native and Terminal execution. Three replay
runs pass all 138 RGB pairs. All 12 real pane-corner probes improve from maximum
channel error 10 to 1 (allowed 3). Separate matching-scale inspection confirms
the synthesizer header alignment and single-composite pane corners.

Continuous root-divider CPU p95 improves from 8.179 to 5.443 ms at 1440x945,
and from 21.189 to 10.692 ms at 3840x2160. The latter remains above the 4.167 ms
240 Hz budget. These are aggregate observations across the compiler refresh,
requested visual corrections and row batching, not an isolated per-mechanism
speedup or display-FPS result. The 4K horizontal divider p95 is 4.215 ms.
`build/validation/oct23-corners-pipeline.json` binds producer, source inputs,
checks and canonical executable `40a263ea...`; detailed results are in
`oct23-corners-render-comparison.json` and `oct23-corners-outline-{1,2,3}.log`.

Follow-up instruction-pointer sampling identifies application code in 125 of
168 samples during the 4K root drag, ntdll in 29, and unknown addresses in 14.
Sampling perturbs the process, so these counts are attribution evidence only,
not timing results (`oct23-pane-profile.json`). The next question is remaining
application raster/dispatch work; raw copy tuning alone is unlikely to close
the gap.


The next shared-edge control reproduces a separate axis-join halo in native
and Terminal execution (`oct23-box-alignment-red.log`): the corner's 1.25-pixel
Hermite filter does not match the opaque straight one-pixel run. The candidate
uses a one-pixel coverage ramp for rounded boxes in scalar, row and cached
corner paths; segment/checkmark/slider motion antialiasing stays separate.
Library review and native/Terminal exit-0 agreement pass with the new control
(`oct23-alignment-pipeline.json`). Retained gallery results: all 138 RGB pairs
pass, and all 12 pane-corner probes have maximum channel error 0 (allowed 3).
The old filter fails this updated physical-profile oracle at error 9/8 for the
default/4K captures (`oct23-alignment-outline-red.log`). Separate image inspection
covers the magnified pane corner and full light-theme synthesizer hover state.

Next drag hypothesis: remove per-row command redispatch below the empty dock
canvas cutoff. Paint commands in order directly into the existing DIB, copying
opaque straight interiors in batches and reading destination rows before
antialiased corners or transparent strokes. Keep dirty-rectangle and header/body
clips, and repaint the entire affected region with the scanline path on copy
failure. Reject the candidate on any RGB mismatch or unsupported timing benefit.
Research query: `site.learn.microsoft.com DIB section direct access bitmap pixels
GdiFlush synchronization`. Read Microsoft's
[CreateDIBSection remarks](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-createdibsection):
direct memory access must follow completion of GDI writes. The existing frame
entry flush and CPU-before-text phase order remain intact. The related Old New
Thing article returned 403; no additional claims are based on it.


Retained rectangle-dispatch iteration: the same producer compiled it in 540.611
seconds after a 175.457-second project review. At 1440x945, continuous root drag
median/p95 is 2.089/3.762 ms; at 4K it is 6.166/9.588 ms, compared with the prior
row candidate's 4.950/5.443 and 10.037/10.692 ms. All three 4K run medians improve
(8.151, 5.512, 5.734 ms); tails vary substantially, and the independent slider and
full-repaint controls also slow in some trials. Preserve all valid runs; do not
attribute their contention to the rectangle change or claim 240 Hz. The 4K
horizontal divider median/p95 is 1.624/2.878 ms and pane preview is 0.482/1.790 ms.
`oct23-alignment-render-comparison.json` contains every case and CPU-budget result;
`oct23-alignment-pipeline.json` binds the source and executable. Zero flat-row and
reused-row counters throughout these pane runs confirm that the rectangle path
handled them without falling back to scanline planning. Fill-cache reads fall
from 830293 to 220568 over the same first replay. The shared AA correction is
included in this aggregate comparison; warm cached corner sampling is unchanged.

The remaining source-level waste is `prepare_rows`: it still constructs the
scanline-varying bitset even though successful rectangle rendering never reads
it. Next candidate builds that plan only for an actual scanline path or bitmap
failure, preserving the existing fallback. Same producer and pixel/profile
controls apply. Retained: source review/native compilation pass in
142.239/537.508 seconds; three native replays pass all 144 RGB pairs, including
two forced destination-layout refusals per run. Both fallback frames reproduce
the healthy frames exactly. All 12 corner probes still have zero error.
Continuous root drag median/p95 is 1.997/2.175 ms at 1440x945 and 5.577/6.336 ms
at 4K. Slider, horizontal divider and preview p95 at 4K are 2.884, 1.848 and
1.761 ms. All default-size drag gates pass; the 4K root gate remains red.
The timing controls also become steadier in this batch, so the large tail
improvement is not attributed solely to deferring the plan. Records:
`oct23-plan-pipeline.json`, `oct23-plan-render-comparison.json`.

The live loop paces once after all dirty regions are painted; its input wait
runs only while idle. Further CPU work is required for the 4K root divider.
The next candidate targets cached-corner samples: the current path scans
opaque runs and writes both mirrored spans through scalar loops on every
corner row, even though those pixels were already calculated in the cache.


Next bounded corner experiment: store mirrored corner samples and the first
checked contiguous opaque run when constructing a cached style. Copy that run
through the existing bounded bitmap reader and blend the remaining samples
normally. This removes both per-frame run scanning and scalar opaque-pixel writes
for the copied span. All copy failures retain the scalar painter. Cache growth
is about 375 KiB (98 rather than 48 columns); no new import or dependency is used.
A same-executable switch in the offscreen harness compares the old scalar run
path with bulk copies; default-size order is scalar/bulk, 4K order is bulk/scalar.
The experimental live build enabled bulk copies. Reject on RGB/profile mismatch
or lack of a supported drag-time benefit.
Research query: `site.learn.microsoft.com BITMAPINFOHEADER 32 bpp BI_RGB high byte
not used`. Read the 32-bit row in Microsoft's
[BITMAPINFOHEADER contract](https://learn.microsoft.com/en-us/previous-versions/dd183376(v=vs.85)):
the high byte is unused in BI_RGB, so opaque cached RGB can retain its packed
coverage byte during copying. This applies to this private CPU-only BI_RGB cache
and the current RGB blitter; it does not establish an alpha-blending contract.


Rejected: three native trials pass 144 RGB pairs and all 12 zero-error corner
probes, but the same-executable 4K root median is worse with bulk copies
(6.279 vs 5.652 ms); p95 moves in the opposite direction (8.444 vs 9.983 ms),
while unrelated timing controls also vary. Default root medians are effectively
unchanged (2.026 vs 2.038 ms). Smaller divider/preview gains do not justify the
extra cache size and machinery for the remaining root-drag objective. Preserve
all trials in `oct23-bulk-render-comparison.json`, its exact producer/source/image
in `oct23-bulk-pipeline.json`, and the rejected implementation in
`oct23-bulk-rejected.patch`. Restore both implementation files exactly from
`807aa1b`; keep the smaller rectangle painter and deferred planning.

The retained iteration fixes the shared rounded-box profile and header alignment
and substantially reduces drag cost. It does not establish the 240 Hz goal at 4K:
the retained root median/p95 is 5.577/6.336 ms. Further corner-copy tuning is not
supported by this experiment; the next useful investigation needs finer
attribution of the remaining application work. Real presentation FPS and
input-to-photon latency remain unmeasured.

Final restored publication passes source review in 175.536 seconds and native
compilation in 609.101 seconds. Its canonical executable is byte-for-byte
identical to the retained candidate (`f8d3a6fea902...`), with every recorded source
input matching as well (`oct23-final-pipeline.json`). This binds the retained
library/adapter checks and three-run pixel/timing observations to the final
published bytes; no rejected cache implementation remains in source or image.
A fresh native launch additionally passes 48 RGB pairs, both forced-fallback
cases, four zero-error corner probes and all 27 tab captures, with the launcher
background preserved (`oct23-final-replay.json`, `oct23-final-smoke.log`).
Separate final image inspection confirms the synthesizer header geometry and
corner profile (`oct23-final-synth-light.png`, `oct23-final-pane-corner.png`).

The fresh root-drag p95 is 2.174 ms at default size and 10.185 ms at 4K (median
2.002/6.919 ms). The latter is slower than the retained three-run p95 of 6.336 ms
on the identical executable. Keep both observations: the 4K tail is variable,
and remains over budget in every retained result. This fresh run is publication
and consumer evidence, not proof of sustained display FPS or 240 Hz closure.
