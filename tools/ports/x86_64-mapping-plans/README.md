# Detached mapper decision witnesses

See the [port record](../../../source/libraries/x86_64/mapping-plans.PORT.md).
`check.py --omega /path/to/omega` audits the pin/mapping and deterministic corpus,
executes112 actual pinned mapper scenarios, then evaluates the matching Omega
decision bodies, extra malformed-input checks and four body mutations.
`--host-only` omits Omega execution.

The Rust template owns initialized page tables through a checked ID registry;
UnsafeCell supports actual mapper mutation without casting a shared reference
into mutable storage. No table is installed and no flush instruction executes.
The generated expected cases distinguish failure outcomes from preceding writes.

`main.omg` retains the complete scenario source but does not execute an aggregate
constant; the runner selects and evaluates groups. The source inventory leaves
full mapper orchestration pending rather than marking a tested leaf as the whole
upstream method. No native Omega ABI or live mapping claim is made.
