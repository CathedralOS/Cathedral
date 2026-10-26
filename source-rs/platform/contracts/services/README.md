# Scoped Matrix connection and service acquisition

Start at [services.rs](services.rs) for the bounded acquisition protocol and
[tests.rs](tests.rs) for request ceilings and version/shape validation.
The executable-side reading path is [runtime acquisition](../../libraries/user-runtime/services/services.rs),
then [retained endpoint custody](../../libraries/user-runtime/services/offer.rs).
The stock provider authenticates requests in
[children/services.rs](../../../distribution/matrix/service/children/services.rs).

The accepted [coordination and IPC contract](../../../../wiki/spec/communication/ipc.md)
adds a supported coordination service for the shipped OS/default matrix and
protocol-controlled shared memory. This Rust broker implements interaction,
storage, explicitly offered input production and capture; coordination identifiers, matching, introductions and
shared writable mailbox custody have not been implemented here.

## Contract

A program declares required and optional imports. Its owner resolves them against
its own offered authority before launch. A declaration requests authority; it
cannot confer it. Missing/incompatible required imports refuse launch. Optional
imports may remain absent. Launch authorizes one scoped Matrix connection.
The accepted [bootstrap contract](../../../../wiki/spec/execution/components_and_matrices.md#scoped-application-bootstrap)
now requires launch-provisioned connector backing and notification rights, retrieved
through a current-process syscall, without broker arguments
in main or a reserved prefix in application argument bytes. Repeated access denotes
the same logical connection; the library coordinates its requests rather than
allocating another connection. This is a migration requirement, not implemented
by the startup-endpoint mechanism documented below.

The Matrix connection is the app-facing environment interface; broker is its
import-resolution role. The parent/provider must provision first-allocator
communication without asking the child to allocate before it has an allocator.
The [accepted connection contract](../../../../wiki/spec/communication/ipc.md#matrix-access-and-connection-setup)
requires funded, recipient-bound offers for exact backing/notification rights,
complete acquisition before usable publication, and bounded pending custody with
acceptance/cancellation races resolved. Common versioned transport is distinct
from service payload schemas. Existing wire versions below are implementation
evidence, not the final syscall/API or proof of this complete bootstrap path.

The owner selects the caller's offer from the receiving endpoint and its live
incarnation. No PID, sibling number, realm, requested scope or claimed manifest
in request bytes can select or widen that offer. The broker is local to this
matrix, not a global registry. An acquired service can point at a matrix mediator;
acquisition must never bypass the mediation or revocation its owner requires.

Acquisition returns explicit service connections. Ordinary operations use those
connections directly; there is no broker round trip for every operation.
Programs may cache their bindings. Binding availability at launch does not promise
provider health or perpetual authority: operations can still fail after revocation
or provider death. Required compatibility is checked before launch, not an excuse
to omit operation errors.

Bootstrap and individual service protocols have separate versions. Adding a
service does not require changing the app entry shape. Service policy stays in
the broker; the kernel's bootstrap lookup is not a service-name lookup.
The Rust transport below is a bounded experimental version, not a frozen ABI or a
claim that every driver and initial boot process already enters this way. Early
processes that establish brokers still require explicit kernel-supplied grants.

## Rust bootstrap version 2

`service_bootstrap!` emits the import record `CTHIMP03`, including resource
ceilings. `CTHIMP02` also selects broker startup; `CTHIMP05` adds required resource
profiles for installed matrices. All use the declarations
documented in [imports](../imports/README.md), instead of the old
three/five-endpoint bundle. The sealed image determines this choice. Supported
positional records and undeclared fixture images keep their documented startup
behavior; the retired `CTHIMP04` profile is rejected. Kernel
admission never interprets either platform record.

The current implementation's initial environment contains two endpoints: broker request sender,
then reply receiver. Together these are one logical connection. Existing kernel
owner-control endpoints and profile/launch arguments remain separate mechanisms.
`entry_with_services!` loads the environment and calls the supplied Rust main with
`Services`; the compatibility loader also accepts legacy matrix bundles. These
describe current evidence, not a second supported discovery route to preserve
after migration. Replace ordinary startup endpoint discovery and its mandatory
main wrapper with the accepted lookup path in one coherent implementation change;
explicit privileged boot grants remain separate.

An acquisition is eight little-endian u64 words:

`[ACQUIRE, bootstrap_version, contract, minimum, maximum, part, 0, 0]`

A success is `[0, service_version, 0, 0, 0, 0, 0, 0]` with one typed right:
interaction supplies a notification wait right; other services move an endpoint.
A refusal is `[status, 0, 0, 0, 0, 0, 0, 0]` without a right. Unknown/ungranted
contracts return UNAVAILABLE; incompatible versions return INCOMPATIBLE;
malformed shapes return INVALID. An already-transferred part returns TAKEN.
Neither rejection nor repeated requests allocate channels. The kernel reserves
recipient custody before committing each handoff. Bootstrap version 1 is refused:
its interaction acquisition expected a copied-channel receiver.

The current services are interaction, storage, input production and capture, all version 1. Interaction
part 0 delegates a notification wait right; its ordinary RPCs reuse the broker pair.
This hint is independent of the connection's reply notification, so processing a
reply cannot consume pending input readiness. The matrix retains signal/control
rights; the child cannot signal or close the object. Mixed event waits observe
readiness without consuming it; [Wake](../../libraries/user-runtime/services/wake.rs)
consumes the hint before polling input state. Signals coalesce, not input events.
Storage, input-production and capture parts 0 and 1 each transfer a separate request sender
and reply receiver. All service
channels and the wake notification are preallocated from the owner's budget at
admission. The notification replaces one queue's object allowance; its local
handle metadata is separately bounded. This slice delays
handoff, not allocation or provider startup. It does not implement arbitrary
service enumeration, dynamic grants or new permission prompts.

The broker retains unsent rights until acquisition or child teardown. Successful
handoff clears that exact retained right; notification delegation releases the
broker's source wait copy once receiver custody commits. Failure before publication
preserves it. Teardown closes
untransferred rights too, including when the child exits halfway through acquiring
a pair. A failed acquisition never silently mints another connection. The owner
keeps input hints in its local state until wait-right handoff, then signals the
notification. Teardown closes its object and releases the owner's signal/control
rights. Positional startup compositions retain an explicit copied-wake adapter;
they do not use this acquisition protocol.

The stock version requires interaction and separate storage streams. It preserves
the child's existing realm binding, parent mediation and provider checks. The
remaining stock configuration and recovery choices are distribution policy.
Other bootstrap shapes can be introduced without making the kernel understand
service names.

## Explicit input production

`INPUT_PRODUCER` requires both a sealed manifest declaration and an explicit owner
offer. `imports::ALL` remains the ordinary interaction/storage default;
`SUPPORTED` names the recognized mask, not a recommended grant. Legacy positional
startup cannot supply this service: a required declaration refuses launch and an
optional declaration remains absent.

The stock broker attempts a grant at first acquisition: two channel slots, one
private focus group and click-to-focus permission. This is stock distribution
policy; the platform mechanism supports separately selected quotas and rights.
Nested hosts negotiate upstream capacity before committing the grant. BUSY or
NO_MEMORY refuses acquisition without transferring an endpoint; declaring even
a required service does not guarantee runtime capacity. No grant silently evicts
existing channels to make room.

The [producer receiver](../../../distribution/matrix/service/children/producer.rs)
authenticates the installed endpoint incarnation and admits only `PRODUCER_INFO`,
`PRODUCER_POINTER`, `PRODUCER_KEY`, `PRODUCER_DESCRIBE` and `PRODUCER_RELEASE`.
Descriptions are immutable bounded hints for the caller's own channel ordinals,
published before delivery; they grant no extra input or observation authority. Interaction, storage,
registration and owner commands on that pair are denied; production on the app's
ordinary interaction pair is denied when it has this separate service. Before
both endpoints have transferred, early producer traffic is discarded without a
reply or input mutation, keeping the retained reply receiver empty for transfer.

Closing the producer connection retires its grant and closes any still-retained
partner without stopping the app. App teardown also reclaims interrupted
acquisitions. View invalidation, release and owner revoke end the grant; the current
broker does not automatically regrant or recreate an acquired pair. Cached
connections therefore do not promise continued authority after a resize/rebind.

## Client discipline and proof

Acquisition is synchronous and must be serialized with interaction RPCs because
they share a reply stream. Omega hosts and Split select shared transport through
[Services](../../libraries/user-runtime/services/services.rs); its
[transport owner](../../libraries/user-runtime/services/transport.rs) retains the
same request sequence across both operation kinds. Acquiring optional storage
later does not create a second client over that pair. Moved endpoint replies are
explicitly accepted, and interaction acquisition accepts only notification replies;
ordinary interaction replies reject unexpected rights. Remaining
copied consumers opt in separately; failure never switches transports.
The runtime caches successful bindings and absent storage/production. A complete refusal, including
capacity refusal, leaves the broker usable. It closes an ambiguously failed broker
connection rather than consuming a late endpoint reply as an interaction reply.
The cache does not revive closed/revoked handles; callers still handle operation
errors. Storage transfer has two wire steps; failure after the first closes that
received endpoint and teardown reclaims whatever remains at the broker.
Input-production transfer follows the same two-step custody rule.

`Services` is the ordinary one-call binding. A caller choosing a larger negotiated
window uses `ServiceBindings<WIDTH>` from the start; acquisition itself still runs
serially. `into_shared_transport` consumes the bindings and transfers that same
pipeline into the caller's event loop, preserving its request IDs and negotiated
window. Previously returned service rights remain caller-owned. The stock matrix
uses this with width two: acquire its wait right and storage pair, then move the
broker into its existing input/presentation dispatcher. It does not reconnect or
reset correlation after acquisition. Installed stock matrices declare broker
startup; positional bundles remain only in explicitly wired fixture compositions.

[The independent matrix proof](../../../distribution/lab/matrix-composition/README.md)
exercises acquisition, refusal, repeat requests, replacement and interrupted
handoffs. Its storage endpoints test custody only, not a filesystem. Notes and
Colors use the actual broker and scoped persistent storage in F5.
`desktop_test.py --service-imports undeclared` and `--service-imports incompatible`
use disposable Notes manifest variants: Notes remains editable without storage,
dirty-close protection works, and Colors retains independent persistence across
cold reboot. No saved interactive media or compiled source image is modified.

`desktop_test.py --input-producer complete`, `partial` and `denied` use disposable
Notes and stock-policy feature builds. They exercise isolated operations and
closure, interrupted acquisition with concurrent capacity refusal, or an absent
owner offer. Normal editing, repeated close/relaunch and cold persistence continue
through the same desktop workflow. These probes do not add a visible synthetic
cursor or grant production to any default installed app.

## Explicit capture

`CAPTURE` is separately declared and offered; `ALL` does not include it. Its two
parts transfer a request sender and reply receiver. This grants access to the
owner-selected capture scope, not a way to name arbitrary processes or images.
The [capture wire](../capture.rs) defines INFO, BEGIN, TAKE and CANCEL. A successful
TAKE transfers only sealed, already-masked backing. Pending/expired/refused replies
have no attachment. A successful transfer consumes the result; queue pressure
retains the same funded image. The source, crop and masks are provider policy and
cannot be replaced through request fields.

The installed [Split provider](../../../distribution/applications/split/session/capture/capture.rs)
selects one Notes leaf and fixed mask. Its [viewer](../../../distribution/applications/capture-demo/main.rs)
uses [runtime capture](../../libraries/user-runtime/capture.rs) after acquisition.
This is a bounded experimental service; recursive source readiness, app-published
mask metadata, remote capture and GPU completion are not implemented here.
