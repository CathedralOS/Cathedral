# Stock matrix configuration

The chooser and stock matrix share this distribution-owned Omega package.
Start at [main.omg](main.omg): saved bytes go through [encoding.omg](encoding.omg)
and canonical [name.omg](name.omg); geometry goes to [placement.omg](placement.omg).
This is the desktop's CTHMAT02 format and bottom-bar policy, not a universal
platform configuration or a source of authority. The owner still checks the
supplied grants before applying any result.

[live.omg](live.omg) plans an authorized live layout. It checks authority before
busy state, then canonical request shape and checked geometry. The launcher takes
the bottom bar, app positions take the workspace, and background takes the full
extent. Only existing app standing placements are selected for remembering.
The [host](../../../../../../source-rs/distribution/matrix/service/children/configuration/layout.rs)
rechecks the exact controller/incarnation/epoch in `World::layout`, validates all
rectangles before mutation, preserves revoked views and key masks, then remembers
placements, flushes and replays retained scenes. These synchronous resource
operations and failure propagation stay in the host; the Omega plan grants no
authority and does not make publication atomic across IPC hops.

The chooser calls the codec directly. The Rust stock matrix calls the compiled
export through [its policy entrance](../../../../../../source-rs/distribution/matrix/policy/policy.rs).
[native](../../../bridge/native/main.omg) packages that temporary
synchronous bridge: 128 u64 words; operation 1 normalizes 104 bytes stored one per
word at 16, operation 2 takes margin/bar/gap/width/height in words 1-5 and returns
two rectangles in 16-23. Success is 1, refusal 0. This is not an IPC wire format.

Rust's public data representation remains in the distribution contracts for
installer/tools and serves as the independent compatibility oracle. Compiled
conformance tests compare every single-byte mutation, resource and legacy bounds,
UTF-8 names, and 100,000 placement combinations including u64 overflow boundaries.
The default Rust host test command includes them on Windows. Other host targets
must not silently substitute Rust behavior for the native export.

The shared [policy entrance](../../../bridge/policy/main.omg) also selects the
[lifecycle decisions](../control/README.md). Admission, IPC custody and upstream
rendering/storage adaptation remain Rust.
