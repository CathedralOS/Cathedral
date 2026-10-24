# Service acquisition runtime

Start at [services.rs](services.rs): load startup bindings, request an interaction
or explicitly offered storage/producer/capture connection, cache the result, and
preserve transport failure state.
[offer.rs](offer.rs) is the provider half: select an endpoint from the approved
ceiling, transfer it once, and retain failures for later teardown. The
[platform contract](../../../contracts/services/README.md) defines the wire and
its limitations.

[Shared admission](../matrix/admission/admission.rs) allocates the channels and
creates the offer. The stock [launch coordinator](../../../../distribution/matrix/service/children/admission.rs)
resolves the sealed import declaration before choosing broker startup; its
[dispatcher](../../../../distribution/matrix/service/children/services.rs)
authenticates the installed client incarnation. The independent matrix exercises
the same mechanism without stock imports.

Apps enter via `entry_with_services!`. Notes requests interaction and optional
storage; Colors requires both. Both acquire once before their ordinary loops.
The runtime does not perform global discovery, request additional authority, choose
realm scopes, or own service recovery. Handles are explicit lifetime-bound local
names; returned copies are not independent grants or automatic reconnections.
