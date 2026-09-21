# Incomplete Field-read transcription checkpoint

Status on 2026-09-21: the implementation and 70 authored behavior/control pairs
are retained for diagnosis. Actual successful Field-read execution is unverified.
The branch retains its original `50d8b3c1c904ff429b66f041a5c6daa83ad4569b`
ancestry; it does not include later independent main-branch milestones.

The exact [regression receipt](evidence/regressions213.json) records 213 passing
pairs: 79 integer execution, 55 generic execution, 22 Program pipeline, 14 bridge,
38 named-store bridge, and 5 added continuation-comparator pairs. The bridge
cases seed the new read mode and deferred-read fields, and the added controls
independently change every new Frame member. All 134 bound input hashes remained
unchanged. The retained harness reproduces and verifies those generated bodies.
This establishes these assertions on the extended executor; it does not execute
the new read-session behavior.

The [original first read pair](evidence/original-tiny1-failed.json) completed
checked compilation, then both positive and control selections trapped with
`equality operands have no supported comparison` at 81,854 evaluator fuel units.
The [decoder-only experiment](evidence/lean-tiny1-failed.json) produced the same
trap at 81,822 units. Neither selection passed. Both used identical generated
fixture bytes and the same checked runner at audited Omega
`eaa7993a23623cd8fabf45350340479c5c9c7879`. The experiment changed only ordinary
Field recognition from `read_value` transport to transparent identity unwrapping
and direct kind inspection; its [patch](evidence/lean-decoder.patch) and source
hash map are retained. The production draft retains the original decoder.

The regression receipt is copied without rewriting its execution root, build
paths, source hashes, or harness identity. Failure records explicitly describe
their after-run hash checks and are not success receipts. The original and
experimental generated sources/build files are preserved in `evidence/`, with
copy hashes in [retention.json](evidence/retention.json).

Supplemental standalone diagnostics passed four
[admission selections](evidence/admission-micro/manifest.json) and six
[decoder/Runtime selections](evidence/decoder-micro/manifest.json), including
one changed-expectation control. Their generated sources, generators, build
recipes, manifests and complete output are retained byte-for-byte. These probes
directly seed the first fixture's metadata and flatten copied module bodies.
Admission uses real geometry helpers but omits loading and execution. The decoder
probe uses a synthetic `execution_turn` that directly calls the real copied
decoder; its excluded validation/contribution paths are explicit failing stubs.
Neither probe executes the complete Program session or proves Field-read
correctness. Their generators retain historical absolute input paths; the shared
declaration input is copied in `evidence/diagnostic-data/`, and the generated
standalone main/build files are sufficient for direct diagnostic replay.

The subsequent [dispatch diagnostic](evidence/dispatch-micro/interpretation.json)
retains five passing selections and one failed expectation, with the original
exit code 1 and full output. It adds the real dispatcher and named operand path,
but supplies the resolved name synthetically. The failed `parent_full` selection
reaches the deliberately failing retirement stub before decoding, so its inherited
decoder expectation does not describe that route. This mixed diagnostic provides
no complete Program or Field-read success claim.

Two authored defects are identified in the unchanged production draft:
`turn::pending` transitions directly to the separate machine `prepare`, and
`complete::word` transitions directly to the separate machine `received`.
Omega state transitions must remain within their machine; these calls need
local wrapper states. Whether they explain the earlier equality trap remains
under investigation. The full 70-pair run was therefore
[stopped as superseded](evidence/full70-stopped/manifest.json), before any
`CHECKED` or behavior output. Its exact generated main/build, 130 bound input
hashes, runner hash, process identity, empty pre-stop log, SIGTERM action and
reaped wrapper exit code 1 are retained. No full-corpus result is claimed.

At this checkpoint the external staged diagnostic remains in progress, and the
two production fixes are pending. Its future results are not included here. Frozen
production and fixture bytes remain unchanged by this documentation/evidence
checkpoint. No native execution, hardware/provider callback, grant, mapping,
lock acquisition, or Field write capability is claimed. ACPI-005 and whole
upstream `do_field_read` remain incomplete.
