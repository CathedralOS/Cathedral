# Shared OS platform

Owns services/, drivers/ and libraries/ used across the OS independently of Cathedral desktop policy. Standard capture, input, audio and lifecycle contracts belong to this layer as they are implemented. It targets shared contracts and foundation, never kernel or distribution implementations. Folder placement does not itself provide runtime isolation.

See [the source layout](../../wiki/architecture/repository_layout.md).
