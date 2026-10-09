# Experimental user runtime

Owns x86-64 task entry, syscall wrappers and the static ELF linker layout for the
Rust lab. Depends only on shared `contracts/`; never imports kernel or
distribution implementation. Applications choose their behavior and return an
exit status through `entry!`. The `ipc::Handle` wrappers expose only boot-issued
grants and bounded send/receive/revoke calls; they do not perform service discovery
or choose endpoint policy. The runtime has no heap allocator or unwinding.

The calling convention and fixed image layout remain experiments, not frozen
Cathedral interfaces. See the [lab guide](../../../README.md) for build commands,
loader restrictions and the remaining admission-policy boundary.
