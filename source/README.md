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

## Language compatibility

Follow Omega's current [machine contract](https://github.com/CathedralOS/Omega/blob/main/wiki/spec/language/machines.md)
and [nested-machine guide](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_4_nested_machines_transitions.md)
when adding or changing source. Compiler acceptance alone is insufficient: the
compiler temporarily accepts forms that the language specification has retired.

Use nested `machine` declarations for local control flow, with explicit runtime
arguments and their own checked contracts. Private requirements may be inferred
when every incoming call establishes them; callers must retain justified facts,
result typing and recursive reasoning across the unified machine path. Missing
compiler support is not a reason to restate those contracts throughout Cathedral.
Nesting captures no runtime data. Keep loops as terminal transitions so the
compiler must preserve their constant-stack behavior. Names such as `State` or a
`state.omg` data module are ordinary names and need no cosmetic renaming.

The existing tree still contains migration debt: `state` declarations, older
`pub` declarations, data-owned machine namespaces and standalone boundary
requirements. Most of the old library source is outside the normal desktop's
compiled closure. Review each form against its owning upstream contract; do not
bulk-rewrite provider identities or weaken resource contracts to obtain a build.
Qualify a changed package through its actual consumers. Record the exact compiler
and runnable evidence in [the handoff](../TASKS_RUST.md), and qualify the full F5
composition before replacing its pinned compiler. A successful app migration does
not establish compatibility of the separate native kernel or every library port.
