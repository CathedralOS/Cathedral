# Shared OS platform

Owns contracts/, services/, drivers/ and libraries/ used across the OS independently of Cathedral desktop policy. Standard capture, input, audio and lifecycle contracts belong to this layer as they are implemented. Its [service contracts](contracts/README.md) are independent of their providers.
It targets the public kernel contract package and foundation, never kernel or
distribution implementations. Folder placement does not itself provide runtime isolation.

See [the source layout](../../wiki/architecture/repository_layout.md).
