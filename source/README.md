# Cathedral in Omega

The three owners match the [Rust reference](../source-rs/README.md):

- [Kernel](kernel/README.md): boot, machine mechanisms and enforced isolation.
- [Platform](platform/README.md): common protocols, drivers and reusable libraries.
- [Distribution](distribution/README.md): the replaceable stock matrix and its apps.

This is an incomplete implementation. The normal F5 desktop boots the Rust kernel
and hosts several compiled Omega components. The separate Omega kernel entry is
an early boot experiment; it is not the desktop's kernel.

Follow [TASKS_RUST.md](../TASKS_RUST.md) for the active migration and its verification.
The [port queue](../TASKS_OMEGA_PORTS.md) separately tracks translated hardware
libraries. `contracts/uefi/` is an older firmware boundary awaiting consolidation
with the UEFI library; it is not a fourth owner for new OS features.

Keep the Rust responsibility tree when moving behavior here. Package boundaries
are compilation choices, not a reason to put every extracted policy at the root.
Temporary native-call arrays belong in adapters; domain code should name its
state, inputs and outcomes. See the [discoverability guide](../wiki/architecture/discoverability.md).
