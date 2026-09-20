# x86 page-table policy field canary

The canonical PTE schema remains in `source/drivers/facts/x86_page_table_entry.omg`.
Its stateless policy now lives in `x86_page_table_layout.omg`. This canary demands
all fourteen field projections on a local equivalent schema through that actual
policy. `check-schema.py` binds the test schema to the canonical one and audits
all64 requested bit positions, eight-byte size and eight-byte alignment.

```sh
OMEGA_BIN=/path/to/omega tools/x86-page-table-layout-canary/run.sh
```

The harness uses current application/package wiring and source checking. Fresh
Omega eaa7993 passes13sources. The old jq assertion is retained as historical
material; current Omega no longer emits the JSON artifacts it expected.
The layout no longer returns a non-copy receiver-owned buffer: it constructs a
local buffer under an explicit Layout witness. No runtime storage is granted.

A local schema is needed to distinguish policy normalization from the current
imported plan-laid private-field visibility limitation. Native emitted geometry
is not measured, and no table installation, frame ownership or TLB operation is
claimed. The separate numeric codec tests exercise the actual canonical type.
