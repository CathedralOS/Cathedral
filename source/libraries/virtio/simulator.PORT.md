# Pure queue simulator — VIRTIO-004

`simulator.omg` is Cathedral-authored model code built on the pinned translated
feature, split-ring and packed-ring algorithms documented in `PORT.md`. It is a
unit-test target for later driver work, not a DMA or concurrency implementation.
The public request/response values carry no hardware authority.

The model has an explicit capacity of eight descriptor/buffer slots. Split
sizes are 1, 2, 4 or 8; packed sizes are 1 through 8, including non-powers of two.
This finite harness limit does not narrow the general queue arithmetic helpers,
which support the protocol's full sizes. Buffer IDs are 0–7; split IDs must also
fit the configured descriptor count, while packed IDs remain independent of
ring position. Storage projections use explicit bounded dispatch. This keeps
the harness storage separate from the general wrap/event algorithms it tests.

`DeviceSession` models reset, acknowledge, driver, features accepted and ready
states. Calls in the wrong order fail without advancing. Negotiation retains
128-bit masks and delegates subset/dependency checking to `features.omg`.
Rejected negotiation records FAILED; reset clears acceptance while preserving
offered features. Queue creation requires ready state and VERSION_1, selects
the packed format from the accepted feature mask, and initializes wrap bits to
one. This is a sequential model of supplied observations; it cannot guarantee
that a real device accepts a status write.

Each submit adds one direct buffer, reserving its ID until reaping. Full queues
and duplicate IDs fail without mutation. The split model writes the descriptor
by ID and records the ID in the available ring. The packed model writes the
next ring position and the correct availability/wrap flags. Submission invokes
the existing split or packed event helper and returns a notification decision.
An invalid packed suppression descriptor fails before changing queue state.

`queue_process` models one in-order device completion. It checks availability,
reserved ID bookkeeping, buffer length, and writable status; writes to a
read-only buffer and excessive written lengths are rejected. It writes a used
entry and, for packed queues, used/wrap flags. `queue_reap` validates the used
observation, returns the completion and releases the ID. Failed operations
return the original queue value. Reset clears the model and makes further
queue operations fail until a new queue is created.

`queue_well_formed` checks capacity/format admission, cursor bounds, accepted
features, queued/completed/inflight counts, modular sequence distances, and
busy-ID count. It is an internal consistency check for model states, not a
validator for arbitrary DMA memory or a proof that every array cell represents
a possible physical device history. States should originate from the model's
constructor and transitions; the test fixture also injects selected invalid
states to verify rejection. The three u16 sequences wrap explicitly, while
packed positions use the actual queue size and separate wrap counters.

The simulator intentionally models single direct buffers and deterministic
in-order processing. It does not model indirect chains, multi-buffer lists,
packed in-order batch compression, out-of-order asynchronous completion,
payload contents, physical addresses, cache coherence, barriers, interrupts,
transport handshakes or allocation. Those features are not claimed implemented
by this finite unit-test target. General raw indirect descriptors, transport
facts and access plans remain available in their respective port modules.

```sh
python3 tools/ports/virtio/simulator-semantic-check.py --omega /path/to/omega
```

The fixture has six groups: status/feature negotiation; split lifecycle and
capacity; packed three-slot lifecycle and reuse in the opposite wrap epoch;
u16 sequence rollover; negotiated notification suppression; and invalid input,
bookkeeping and reset. Six independent negative controls change expected
behavior inside those bodies rather than modifying the final assertion.
Fresh Omega `eaa7993a23623cd8fabf45350340479c5c9c7879` passes all six groups;
all six body-mutating controls compute failure and are rejected. The final
source check also passes after the explicit bounded storage projections.
Execution is Omega semantic evaluation through a success contract. No native
Omega executable, DMA, live device, timing or concurrency result is claimed.
