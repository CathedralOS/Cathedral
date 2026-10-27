# Temporary native entrances

[app/main.omg](app/main.omg) selects the complete chooser app for the common
Rust Omega host. The app owns its session, state, rendering and service decisions.

[native/main.omg](native/main.omg) selects [policy/main.omg](policy/main.omg)
for existing Rust decision tests and explicitly selected Rust fault probes.
That adapter decodes the old 128-word request, calls the typed chooser policy,
and encodes its result. Nothing in the ordinary Omega app calls this adapter.

The [Rust build entry](../../../../../source-rs/distribution/applications/chooser/build.rs)
packages both exports. The existing compiled tests exercise both the adapter
and the complete app against deterministic service peers.
