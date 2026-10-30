# Colors in Omega

This is the Colors program installed by normal F5, including copies inside
nested desktops. Enter toggles a persisted color; X closes it. Failed writes
keep confirmed pixels, turn the indicator red and require close confirmation.
Enter then reconciles committed storage before allowing another mutation.

Start at [app/main.omg](app/main.omg): resume completed service work, preserving
storage uncertainty while stopping on transport loss elsewhere. Descend into:

- [session/session.omg](app/session/session.omg): service acquisition and the
  receive/act/present loop; [input.omg](app/session/input.omg) tracks independent
  scoped pointer edges and handles Enter and close.
- [storage/storage.omg](app/storage/storage.omg): the app's six-byte `MC01`
  schema, creation, revision-checked writes and uncertainty reconciliation.
- [rendering/rendering.omg](app/rendering/rendering.omg): local surface lifetime
  and publication; [pixels.omg](app/rendering/pixels.omg) supplies the checker
  image and close glyphs. The confirmation button accommodates the whole label.
- [state.omg](app/state.omg): app-owned state and typed storage/rendering phases.
  [operations.omg](app/operations.omg) names completed replies, scoped input,
  requested work and app-owned geometry. No native packet offsets appear here.

The standalone executable starts at [native/main.omg](native/main.omg). Its
[process entrance](process/main.omg) establishes installed authority, runs the
app and releases custody. [application.omg](process/application.omg) alternates
completed work with app decisions; [operations.omg](process/operations.omg)
dispatches those decisions to service acquisition, storage, presentation and
waiting. Follow that dispatch into:

- [services.omg](process/services.omg): interaction and storage endpoint grants,
  including half-pair cleanup. The platform runtime owns grant validation.
- [storage.omg](process/storage.omg): bound requests and immutable `MC01` backing.
  The platform's [storage scope](../../../platform/libraries/user-runtime/storage/main.omg)
  encodes the child-local binding; app policy still owns retries and persistence.
- [presentation.omg](process/presentation.omg): drawing packets and image backing.
  Pixel staging ends before any wait; only the sealed handle stays live.
- [exchange.omg](process/exchange.omg): publish once, then service the independent
  owner connection while waiting. [control.omg](process/control.omg) answers
  HEALTH/LOSS without advancing the app. A pending save reports BUSY.
- [transfer.omg](process/transfer.omg): retain/release attachment custody and
  reject unexpected reply grants. [replies.omg](process/replies.omg) decodes the
  64-byte protocol into named app completions.

The common [Omega runtime](../../../platform/libraries/user-runtime/README.md)
provides startup, kernel calls, transport and waits. This is an x86-64 Omega
process under the Rust kernel, with a small reviewed native assembly adapter;
no Rust application host is linked. The process has no new ambient authority.
HEALTH's CPU counter currently reports zero because that diagnostic kernel call
has not been added to the Omega runtime. Other app counters and loss state remain
available. UI-library adoption is separate; Colors uses the existing rectangles
and sealed images directly.

The retained [component bridge](bridge/component/main.omg) and
[Rust entry](../../../../source-rs/distribution/applications/colors/main.rs)
provide a comparison fixture. They adapt the same app model through the earlier
host packet format. [Compiled-code tests](../../../../source-rs/distribution/applications/colors/tests/behavior.rs)
exercise this model's save, uncertainty, close and input policy.

Build the independent process with [tools/omega-user](../../../../tools/omega-user/README.md).
`colors_test.py` holds and rejects a real save, requires concurrent owner replies,
checks confirmed pixels and image backing, retires the owner independently and
checks resource refunds after cancellation. It also verifies idle parking and
draining multiple queued batches after one wake, with owner progress between
batches. The desktop harness additionally
checks successful saves and cold reopen; nested testing checks that inner and
outer Colors remain independent. Kernel-lab probes still use the old Rust library.
