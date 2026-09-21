# Public pinned PCI interrupt-routing observations

This host-only fixture set calls the actual public rust-osdev/acpi 6.1.1
`Interpreter::new`, `load_table`, `PciRoutingTable::from_prt_path`, and `route`
APIs at `257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`. There are 101 original
synthetic AML/resource fixtures. No firmware image, external test-suite fixture,
private-body replacement, interpreter source edit or forged ObjectToken is used.

The probe follows the existing Cathedral public-interpreter trap pattern. All
physical mapping, memory, port, PCI, clock, sleep, synchronization, debug and
fatal callbacks increment a counter and panic. Only construction of one inert
mutex identity is permitted; acquiring it is forbidden. Actual SystemIo GAS
construction stores metadata without mapping or I/O. Every completed fixture
must report zero forbidden calls and exactly one constructor identity. Each runs
in a fresh process with a five-second deadline; timeout is failure, not evidence.

## Primary specification and intended strict profile

[ACPI 6.6 §6.2.14](https://uefi.org/specs/ACPI/6.6/06_Device_Configuration.html#prt-pci-routing-table)
defines four fields per mapping package: DWORD address, pin 0–3, NamePath or
integer-zero source, and DWORD source-index. The function field must be FFFF.
A link source-index identifies a resource descriptor; a zero source instead
makes it a GSI. String literals are explicitly distinguished from NamePaths.
The intended first Omega adapter uses those rules and the existing strict
resource-template validation. Its bounded output capacity of 32, finite storage,
and explicit failure outcomes are Cathedral policies, not ACPI limits.

`fixtures.json` records `strict_expectation.complete_request` and a rationale
for the complete decode/selection/resource request. These are prospective design
expectations, **not executed Omega results or claims of full ACPI conformance**.
For example, a link with an out-of-range descriptor index can decode into a
request before detached `_CRS` validation rejects it. No compatibility branch
has been implemented merely to reproduce a pinned defect.

## Observed pin behavior

| Fixture family | Public pinned observation |
| --- | --- |
| Row lengths | Direct rows shorter than four panic; three-element link rows succeed. Extra fields are ignored. |
| Address/GSI overflow | Address bits above31 are discarded; direct GSI casts to u32. |
| Function/ordering | Exact functions are supported in addition to FFFF. The first matching record wins, including wildcard/specific overlap. |
| Source fields | Only integer zero means direct GSI. NamePath and legacy String links are accepted; other sources fail. |
| Name scopes | NamePath retains its declaration scope; String searches from the `_PRT` path. Separate fixtures obtain IRQ11 versus IRQ10 from these different scopes. Ordinary object shadowing does not hide a farther namespace level. |
| Non-search link paths | Relative multi-segment and parent-prefixed links remain unresolved, then fail `_CRS` resolution. Missing absolute links decode and fail later; missing simple links fail during decode. |
| Malformed strings | An invalid short leading character can panic during name-error construction. Non-ASCII string bytes panic while loading. |
| Link SourceIndex | Its value and type are ignored, including a string and values above DWORD. |
| Resource counting | The pin accepts exactly one retained supported IRQ resource. It drops recognized unsupported descriptors, while a retained DMA plus IRQ or two IRQs fails. This differs from selecting the physical descriptor named by SourceIndex. |
| Template termination | Missing EndTag, incorrect checksum, and trailing data after EndTag are accepted in the demonstrated cases. |
| Extended IRQ | Multiple interrupt numbers are returned by the pin; the planned current-resource profile rejects this fixture. |
| Capacity | The public pin accepts 32 and 33 rows; the proposed bounded output policy rejects33. |

`observations.json` retains raw public debug/error output, normalized IRQ fields,
exact bytecode hashes, all harness and upstream Rust source hashes, compiler
version and binary hash. The bytecode itself is retained in `fixtures.json`.
Load, decode and route outcomes are counted separately; errors and panics are
not reported as successful compatibility cases. Reference-cycle construction
and concurrent mutation are not modeled by this public AML fixture set.

## Reproduction

```sh
python3 tools/ports/acpi/pci-routing/fixtures.py --check
python3 tools/ports/acpi/pci-routing/check.py
python3 tools/ports/acpi/pci-routing/verify_record.py
```

The host check builds locked/offline using `nightly-2026-09-04`, reruns every
public observation, checks concrete canaries, and rejects any output or input
hash drift. `--write` replaces the record only after reviewing intentional
changes. The read-only verifier checks the retained complete input closure,
bytecode/row identity, safety counters, stage counts and current binary hash.
There is no Omega source-check, checked-interpreter, native ABI, live interrupt
routing, `_SRS` operation, or production-integration claim in this milestone.

The fixture generator and probe are original authored harness code under the
same MIT OR Apache-2.0 project terms. The pinned crate retains its original
[license notices](../../../../licenses/rust-osdev/acpi/).
