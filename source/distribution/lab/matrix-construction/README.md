# Stock matrix construction observation

The native composition selects the real stock matrix and a one-second window
before it observes an ordinary child's construction result. The normal native
composition selects zero. Neither composition changes the kernel admission worker.
The child can commit during the window; cancelling then must collect it, not
merely discard the receipt. Pre-commit cancellation remains covered by the generic
admission witness.

The Rust test parent uses its existing owner connection: HEALTH, STOP, or a
LAUNCH_IMPORTS revoke/restore. The harness requires replies before the window
releases, then checks refusal or activation, slot reuse, saved data and cold boot.
A timeout or a missed overlap fails the test rather than counting as coverage.

Run from the repository root:

```text
python tools/boot-harness-rs/omega_construction_test.py --mode health
python tools/boot-harness-rs/omega_construction_test.py --mode stop
python tools/boot-harness-rs/omega_construction_test.py --mode regrant
python tools/boot-harness-rs/omega_construction_test.py --mode requester
```

All four runs generate real input and require two successive parent polls while
construction observation is held. One poll may already be queued; the second
requires the child to consume its predecessor and request more work. This proves
input-service progress, not input delivery to the not-yet-admitted application.
The regrant run first establishes an explicit ordinary-import policy, then revokes
and restores that exact policy during construction. The old launch must fail;
a fresh request must reuse the target successfully.

The fixture [process entry](process/main.omg) calls the stock matrix's
[entry](../../matrix/process/program/main.omg) with the observation window.
[Construction observation](../../matrix/process/children/construction/observation.omg)
is a small leaf in the existing children package. The normal entry passes zero,
so it neither reads a probe clock nor waits. This adds no kernel test policy,
service capability or change to the common program entry contract.

The requester case stops slot zero (the launcher) while its child construction
is held. The owner then resolves `Desktop` in that matrix's own realm and uses
RUN_PAGES, RUN_OBJECT, SURFACE_LIMIT and PLACE to install a fresh requester in
slot zero. It uses the stock template's 96 private pages and bottom-bar geometry.
This is explicit owner orchestration, not automatic launcher recovery. The old
launch must be refused before replacement completes; the new launcher must issue
a fresh successful request into the same app slot, then save and cold-reopen Notes.
Before reinstalling, the requester fixture supplies an invalid CPU ceiling to
RUN_OBJECT. This fails after a real reservation. Its construction-stage refusal
must precede the kernel-confirmed unused-reservation return, and the owner must
receive that refusal only after that return. The diagnostic is emitted only for
a successful return, not a BUSY reservation queued for later retry. The fixture restages its consumed allowance, then succeeds in the
same slot. Repeated rounds compare the usual resource accounts.

This exercises requester retirement during construction, deferred reservation
cleanup and orderly same-slot replacement, not arbitrary reordering of old
replies after replacement.

The Rust fixture entrance is
[construction_probe.rs](../../../../source-rs/distribution/matrix/service/construction_probe/construction_probe.rs).
Its [replacement leaf](../../../../source-rs/distribution/matrix/service/requester_probe.rs)
owns only the ordinary control-command sequence; `mod.rs` is plumbing.

Use `--rounds 4` with any mode to repeat the observation, owner/input overlap,
cancellation or release, successful launch, save and two Notes closes four times
per boot. Each close samples private pages, reader pages, charged frames and free
launch slots from the existing matrix reclamation report. The samples must match
the first close of that boot exactly, across all rounds. Notes accumulates one
character per round and must retain the complete fresh-boot text on cold reopen.
These are comparisons at the same reclamation point, not a claim that every
asynchronous reservation return has settled or that all provider memory is measured.
