# Experimental user runtime

Owns x86-64 task entry, syscall wrappers and the static ELF linker layout for the
Rust lab. Depends only on shared `contracts/`; never imports kernel or
distribution implementation. Applications choose their behavior and return an
exit status through `entry!`. The `ipc::Handle` wrappers expose only boot-issued
grants and bounded send/receive/revoke calls, including deadline receives; they do not perform service discovery
or choose endpoint policy. The `task` wrappers expose indexed, bounded launch grants,
child outcome collection, deadline waits, cancellation and explicit connection
acceptance. `time` reads the boot-granted monotonic clock; it grants no authority
and chooses no deadline policy. Restart decisions
belong to the caller, not this library. The runtime has no heap allocator or unwinding.
`display::mapping` retrieves only the caller's boot-installed framebuffer geometry;
pixel access and drawing requests belong to the separate platform display service.
`keyboard` exposes only the granted raw PS/2 byte channel and bounded writes;
controller configuration and physical-key decoding belong to the input service.
Timed IPC/raw reads require explicit clock authority. Timeout cancels the local
wait; callers choose how to handle remote requests and late replies.

The calling convention and fixed image layout remain experiments, not frozen
Cathedral interfaces. See the [lab guide](../../../README.md) for build commands,
loader restrictions and the remaining admission-policy boundary.
