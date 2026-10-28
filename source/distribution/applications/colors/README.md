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

The [component bridge](bridge/component/main.omg) decodes replies, invokes the
typed app and encodes its next command. `replies.omg` owns transport interpretation;
`commands.omg` owns the experimental platform wire formats; `images.omg` packs
app-authored pixels into attachment bytes. The six-byte schema lives with storage
in [record.omg](app/storage/record.omg), not in that native codec.

The [native consumer](bridge/native/main.omg) selects this bridge as an independent
Omega component. The [Rust entry](../../../../source-rs/distribution/applications/colors/main.rs)
declares the existing service imports and invokes the reusable
[platform host](../../../../source-rs/platform/libraries/omega-app/README.md).
Omega owns application behavior; Rust still supplies executable startup,
syscalls, service transport, waits and immutable attachment custody. This is
an x86-64 bridge into the reference OS, not a standalone Omega OS executable or
the final Omega application ABI. No new syscall, ambient locator or authority
is involved. UI-library adoption is a separate step; this app uses the existing
platform rectangles and sealed images directly.

Build/packaging starts at [tools/omega-component](../../../../tools/omega-component/README.md).
Compiled-code tests in the
[Rust wrapper](../../../../source-rs/distribution/applications/colors/tests/behavior.rs)
drive actual Omega exports against controlled service replies. The ordinary
desktop harness verifies the installed program, persistence and cold boot;
the nested desktop harness also saves Colors four levels down and checks that
the outer Colors remains independent. Kernel-lab probes continue using the
old Rust library; they are not the installed application.
