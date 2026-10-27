# Stock matrix configuration

Start at [main.omg](main.omg) for live layout: check authority, reject a busy
owner, validate the request, then plan placements by role. Its inputs and results
are named values. [wire.omg](wire.omg) decodes the actual eight-word IPC request;
[placement.omg](placement.omg) checks the bottom bar and workspace geometry with
bounded arithmetic. Chrome takes the bottom bar, app positions take the workspace,
and background takes the full extent. Only standing app placements are remembered.

The [host](../../../../../../source-rs/distribution/matrix/service/children/configuration/layout.rs)
rechecks the exact controller/incarnation/epoch in `World::layout`, validates all
rectangles before mutation, preserves revoked views and key masks, then remembers
placements, flushes and replays retained scenes. These resource operations and
failure propagation stay in the host; the Omega plan grants no authority and does
not make publication atomic across IPC hops.

Saved settings enter through [encoding.omg](encoding.omg), with canonical names
in [name.omg](name.omg). The chooser calls this codec directly. CTHMAT02 is the
stock distribution's file format, not a universal platform configuration or a
source of authority. Rust's public representation remains in distribution
contracts for installer/tools and as an independent compatibility oracle.

The [native adapter](../../../bridge/policy/configuration/main.omg) translates the
Rust host's temporary packets. Native operation numbers, word offsets and numeric
status codes belong there; they do not shape the service's domain API.

Compiled conformance tests compare every single-byte mutation, resource and
legacy bounds, UTF-8 names, and 100,000 placement combinations plus u64 overflow
boundaries. Live-layout tests cover refusal precedence and role placements. Guest
`settings_test.py` checks saved/live application, retained drafts, later launches
and cold persistence. The default Rust host tests include the compiled policy on
Windows; other hosts must not silently substitute Rust for the native export.
