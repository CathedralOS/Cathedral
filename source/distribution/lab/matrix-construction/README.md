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
```

All three runs generate real input and require two successive parent polls while
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
