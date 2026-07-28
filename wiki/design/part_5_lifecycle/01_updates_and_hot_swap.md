# Chapter 01: Updates And Hot Swap

> Cathedral owns the operational policy for replacing a live provider
> realization. Omega supplies checked contracts, artifacts, resource demands,
> ownership/liveness facts, and admitted boundary operations; it has no
> replacement keyword or component manager.

## Terms

A **package** is source organization and dependency reach. A **requirement** is
a behavioral contract. A **provider realization** is code and state satisfying
that requirement. A Cathedral **component** is a realization selected for
independent deployment or replacement, plus the closed code, state, resource,
and metadata graph it owns.

A component is therefore not a package or one arbitrary machine by definition.
Cathedral may initially accept only closures that coincide with whole packages,
but that is an implementation restriction. Removing it must only admit more
valid closures.

Calls inside a component may name concrete machines and inline normally. Every
incoming edge from outside a replaceable closure names a requirement contract.
The same requirement may be statically selected in another build. There is no
`slot` keyword or hot-swap call syntax; Cathedral's loader/runtime maintains the
replaceable binding selected by the deployment.

## Division of labor

Omega owns:

- normalized requirement, provider, calling, representation, artifact, and era
  identities;
- checked conversion machines and historical data shapes;
- candidate-specific stack, structural-work, and machine-state demand;
- closure/leak checks and two-sided component import/export validation;
- liveness pins and general installation, visibility, and quiescence facts; and
- admitted executable installation with no arbitrary bytes-to-code operation.

Cathedral owns:

- which realization is independently deployed or replaced;
- stack/storage provision and peak-coexistence admission;
- the binding-era algorithm and live-era ledger;
- drain, cancellation, coexistence, migration, restart, health, rollback, and
  eviction policy;
- device quiescence and authority transfer; and
- loader mappings and reclamation of lifetime cohorts.

`boundary` remains a trust, ABI, or externally supplied entry marker. It is not
a replacement marker.

## Local dispatch does not cross the replacement boundary

Omega's `dyn Trait` is local dispatch inside one compiled artifact: a borrowed
value is an instance pointer plus a selected-conformance table pointer. Its
per-requirement operational envelope is compile-time contract information, not
extra runtime words. A local descriptor never crosses a replaceable component
edge because its table uses artifact-local calling conventions and its code or
state may be reclaimed.

A boundary-trait value instead names Cathedral's selected binding. Calls enter
through the binding's published boundary plan and era protocol. Application
code that wants a local `dyn` interface over a replaceable provider owns a
proxy in its own artifact; the proxy holds the boundary binding and implements
the ordinary trait by making real boundary calls. This concentrates ABI,
effect, lease, and replacement cost at one named seam without laundering a
local table across it.

Binding multiplicity uses Omega's ordinary multiplicity vocabulary and is
affine unless the boundary trait author permits copying or requires linear
release. A rebindable binding normally resolves the current era on each call; a
pin is a separate linear value that intentionally retains one era. Binding
entry publishes its operational/resource ceiling, and Cathedral's admitted
entry algorithm must fit it.

Provider-originated session claims may retain an old era even after new calls
route elsewhere. Each such claim records immutable origin separately from its
current custodian. Only checked transfer to a named receiver, a boundary
receipt, or preservation under moves changes custody. Replacement reports name
retention paths rather than merely saying “not quiescent.”

Cathedral supports three explicit capability tiers:

1. **drain/coexist** — new calls route to the new era while old sessions finish
   or keep their old provider alive;
2. **explicit migration** — application/provider code cooperates to replace
   existing session values;
3. **stable object identity** — an object table may redirect handles without
   application cooperation, but only alongside a proved state migration and a
   defined racing-call disposition.

Tier 3 is not implied by indirection and is not required for short-lived
sessions.

## Replacement protocol

Replacement is a checked Cathedral state machine over admitted providers:

```text
validate candidate
    -> provision candidate and peak coexistence
    -> prepare reversible state work
    -> publish new binding era / close old era
    -> dispose of the old population
    -> reclaim empty lifetime cohorts
```

Before publication, the replacement plan declares:

- its point of no return;
- its drain policy: bounded cancellation backed by a real contract, or accepted
  indefinite coexistence and its retention cost;
- the disposition of owned state, live activations and their retained stacks,
  registrations, authorities, and device claims; and
- enough resource provision for every retained era plus the candidate.

Before the point of no return Cathedral may abort and discard the candidate.
Afterward recovery rolls forward or performs a separately admitted reverse
replacement. A timeout does not authorize dropping an obligation; cancellation
must be part of the component contract and must discharge owned resources.

## Era-safe entry

Cutover is not an instantaneous pointer assignment. A caller may observe the
old binding and be descheduled before entering it. Cathedral's entry protocol
therefore has a linearization rule:

```text
acquire current era
    -> resolve the entry for that era
    -> execute
    -> release the era
```

Closing an era and publishing its successor must place every racing caller in
exactly one population: counted against the old era, or redirected to the new
one. RCU, active counters, hazard references, or another admitted algorithm may
realize this contract.

Code visibility and binding-era quiescence are different:

- visibility proves future entrants observe the newly installed bytes;
- quiescence proves no reader may subsequently enter the old realization.

They may reuse completion-obligation infrastructure but never one fact or token
type.

## Complete disposition, not universal migration

Every item in the old population receives one explicit disposition:

- drain to completion on the old era;
- coexist under the old era;
- migrate through checked code;
- restart or cancel under contract;
- redirect to another owner; or
- transfer to a named receiver that acknowledges the obligation.

The source remains responsible until the receiver accepts. Cathedral may use
era counts or conservative summaries rather than enumerate every activation
individually, but reclamation requires proof that the relevant residual
population is empty.

A parked activation retains its fixed stack and normally pins its code, unwind
metadata, provider-owned session claims, and state era. It must resume and
drain, cancel validly, migrate through a separately validated activation/state
transformation, or retain that era indefinitely. Architectural preemption does
not create a lifecycle transition and therefore cannot substitute for one of
those dispositions.

“New routing is active” and “the old era is reclaimed” are separate completion
states.

## Resource admission

Semantic compatibility does not freeze implementation resource demand.
Every candidate carries target-specific realized demand; Cathedral admits it
against current provision.

A replacement needing a larger `StackPlan` is legal for new activations when
Cathedral can provision matching fixed stacks before publication. Existing
activations keep their old stacks and code era until disposition; a live stack
is never resized underneath them. A fixed budget belongs to a requirement only
when policy intentionally promises replacement without reprovisioning—for
example, an already-provisioned hard-root class.

Admission covers peak coexistence, not just the candidate: old and new code,
state pools, activation metadata, stacks, and device claims may all be live
during drain.

The ledger is a set of live eras keyed by identity. Cathedral declares a
bounded maximum live-era policy. A two-era implementation is a legitimate
first restriction; increasing the bound only admits more replacements.

## Mapping and reclamation

The reclaimable unit is a mapping lifetime cohort, not a source section or
entire component. Code, immutable data, mutable state, relocation/unwind
metadata, and activation-stack pools may have different lifetimes. Objects that
must be unmapped independently cannot share a page.

Nothing outside the component may retain an untracked concrete code address or
raw pointer into a reclaimable cohort. Cross-component calls use requirement
identity; state crossing the edge uses validated component representations and
version-aware ownership.

## Devices

Software activation quiescence does not prove device quiescence. A driver
replacement also accounts for:

- in-flight DMA and IOMMU loans;
- interrupt sources and pending acknowledgements;
- device queues and firmware state;
- mapping invalidation and cross-core completion; and
- reset scope, including collateral devices where reset is not isolated.

The driver may drain, reset/restart, coexist, or defer replacement. Cathedral
must report which policy blocked reclamation.

## Still open

- concrete era-acquisition implementation;
- replacement-plan, migration, and disposition receipt schemas;
- maximum-live-era and eviction policy;
- outbound calls from old activations;
- activation migration across stack plans;
- crash recovery across a committed replacement; and
- device-specific quiescence protocols.

These are Cathedral runtime/design questions. They do not justify Omega
replacement syntax, a package-equals-component rule, or compiler-owned OS
lifecycle types.

## Related

- [[component_model]] — component, instance, task, and service identities.
- [[versioned_state_and_migration]] — checked historical shapes and state
  transformations.
- [[memory_and_persistence]] — crash recovery during replacement.
- [[package_system]] — delivery of candidate artifacts.
- [[driver_model]] — the hardware-quiescence customer.
- [[capability_lifecycle]] — authority transfer and revocation.
