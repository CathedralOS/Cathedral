# Native launch codec

Start at [main.omg](main.omg). It routes the Rust host's private operations to
[the launch workflow](../../../service/children/launch/main.omg), role selection,
[admission encoding](admission.omg) and [import encoding](imports.omg).
It translates opaque workflow storage, command numbers and role flags; the
service policy uses named state and outcomes.

[declaration.omg](declaration.omg) unpacks native words for the platform record
decoder and publishes only a fully validated declaration. [image.omg](image.omg)
encodes the common scanner's state and bounded read requests. Both call
[platform contracts](../../../../../platform/contracts/imports/README.md)
directly. The bridge imports that package explicitly; protocols never import the
stock matrix or this bridge.

The service owns target/refusal order and stock service offers. Rust owns exact
caller, task and grant identity, the sealed mapping, requested byte spans,
pending replies and resource allocation. This packet is temporary same-process
plumbing for that host, not a public service protocol or an authorization boundary.

The compiled matrix tests exercise this route through the unchanged Rust adapters.
See [launch verification](../../../service/children/launch/README.md#verification)
for the ordinary desktop, cancellation and rollback fixtures.
