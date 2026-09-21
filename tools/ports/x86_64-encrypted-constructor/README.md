# Explicit-profile recursive constructor observations

`check_interpreted.py` runs the complete610 reference rows and10 changed-frame
comparison controls using a bounded row loop. `check_const.py` checks the disabled
and repeated-profile loop bodies as constants, each with its control. The full
expanded const route remains available via `check.py`; `--start/--end` select
profiles, and `--host-only` checks fresh Rust observations and fixture generation.
`generate_compact.py --check`, `map_inventory.py --check` and `verify_record.py`
check fixture freshness, pinned source coverage and retained verification hashes.

The Rust generator extracts the ordered constructor checks with explicit pointer,
CR3 input, selected-entry and final-object substitutions. It calls real physical
address/frame/PTE APIs in ten isolated feature configurations, without creating a
RecursivePageTable or executing a register instruction. See the component PORT.

The compact harness changes only the computed frame comparison in control entry
bodies; positive entries return0 and controls return1. All61 rows of each profile
are still evaluated. Source/tree preparation and checked interpretation are
separate from native Omega execution or a universal proof.
