# Stock matrix process

Start at [main.omg](main.omg): the stock matrix as a standalone Omega process on
the [process runtime](../../../platform/libraries/user-runtime/README.md) and the
[World](../../../platform/libraries/matrix/world/README.md). It is replacing the
[Rust host](../../../../source-rs/distribution/matrix/CHARTER.md) one responsibility
at a time; F5 keeps the Rust host until this program reaches parity, then the
desktop profile swaps and the [bridge](../bridge/policy/main.omg) goes. The
[native consumer](../native/main.omg) links it the same way as the installed apps.

Today it is the display root: the desktop supervisor admits it exactly as it
admits the Rust host, and it serves three connections.

- [startup.omg](process/startup.omg): read the environment. Four endpoints make
  the display root; owner control opens first, because the supervisor connects
  it right after the spawn, then the display connection, then the source
  listener. Other environment shapes are refused until their slices arrive.
- [commands.omg](commands.omg): owner words over the kernel-installed control
  pair. HEALTH, BIND (the display extent and keys become the world's grant, with
  the compositor's whole tile allowance and the source streams bound), the
  LAUNCH_IMAGES, LAUNCH_POLICY and LAUNCH_PLACE policy for a slot, and the
  World's placement words. Everything else is denied, never silently accepted.
- [source.omg](source.omg): the installed source endpoint. POINTER_SOURCE moves
  the compositor's physical cursor, then routes through the World on the pointer
  stream; KEY_SOURCE routes on the keyboard stream, and RESET cancels both. The
  driver checks that owner words are refused here before it trusts the link.
- [scene.omg](scene.omg): the shared client to the compositor, speaking its
  six-word frames through the runtime's sized requests.
- [report.omg](report.omg): the startup line the boot harness keys on.

Prove it with `python tools/boot-harness-rs/omega_desktop_test.py`: the stock
composition with this program in the root matrix's launch and the supervisor
built with `omega-root`, which admits the root as F5 does and then parks instead
of starting storage. The proof reads the supervisor's and the drivers' startup
lines, moves the pointer over QMP and watches the compositor's cursor follow it
through this matrix.

Not yet here: children and their admission from the realm, surfaces and the
begin/commit presentation, storage binding and forwarding, management and
launch observation words, nested operation under a parent matrix, and the
stock policy packages under [service/](../service/README.md), which still
execute inside the Rust host through the bridge.
