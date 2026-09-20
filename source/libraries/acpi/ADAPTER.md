# Later Cathedral ACPI adapter contract (ACPI-007)

**Status: specification only.** This defines the later adapter's obligations; it installs no provider, issues no grant, maps no memory and performs no hardware operation. The existing [fixed-table](fixed.PORT.md) and [topology](topology.PORT.md) libraries continue to return ordinary copyable data. The names of proposed outcomes below are review vocabulary, not implemented Omega declarations or authority constructors.

## Governing contracts and current implementation boundary

This adapter follows Cathedral's [capability model](../../../wiki/design/part_1_authority/00_capability_model.md), [capability lifecycle](../../../wiki/design/part_1_authority/01_capability_lifecycle.md), [hardware foundation profile](../../../wiki/design/part_0_foundations/03_hardware_foundation_profile.md) and [driver model](../../../wiki/design/part_5_lifecycle/02_driver_model.md). It uses Omega's existing contracts for [authority establishment](../../../../Omega/wiki/spec/resources/authority.md), [extent conservation and mapping](../../../../Omega/wiki/spec/resources/extents.md), [placed access](../../../../Omega/wiki/spec/resources/placed_access.md), [device custody](../../../../Omega/wiki/spec/resources/device_access.md) and [carry](../../../../Omega/wiki/spec/resources/carry.md), reviewed at Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`.

The [current root adapter](../../core/extent.omg) satisfies the owner-authored `ExtentRootProvider::grant` requirement. Its ordinary direct invocation does not establish a grant. The [boot charter](../../boot/CHARTER.md) explicitly leaves occurrence-scoped custody receipts, physical-space/right/backing facts and complete firmware succession outstanding. Existing checked PIC/PIT/x2APIC helpers retain `PortIo` or `MachineControl` reach; that reach is not a per-device grant. Neither the bootstrap geometry milestone nor a new ACPI address may be used to fill those gaps by assertion.

The confined parser belongs in `libraries/acpi`; it receives initialized bytes. Public future broker interfaces belong in `contracts`, privileged issuance/mapping/IRQ lifecycle implementation in `core`, and device-specific placement and protocols in `drivers`, respecting their charters. This document introduces no dependency from a library or driver into core internals.

## Inputs that must remain separate

| Input | Establishes | Does not establish |
| --- | --- | --- |
| Parsed table snapshot and ordered facts | Checked bytes, lengths, checksums and the parser's stated interpretation | Authentic firmware, resource ownership, device correspondence or permission |
| Broker's live platform/device record | Separately established stable device instance, custody lineage, configuration epoch and supported profile | Permission for an arbitrary caller |
| Specifically named held parent grant | Its provider, principal, space, range, rights, lease and derivation lineage, after redemption | Rights outside that grant, or another grant found by searching |
| Caller request and manifest ceiling | Requested operation; maximum category the principal may receive | Held authority or automatic approval |
| Selected layout/access plan and provider supply | Compatible geometry, primitive operations and observation rules when admitted against the real resource | Device correspondence or fresh ownership merely from plan construction |

A table address, APIC ID, PCI BDF, GAS address-space number, copied `ObservedBootApic { known: true, ... }`, checksum or source digest remains data. Reconstructing one never reconstructs provider identity, routed qualification, a runtime occurrence, a lease or a live generation. Checksums detect the parser's specified corruption class; they are not authentication.

The broker records the table's complete selected snapshot identity, source offsets and interpretation profile alongside the device/configuration epoch. These are audit coordinates, not bearer credentials. Equality of cached coordinates does not replace revalidation against the live provider record. CPU hotplug, device replacement, reset, resource rebalance and resumed platform configuration may invalidate that association even if the same numeric ID or address reappears.

## Discovery before granting device access

1. The boot/platform custodian establishes table-read access through its actual admitted custody path. An RSDP, RSDT or XSDT pointer cannot authorize the next read. For each followed pointer, check the complete requested interval against a separately held source claim and the selected physical-address profile before access. Physical-address validation is distinct from virtual canonical-address validation.
2. Read the bounded header through that access, validate the advertised size against the parser's 4096-byte profile and provider range, then obtain the complete snapshot under the provider's coherence/version contract. An oversized table fails explicitly; a truncated prefix is not reported as a complete table.
3. Parse private initialized bytes. A hostile writer requires private copying or completed revocation of its write access. A copy makes subsequent parsing memory-safe; a possibly torn copy alone does not establish a coherent firmware version or device correspondence. Missing coherence/correspondence evidence prevents subsequent grant issuance.
4. Preserve firmware custody classes. ACPI reclaimable storage becomes reusable only through the custodian's succession/reclamation rules after all loans and retained users end. NVS, runtime and otherwise retained storage remain with their continuing owners. Successful parsing and a copied checksum do not reclaim the source.
5. Produce candidate facts only. Root-table traversal needs a bounded visit count, duplicate/cycle handling, bounded total copied bytes and an explicit failure on exhausted limits. These traversal controls are future adapter work, not claimed by the existing single-table parsers.

Discovery may use an existing narrow table-read grant without granting the parser any register or controller access. The parser does not receive the broker's issuance authority.

## Concrete attenuation transaction

Every derivation names one actual parent capability held by the principal performing that derivation (the broker or an authorized delegator), the target stable device instance/configuration epoch and one operation. There is no search for some equivalent authority elsewhere in the graph.

An ordinary driver need not own the broker's physical source extent. It invokes a specifically held, device-scoped broker capability; the broker separately supplies its own established physical/I/O source custody. The receiving grant is constrained by both the redeemed request authority and that source custody. The provider retains the source loan/claim and its lifecycle dependency; delegating an operation handle does not duplicate linear byte ownership. The grant ledger and provider record must retain these distinct derivation and backing dependencies, and either becoming unavailable prevents use. This document does not authorize an ambient request endpoint or a new root issuer.

1. **Redeem and identify.** The provider authenticates the current principal from its execution context, validates the named handle/lineage, lease and live generation, and checks the requested device instance and configuration epoch. It compares the request with that principal's manifest ceiling at grant insertion. Unknown or stale candidates cannot trigger probing through an ungranted address.
2. **Normalize the complete footprint.** Retain space, base, length, transfer width, absolute alignment, observation, operation and any implicit container/page footprint. Check additions using nonwrapping interval arithmetic, including the one-past endpoint. A GAS bit width, an ECAM function base or an I/O-APIC base is insufficient to determine all touched bytes; the selected device contract supplies the actual transfer/container/register footprint.
3. **Check containment and compatibility.** The whole physical effect footprint must fit the specifically held parent range in the same address space and backing lineage. Requested rights, operations, reach, lifetime and carry permissions must be subsets of admitted parent supply and Cathedral policy. A request that needs more is rejected, not silently widened or partially executed. A separately requested smaller grant may succeed.
4. **Preserve conservation.** For a call-scoped subrange use an ordinary permitted borrow. For independently transferred owned range authority, consume/split the parent under the exact content-conservation equation, retaining or returning all complementary ranges. Rights attenuation discards rights without creating or discarding byte ownership. Different spaces, eras, providers or numerically adjacent ranges are not merged by arithmetic.
5. **Choose enforcement.** Use a mediated fallible broker operation when the promised restriction cannot be enforced by a direct mapping. A direct mapped grant additionally requires separate destination virtual-space authority, exact source custody, admitted translation/cache policy, completed activation and a lifetime model satisfying the next section. Mapping metadata or a pure PTE validator is only a candidate until the provider receipt establishes the installed mapping.
6. **Bind interpretation.** Establish the selected `Placed<P, T>` view only over the real qualified range/borrow, with provider-bound `ResourceProfile` supply and a separate schema-to-device correspondence fact. The provider/plan/device instance, mapping era and exact field/operation must agree. Every omitted field is inaccessible. No generic cast or public base-plus-offset primitive is introduced.
7. **Publish only completed grants.** Insert the derived grant for the receiver through its authorized receiving endpoint, recording the parent edge and actual restricted rights/lifetime. Publish success only after the provider's establishment obligations are complete. A proposal, pending mapping or partially admitted plan is not a live returned capability.

A mapping rounded to a page includes the entire mapped page in its enforcement analysis. It cannot borrow neighboring bytes from an address coincidence. If hardware isolation exposes more registers or operations than the grant allows, retain the mapping in the broker and expose only mediated semantic operations, or reject. A foreign component with a writable MMIO page is not confined to one advertised field by an Omega accessor type it does not obey.

## Resource-specific admission

| Parsed fact | Additional evidence and initial restriction |
| --- | --- |
| SDT/RSDP physical table range | Read-only source custody and coherent snapshot/version. No write, allocation, execution or firmware-call right. |
| GAS SystemMemory | Actual device/register identity, complete register-container footprint and admitted External observation/width/alignment. It is not ordinary Stable RAM merely because GAS space is zero. |
| GAS SystemIo | An I/O-port-space parent and admitted port operation/profile. Validate the full transfer interval against the selected target's port-space bound; never truncate a u64 GAS address into a port operand. A physical-memory grant cannot substitute. |
| GAS PciConfigSpace | Decode and validate the GAS device/function/register coordinates under that format, then redeem the broker's matching segment/bus/function configuration authority. Do not reinterpret the packed GAS value as a physical address. |
| Other GAS spaces, including FFH and OEM | Preserve as facts; return unsupported for this initial adapter unless a separately selected, admitted provider defines that exact space and operation. Numeric recognition by `gas_from_raw` is not provider support. |
| MCFG region/function | Unique matching segment/bus interval, checked actual bus-zero-relative ECAM offset and complete requested register span, parent configuration-space custody, device-instance match and permitted operation class. Default to read/query mediation; BAR changes, bus mastering, reset and routing writes stay separately controlled. |
| MADT local APIC / x2APIC / I/O APIC / GIC | Match the selected machine/controller profile and actual CPU/device instance. x2APIC machine-register access is not an MMIO grant. Validate duplicates, conflicting overrides, controller model compatibility and supported entries before creating an operational configuration. The current ordered extractor does not perform those global checks. |
| MADT CPU and boot identity | Reconcile table order/flags with an independently established observed CPU identity. An enabled bit is not proof a CPU is running. CPU start requires separately admitted executable artifact/entry, stack, machine-control and lifecycle authority. |
| MADT interrupt source/NMI | Translate GSI/polarity/trigger facts through an admitted controller/routing policy. A GSI is not an installed vector or IRQ endpoint. NMI delivery has its own core policy; it is not delegated as an ordinary maskable driver interrupt. |
| FADT PM timer / HPET | Retain hardware-reduced suppression and the selected device's register semantics. Grant only the chosen counter/query operation initially. Reading a timer does not grant timer programming, interrupt delivery, power-state transitions or trusted-clock status. |
| Wake mailbox, FADT reset/sleep or SMI command | Facts only for this adapter. Their state-changing use requires separate system-control authorization and a complete bounded protocol; finding the address never enables it. |

DMA is not implied by an MMIO or PCI configuration grant. It requires independently held RAM custody, device direction, IOMMU confinement and exact loan/completion receipts. No bus-master-enable write precedes that confinement. Completion status alone does not prove device release. This task adds no DMA operation or completion constructor.

External register access remains External. A destructive read needs the corresponding whole-container take/snapshot contract; a nominal read-only grant does not silently permit that side effect. W1C, FIFO, selector/data pairs and posted-write flushing require authored device protocols. No generic read-modify-write or Atomic permission is inferred from bitfield geometry, exclusive receiver borrowing or a CPU fence.

## Example: narrow ECAM query

Suppose the candidate MCFG record describes base `0x80000000`, segment `0x1234`, buses `0x80..0xff`. Bus `0x80`, device 0, function 0 has numeric function base `0x88000000`; the managed range's start is not subtracted. A request for a four-byte register at function offset `0x100` touches `[0x88000100, 0x88000104)`.

The broker still needs a live configuration-space parent covering that interval, a matching device/configuration epoch and permission for that register's four-byte read. The containing page `[0x88000000, 0x88001000)` needs independently justified mapping custody. If direct access would expose forbidden control registers on that page, the broker performs the narrow read and returns detached data. The caller receives neither the rest of the page nor reset/DMA authority.

A numeric function base at `u64::MAX` may be preserved by a pure query with zero displacement; any multi-byte read extending beyond the selected address-space bound is rejected here. Checking the first byte alone is insufficient.

## Lifetimes, revocation and recovery

The grant records its holder instance, parent chain, stable backing/device identity, source and destination spaces, allowed interval/operations, configuration/mapping era, expiry condition and carry restrictions. Borrowing cannot outlive the parent claim. Delegation remains subordinate to the parent; transfer uses the existing atomic handoff semantics. Restart, persisted handles or equal-looking replacement hardware do not revive an old grant. Reboot requires reconciliation against the new platform instance before device use.

Two enforcement modes must remain explicit:

- **Mediated revocable operation.** The provider checks the live grant at each operation or bounded chunk and returns a typed revoked/expired result before starting rejected work. Already admitted in-flight effects follow the declared operation contract; revocation cannot pretend a completed physical effect was rolled back. A later operation is not silently replayed on a replacement device.
- **Direct placed view.** Its admitted lease/claim prevents asynchronous invalidation while the view is live. Primitive access has no per-access generation probe and no recoverable mapping-fault result. Polite revocation stops new loans, requests retirement and waits for all relevant views/calls to end. A grant requiring asynchronous rejection must instead use a fallible provider protocol. Hard teardown of a hostile or failed protection domain is a containment/fault boundary, not normal continuation through a still-valid Omega view.

Retirement closes admission, drains or contains users, handles the device's interrupt/DMA dependencies, ends placed loans, and completes mapping teardown and all required cross-core/IOMMU invalidation acknowledgements before storage or identifiers are reused. CPU-affine grants retain their affinity; suspension/migration is allowed only where the accepted carry contract and runtime preserve it. An interrupt root may not inherit a blocking drain operation simply because ordinary task cleanup can wait.

Generation invalidation alone does not retire a mapping or stop DMA. If release, device reset, shootdown or quiescence cannot be proved, return/retain the exact pending recovery custody and quarantine the affected resource; do not return a reusable range or report clean completion. Root/device/session occurrences must not be recreated from an era integer after failure. Reset is a core-controlled containment/recovery operation and shared reset domains may affect siblings.

Cathedral's capability model specifies lazy ancestor-generation checking, while one lifecycle paragraph still describes an eager subtree walk. This adapter requires transitive invalidation and the active mapped-resource teardown above; it does not settle that documentation discrepancy or depend on a particular arena traversal cost.

## Rejection contract

Before any hardware access or successful issuance, preserve the candidate and return every moved claim unchanged, or end an exact borrowed loan, for:

- malformed/oversized/incomplete input, unsupported interpretation/provider or exhausted traversal budget;
- absent parent, wrong principal/device/space, expired/revoked lineage or stale configuration;
- out-of-range/overflowing footprint, misalignment, ambiguous/overlapping region match or insufficient rights;
- unsupported observation/operation, schema/device mismatch, missing correspondence or invalid carry/lifetime;
- absent virtual destination authority, incompatible page exposure or incomplete mapping/installation evidence.

Distinguish these ordinary data reasons from provider trust violations and hardware faults. Known static plan/supply incompatibility is rejected at checking/installation. An admitted primitive write commits its hardware effect; a physical fault has the declared crash behavior, not a fabricated transactional error. Failures after a state-changing protocol begins retain the live pending operation/recovery inputs and report the exact outstanding obligations rather than returning an apparently untouched grant.

Absence of a grant is never successful hardware access through a zero/null placeholder. Unknown ACPI entries can remain in the discovery report without granting access or being silently selected as a supported device.

## Integration gates and review evidence

The eventual implementation must demonstrate the following with a synthetic provider before any live-device integration; **none of these adapter tests has run in this specification milestone**:

| Required test | Observable acceptance criterion |
| --- | --- |
| Reconstructed grant/profile/receipt and cross-space substitution | No access event or live capability; original custody returned |
| Exact subrange and rights reduction | Full footprint contained; complementary owned ranges conserved; child cannot recover discarded rights |
| Page/container expansion beyond grant | Mediation or rejection; no hidden widened mapping |
| ECAM end overflow, malformed GAS port, overlap and stale device epoch | Typed rejection before access; no truncation, first-match selection or reprobe under stale authority |
| Hostile snapshot mutation / firmware custody change | No unjustified coherent-version claim or premature reclaim |
| Revocation before operation / during in-flight protocol | Pre-start rejection or declared pending-effect recovery; no silent replay |
| Outstanding placed view, DMA loan or cross-core invalidation | No successful retirement/reuse until the corresponding obligations finish |
| Reset failure, CPU affinity mismatch, unsupported unknown entry | Quarantine or explicit refusal without minting substitute authority |

Review must retain the source-table evidence, target/device profile, exact parent/child lineage, principal/manifest decision, admitted provider and structural placement identities, operation footprint, lease/era transitions and success/rejection custody disposition. Audit identifiers do not substitute for the actual evidence they index.

Remaining implementation work includes real boot custody succession, broker interfaces and provider admission, device correspondence, complete placement/lowering, mapping activation/teardown, IRQ installation, IOMMU/reset lifecycle and integration tests. This document adds no placeholder schema whose fields could be mistaken for any of those established capabilities.
