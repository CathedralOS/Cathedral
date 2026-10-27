# Native workspace codec

Start at [main.omg](main.omg): route catalogue operations to
[catalogue.omg](catalogue.omg) and switch operations to
[selection.omg](selection.omg). These files translate the Rust host's opaque
words, query tags and command numbers. The actual policies live under
[service/children/workspace](../../../service/children/workspace/README.md).

The catalogue codec preserves the monotonic ticket counter, selected position
and three app records. The switch codec preserves the operation ticket, first
refusal and ready/waiting command. A fatal failure cannot complete into a
successful finish. Neither codec owns tasks, grants, scene resources or the
host's authenticated connection checks.

The unchanged Rust catalogue and switch adapters exercise these codecs in the
compiled matrix tests and ordinary desktop. See the service README for the
pending-switch and late-commit guest fixtures.
