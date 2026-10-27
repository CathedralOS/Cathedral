# Stock matrix resource offers

Start at [main.omg](main.omg): fit a program declaration to its owner's offer.
Named capacities, optional declarations and admission outcomes expose the policy;
the native packet never enters these operations. These policies run
in ordinary F5 matrices, including nested copies. They are distribution defaults,
not a platform-wide resource policy and not authority to exceed kernel grants.

[recipes.omg](recipes.omg) owns the stock launcher, picker, background, workspace
app and selected-user offers. Configuration supplies launcher/app page ceilings
and app surfaces. The picker needs enough backing for a host and reserves its own
six surfaces before offering the rest to the selected user. The background is
offered interaction only; it receives no storage offer. CPU zero retains the
existing inherited-ceiling meaning.

[main.omg](main.omg) decides what a realm program may request:

- An explicit profile must fit private pages, reader pages and any declared
  channel/descendant requirements. Excess refuses instead of silently reducing
  an explicit requirement. Its surface request remains distinct from admission.
- An unprofiled desktop app gets bounded defaults, with a larger page offer when
  it declares descendants. The owner's offer still caps every value. Other roles
  keep their offered pages; absent resource declarations keep offered channels
  and descendants.
- [Surface admission](surfaces.omg) refuses requests above the standing ceiling. A descendant
  host may prefer spare surfaces, bounded by backing left after chrome and
  background; unprofiled hosts additionally cap that preference at sixteen.
  A leaf requests only its minimum. A preference below the minimum is an empty
  allocation range, not permission to exceed backing.

The Rust [resource adapter](../../../../../../../source-rs/distribution/matrix/policy/resources.rs)
carries plain facts and results through the [private native codec](../../../../bridge/policy/resources.omg).
That codec alone owns operation numbers, packet slots and native error encoding.
[Realm setup](../../startup/realm/README.md) applies role offers;
[children/admission.rs](../../../../../../../source-rs/distribution/matrix/service/children/admission.rs)
supplies decoded profile facts. It still consumes pending one-shot funding,
accounts for actual transport overhead, validates arithmetic, tries disjoint
surface ranges from preferred to minimum, reserves kernel resources and rolls
back failed admission. Failed Omega choices return before those later operations.
Exact images, slots, process reservations and capabilities remain host-owned.

Legacy catalogue launch and mechanism-specific image/transport funding remain
Rust work. The native packet is temporary implementation plumbing, not an app
manifest or public IPC
contract. This does not finish the matrix's Omega port.

[Service matching](../imports.omg) and [live-layout planning](../../configuration/main.omg)
now run in Omega. The bounded image scanner and declaration decoder also run in
Omega; mapped-byte access and actual application/publication remain host work.

[Compiled tests](../../../../../../../source-rs/distribution/matrix/tests/resources.rs)
cover distinct role authority, picker capacity, strict profile refusals,
unprofiled defaults, owner ceilings and surface arithmetic boundaries. Exact host
and guest verification is recorded in `TASKS_RUST.md`.
