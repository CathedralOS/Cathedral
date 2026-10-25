# Shared OS platform

Owns services/, drivers/ and libraries/ used across the OS independently of Cathedral desktop policy. Standard capture, input, audio and lifecycle contracts belong to this layer as they are implemented. It targets shared contracts and foundation, never kernel or distribution implementations. Folder placement does not itself provide runtime isolation.

See [the source layout](../../wiki/architecture/repository_layout.md).

Update/install and loading services coordinate through explicit kernel authority;
component owners decide when and how their running instances adopt replacements.
Storage services own transaction/recovery orchestration, and authorized reporting
services assemble attestation from authentic observations. This does not make those
services untrusted by placement or invent cross-service atomicity. See
[installation and activation](../../wiki/spec/execution/updates.md).
