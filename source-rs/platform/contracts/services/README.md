# Scoped application bootstrap

Start at [services.rs](services.rs) for the bounded acquisition protocol and
[tests.rs](tests.rs) for request ceilings and version/shape validation.
The executable-side reading path is [runtime acquisition](../../libraries/user-runtime/services/services.rs),
then [retained endpoint custody](../../libraries/user-runtime/services/offer.rs).
The stock provider authenticates requests in
[children/services.rs](../../../distribution/matrix/service/children/services.rs).

The accepted [coordination and IPC contract](../../../../wiki/spec/communication/ipc.md)
adds a supported coordination service for the shipped OS/default matrix and
protocol-controlled shared memory. This Rust broker implements interaction,
storage and explicitly offered input production; coordination identifiers, matching, introductions and
shared writable mailbox custody have not been implemented here.

## Contract

A program declares required and optional imports. Its owner resolves them against
its own offered authority before launch. A declaration requests authority; it
cannot confer it. Missing/incompatible required imports refuse launch. Optional
imports may remain absent. The launch installs one scoped broker connection.
Application entry receives a service-acquisition interface, not a growing
positional list of service arguments.

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
service does not require changing the app entry shape. This is the adopted model;
the Rust transport below is a bounded experimental version, not a frozen ABI or a
claim that every driver and initial boot process already enters this way. Early
processes that establish brokers still require explicit kernel-supplied grants.

## Rust version 1

`service_bootstrap!` emits the import record `CTHIMP03`, including resource
ceilings. `CTHIMP02` also selects broker startup; both use the declarations
documented in [imports](../imports/README.md), instead of the old
three/five-endpoint bundle. The sealed image determines this choice. Old records
and undeclared legacy images keep their documented startup behavior. Kernel
admission never interprets either platform record.

The initial environment contains exactly two endpoints: broker request sender,
then reply receiver. Together these are one logical connection. Existing kernel
owner-control endpoints and profile/launch arguments remain separate mechanisms.
`entry_with_services!` loads the environment and calls the supplied Rust main with
`Services`; the compatibility loader also accepts legacy matrix bundles.

An acquisition is eight little-endian u64 words:

`[ACQUIRE, bootstrap_version, contract, minimum, maximum, part, 0, 0]`

A success is `[0, service_version, 0, 0, 0, 0, 0, 0]` with one transferred endpoint.
A refusal is `[status, 0, 0, 0, 0, 0, 0, 0]` without an endpoint. Unknown/ungranted
contracts return UNAVAILABLE; incompatible versions return INCOMPATIBLE;
malformed shapes return INVALID. An already-transferred part returns TAKEN.
Neither rejection nor repeated requests allocate channels. The kernel reserves
recipient custody before committing each endpoint transfer.

The current services are interaction, storage and input production, all version 1. Interaction
part 0 transfers its wake receiver; its ordinary RPCs reuse the broker pair.
Storage and input-production parts 0 and 1 each transfer a separate request sender
and reply receiver. All service
channels are preallocated from the owner's budget at admission. This slice delays
handoff, not allocation or provider startup. It does not implement arbitrary
service enumeration, dynamic grants or new permission prompts.

The broker retains unsent rights until acquisition or child teardown. Successful
transfer clears that exact retained right; failure preserves it. Teardown closes
untransferred rights too, including when the child exits halfway through acquiring
a pair. A failed acquisition never silently mints another connection. Wake state
stays pending until its receiver transfers: the kernel refuses to transfer a
receive endpoint with queued data.

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
`PRODUCER_POINTER`, `PRODUCER_KEY` and `PRODUCER_RELEASE`. Interaction, storage,
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
they share a reply stream. Current apps acquire during startup. The runtime caches
successful bindings and absent storage/production. A complete refusal, including
capacity refusal, leaves the broker usable. It closes an ambiguously failed broker
connection rather than consuming a late endpoint reply as an interaction reply.
The cache does not revive closed/revoked handles; callers still handle operation
errors. Storage transfer has two wire steps; failure after the first closes that
received endpoint and teardown reclaims whatever remains at the broker.
Input-production transfer follows the same two-step custody rule.

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
