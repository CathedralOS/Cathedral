# Pinned UEFI raw contract corpus

This isolated package translates `uefi-raw` from
[`uefi-rs`](https://github.com/rust-osdev/uefi-rs) revision
`c0facddf9ba42b74906a37fca2869e6cdbc8da6a` under **MIT OR Apache-2.0**.
See the repository [notices](../../../../THIRD_PARTY_NOTICES.md) and each slice's
source inventory, vectors, verification commands, and deviations.

| Slice | Status and evidence |
| --- | --- |
| Scalars and pure scalar helpers | [Tested by Omega semantic evaluation](scalars.PORT.md) |
| Complete boot/runtime/system table declarations | [Typechecked; semantic fixture tested; full plan reflection blocked](tables.PORT.md) |
| Console, image loading, and device paths | [Semantic helpers tested; fixed policies typechecked; dynamic tails blocked](console.PORT.md) |
| Display, PCI/USB, and machine protocols | [Typechecked; 325 upstream representation vectors verified](machine.PORT.md) |
| Storage and firmware volume/management | [Typechecked; semantic constants tested; 681 representation vectors verified](storage.PORT.md) |
| Shell protocol remainder | [Raw declarations typechecked; complete 46-field plan reflection blocked](shell.PORT.md) |
| HII protocols, IFR, and helpers | [Semantic helpers tested; fixed carriers checked; typed unions and runtime tails blocked](hii.PORT.md) |
| Network representations and helpers | [Semantic helpers tested; fixed plans checked; tail/projection/capacity gaps recorded](network.PORT.md) |
| TCG v1/v2 measured-boot representations | [Typechecked; upstream ABI vectors verified](tcg.PORT.md) |

Stages apply to individual slices, not the entire upstream crate. The package
is not connected to a production build root. Existing contracts in the parent
directory retain their separate admission and lifecycle policy; see their
[reconciliation](../RECONCILIATION.md).

Raw address carriers describe foreign data. They do not authorize dereference,
firmware calls, callback registration, or device access. Explicit layout plans
and upstream representation vectors do not establish emitted Omega ABI
agreement. Each slice records the evidence still needed before integration.
