# Independent Omega matrix

Start at [process/main.omg](process/main.omg): a standalone Omega process that
admits three peers from lent launches, gives each a third of its granted area,
offers them services and routes attenuated input. One turn serves the owner,
each slot and the relay probe, then parks on every live interest. No stock
desktop, launcher, configuration object, taskbar or filesystem is involved; the
program depends only on the [process runtime](../../../platform/libraries/user-runtime/README.md),
the [World](../../../platform/libraries/matrix/world/README.md) and the
reference protocol contracts. The [native consumer](native/main.omg) links it
the same way as the installed apps.

Descend by responsibility:

- [startup.omg](process/startup.omg): accept owner control first, learn the
  granted area (a fixed 300 by 600 region, or a real parent's poll), admit the
  three peers from launches zero to two and place them. Slot two alone is offered
  storage. Mode zero starts a slot's ordinary peer; modes one and two start its
  interrupted-handoff variants.
- [commands.omg](process/commands.omg): owner words. A nonzero high word relays
  the packet through that peer's control link; STOP and SPAWN change slots; source
  words inject input; everything else is a World owner command. A SPAWN
  replacement is not placed, so a new occupant never inherits the old view.
- [slots.omg](process/slots.omg): one slot per turn. Acquisition goes to the
  runtime's offer; other 64-byte requests are World requests under the listener's
  incarnation; shorter requests are denied. A retired listener reclaims the slot
  and drops its view.
- [probe.omg](process/probe.omg): the proof-only relay. One probe at a time; an
  expired probe or one to an empty slot ends the program. It is not a runtime or
  application API.
- [rollback.omg](process/rollback.omg): sixteen admissions that must fail the
  same way while channel, page and notification budgets stay exactly as they
  were. Launch zero is busy after the first peer; once all three listeners hold
  their bells, two channels remain and the bundle's bell cannot be created.
- [upstream.omg](process/upstream.omg): with a supplied parent, one synchronous
  poll before each local decision, adopted through the World's upstream
  inheritance. The top-level proof has no parent.

Run the live IPC proof against the Omega matrix and against the original Rust
program, which remains the comparison path:

```powershell
python tools/boot-harness-rs/matrix_composition_test.py --omega
python tools/boot-harness-rs/matrix_composition_test.py --upstream
python tools/boot-harness-rs/matrix_composition_test.py
```

The Omega profiles select this native package directly and build the Rust
fixtures with their `omega-matrix` feature, which speaks the shared v4 protocol
the runtime's server and client use; the Rust comparison matrix keeps copied
requests. The matrix is funded with seventeen channel reservations at the top
level and eighteen under a parent: three children's channels and bells, their
listeners' bells, the owner bell, the parent client's bell, and two spare for
the exhaustion probe. The [Rust fixtures](../../../../source-rs/distribution/lab/matrix-composition/README.md)
describe what the guest proof checks: service transfers, forged and forbidden
requests, sibling isolation, stale views, repeated replacement, interrupted
transfers, failed-admission rollback and whole-subtree reclamation; under a
parent, coordinate translation, queued key edges, withheld keys, shrink, revoke
and regrant, and old-epoch rejection.

This proves a matrix program in Omega over the common runtime. Its storage grant
exercises endpoint custody only; it does not serve a filesystem or distinguish
persistent realms. It has a virtual input space, no framebuffer and no
rendering. The upstream refresh is synchronous on local requests, so a slow
parent delays local dispatch; the stock desktop's asynchronous parent handling
must be preserved when that program moves. The stock F5 matrix still runs its
Rust host.
