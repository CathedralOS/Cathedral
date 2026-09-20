# Bounded static AML syntax and namespace layer

Current stage: **tested** for the static subset below. All 27 semantic cases and
27 body mutations pass; the complete package source-checks as 18 files.
[Recorded verification](../../../../tools/ports/acpi/aml/verification.json)
binds compiler, package and fixture hashes to those results.

This is a staged ACPI-004 implementation, **not a complete AML interpreter or a completed ACPI-004 checkbox**. It is an independent `cathedral-acpi-aml` package with no production boot import. The public entry `loader::load` consumes an already checked definition block's AML TermList. It does not find, map, checksum, dispatch, or execute firmware tables.

The source is translated and adapted from [rust-osdev/acpi](https://github.com/rust-osdev/acpi/tree/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml), exact revision `257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5` (crate 6.1.1). Upstream copyright is 2018 Isaac Woods, licensed MIT OR Apache-2.0; exact texts remain in `licenses/rust-osdev/acpi/LICENCE-MIT` and `LICENCE-APACHE`. Omega representations, resource profiles, transactional errors and original fixtures are Cathedral adaptations, not unmodified upstream code. `inventory.json` binds all six pinned AML source files and every scanner anchor; `tools/ports/acpi/aml/coverage.json` additionally maps all 113 opcode/range facts to the supported static subset or pending behavior; incomplete operations remain **pending**, not omitted or blocked.

## Implemented boundary

- Bounds-checked little-endian reads; extended and negated opcode tokenization; all pinned raw opcode/range facts; validated package envelopes and every encoded NameString form.
- Integer constants (including AML Revision), ASCII string spans, buffer descriptions, fixed packages and literal-count variable packages. Packages have an explicit parser stack, uninitialized padding, and linked object IDs. Package NameStrings retain their declaration scope and resolve lazily; forward references are not mistaken for method invocation.
- Static Name/Alias, Scope, Device, Processor, PowerResource, ThermalZone, Method, External, Mutex/Event and literal-address OperationRegion declarations. Methods retain flags and uninterpreted body spans. External declarations retain metadata without manufacturing a defined namespace object. Mutex, Event and OperationRegion are inert descriptions, with no runtime lock/event/region handler.
- Absolute and parent-prefixed resolution; nearest-ancestor lookup only for a single unqualified segment; separate level and object slots; stable alias identity across rebinding; bounded lazy reference chasing with exact repeated-object cycle detection.

Dynamic TermArgs, executable NameString invocation, fields/CreateField, If/While/control flow, methods/locals/arguments, conversions/operators, synchronization, region access, table loading/unloading, namespace traversal/removal, generic string-based names, and resource/PCI routing evaluation remain pending. Unsupported executable terms return `UnsupportedSyntax`; method bodies are deliberately retained without interpreting their contents. The parent inventory must not infer completion from this package's static subset.

## Resource and ownership policy

The initial profile is **1024 initialized input bytes, 16 path segments or parent prefixes, 32 namespace path entries including root, 64 object slots, eight package frames, eight scope frames, and 32 External records**. These are Cathedral resource limits, not ACPI maxima. Independent caller budgets bound loader turns, per-value parser turns (at most 1024 each), and reference hops (at most 64). A driver may consume remaining turns as terminal no-ops; these counters are not compiler logical-work or instruction meters.

`Capacity`, `Depth`, `WorkLimit`, `Truncated`, `BadEncoding`, `UnsupportedSyntax` and namespace errors are semantic outcomes. `load` restores the input namespace on any failure. Lower-level `parse_value` exposes a partial candidate arena with its error; callers must not install it as a successful result. Replacement allocates a fresh object ID; existing aliases retain the old ID. IDs are not reclaimed in this profile, so repeated replacement can exhaust the arena.

All records are ordinary initialized data, not validated authority types. Namespace inputs should come from `empty`, explicit seeding, or previous successful namespace operations. The public indexed `entry_at`/`object_at` kernels use physical-capacity guards and return defaults outside those capacities; consumers must use checked lookup and require an object ID below `object_count` before interpreting a slot. They are not namespace-validation certificates. `Span { unit, start, end }` retains byte offsets, never an address or live borrow. The caller must retain the matching immutable input and assign distinct unit IDs when combining definition blocks. A span, region space code or namespace ID grants no right to access hardware or invoke a handler. No layout policy or native ABI claim applies to these semantic cases.

`namespace::empty` creates only root. `predefined_scopes` also creates `_GPE`, `_SB_`, `_SI_`, `_PR_`, `_TZ_`. The pin's global-lock handler and `_OS`/`_OSI` impersonation choices are deliberately deferred to host policy. They are not silently initialized.

## Interpreter handoff

The public namespace functions take ordinary `Namespace` values. `get` performs exact absolute lookup; `search` resolves a name against its declaration scope and applies ancestor search only to a single unqualified segment. A successful `Lookup.object` identifies an existing object. Consumers must check `object < object_count` before reading it with `object_at`. `references::resolve_reference` follows package name references using their retained declaration scopes.

Declaration `insert` creates a fresh object identity, so it must not implement interpreter Store. A future Store updates the existing checked slot with `object_set`, preserving the `Object.has_next`/`next` package linkage and alias identity. Method bodies retain only flags and a `Span`; the execution caller must supply matching immutable bytes, validate the unit and span bounds, and establish its own frame, argument and work policies. This package supplies no unit-to-input registry, method-local teardown, runtime object conversion, or execution service.

## Pin behavior, validation differences and specification

The grammar review uses the [primary ACPI 6.5 Errata A AML grammar](https://uefi.org/specs/ACPI/6.5_A/20_AML_Specification.html), sections 20.2.2–20.2.5. Names and ancestor lookup were checked against [ACPI 6.6 section 5.3](https://uefi.org/specs/ACPI/6.6/05_ACPI_Software_Programming_Model.html#acpi-namespace). These sources define wire grammar and namespace rules; the bounded profile is separate.

| Behavior | Pinned evidence | This adaptation |
|---|---|---|
| PkgLength reserved bits / invalid envelope | `mod.rs::pkglength` ignores bits 4–5 in extended form; consumers subtract lengths | Rejects reserved bits, lengths shorter than their encoding and enclosing-bound crossings |
| MultiName count zero | `mod.rs::namestring` accepts the empty loop | Rejects zero count; NullName has its own encoding |
| Buffer initializer exceeds declared length | `mod.rs` Buffer retirement takes a shortened destination but the whole initializer source for `copy_from_slice` | Stores declared length and complete initializer span without allocating/copying; eventual materialization must truncate/pad |
| Extra Package elements | `mod.rs` Package retirement asserts end; VarPackage completion subtracts supplied count from declared count | Recoverable `BadEncoding` |
| Alias collision | `namespace.rs::create_alias` inserts before returning NameCollision | Checks collision before mutation |
| Rebinding names / opening levels | `namespace.rs::insert` replaces; `add_level` creates/reopens and preserves existing kind | Preserved as explicit pin-compatible namespace operations, even though ACPI 6.6 describes load-time name collisions as fatal |
| Scope declaration | `mod.rs` Scope resolves against current scope then calls `add_level` | Preserves that pin behavior, including root Scope; this is not a claim of full spec Scope interpreter semantics |
| Missing root object | `get_level_for_path` asserts that path is not root | Recoverable missing object |
| Strings / Mutex / External metadata | Pin strings use UTF-8 conversion; sync byte and External fields are permissive | ASCII strings, reserved mutex sync bits and External type/argument metadata checked before installation |
| Lazy reference limits | Pin `MAX_NAME_PATH_INDIRECTIONS` is 8 and exhaustion yields NameResolutionLoop | Caller-selected budget through 64; repeated IDs produce ReferenceCycle, budget exhaustion WorkLimit |
| Full namespace constructor | Pin installs host mutex and Windows identity behavior | Only explicit pure scope seeding; host decisions pending |

## Verification and update audit

Run `python3 tools/ports/acpi/aml/audit.py`, then `python3 tools/ports/acpi/aml/check.py --omega /tmp/cathedral-omega-eaa7993/release/omega`. `fixtures.py` regenerates independently authored bytes and expectations. Each case has a unique mutation **inside its expected behavior**, and the negative must compute failure 1 and be rejected by `requires result == 0`.

Validation is semantic evaluation, not native execution, Omega/Rust ABI agreement or execution of an AML interpreter. The harness records the compiler hash. Source hashes, license hashes and metadata-only upstream test provenance are audited separately. No firmware dump, `uacpi_examples.rs`, external global-lock example, or firmware-derived package fixture was copied.

When changing the pin: review license and manifest changes, hash and review every AML source/test delta, reclassify all changed anchors, review opcode ranges and each validation difference above, regenerate original fixtures only after semantic review, and rerun both audits and body controls. An inventory snapshot alone cannot authorize a completion claim.
