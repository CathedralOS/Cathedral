# Replaceable background

Start at [app/main.omg](app/main.omg): acquire presentation, verify the refused
storage request, paint a changed grant, then wait for a wake or lifecycle request.
[model.omg](app/model.omg) names the session, completed operations and requested
work. The app reuses one surface and paints the current matrix-local dimensions;
it owns no input keys, storage or descendants.

The matrix resolves `Background` in its own realm, so replacing that executable
changes the backdrop without changing the kernel, compositor or chrome. The
current app is a solid fill. Wallpaper, animation and role-selection UI remain
distribution choices; retry and fallback belong to the owning matrix.

The standalone replacement is being built at [process/main.omg](process/main.omg).
It establishes owner control before the broker, drives this same app model,
and waits on owner requests plus presentation notifications.
[exchange.omg](process/exchange.omg) keeps answering owner control while a broker
request is pending, without replaying that request or advancing the app model.
`presentation.omg`
encodes actual service packets; there is no Rust host packet between it and the
runtime. [native/main.omg](native/main.omg) supplies component packaging and
`native/process.json` declares the existing interaction-only import and channel
request. This path passes the candidate ordinary desktop, three-level fresh/cold
nesting and startup-failure recovery. An intermittent nested close timeout also
reproduces in the Rust baseline. It is not installed by F5 yet: see
[the current tasks](../../../../TASKS_RUST.md). The paths below still supply F5.

[bridge/component/main.omg](bridge/component/main.omg) adapts this session to
the temporary [Omega app host](../../../../source-rs/platform/libraries/omega-app/README.md).
Its codec owns packet offsets, protocol values and strict HEALTH/LOSS decoding.
The diagnostic encoder preserves the ordinary desktop's storage-refusal witness.
[bridge/native/main.omg](bridge/native/main.omg) selects the independently built
component. Neither package contains a second copy of the app's behavior.

The installed [Rust entry](../../../../source-rs/distribution/applications/background/main.rs)
declares interaction only and runs that export through the common host. Rust
retains executable startup, scoped IPC and buffer custody, plus the existing
startup-failure injection. The stock matrix funds five transport pages and nine
temporary host pages; this does not add storage or child-launch authority.

The [compiled tests](../../../../source-rs/distribution/applications/background/tests/behavior.rs)
drive the same export with controlled replies. They check scope refusal, retained
view data across IPC, surface reuse, strict lifecycle replies and failure without
replay. The ordinary F5 desktop test checks the real process and its visible fill;
nested desktop and startup-failure tests exercise recursive placement and recovery.

[The independent control test](../../../../tools/omega-user/background_test.py)
holds the real process's broker reply while issuing HEALTH, LOSS and malformed
owner requests. It checks startup and steady-state progress, owner disconnect
isolation, and channel, page and notification refunds after repeated teardown.
