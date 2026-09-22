# Resumable normal Field reads with synthetic completions

This is an incomplete transcription checkpoint on the `50d8b3c` lineage.
The retained 213 ordinary execution and bridge regression pairs passed, but the
first actual Field-read pair fails checked interpretation with
`equality operands have no supported comparison`. A decoder-only experiment
also failed. No successful Field-read execution is claimed; see the exact
[verification status](../../../../tools/ports/acpi/pipeline/field-reads/STATUS.md).

The retained draft had two transitions to separate machines: `turn::pending`
targeted `prepare`, and `complete::word` targeted `received`. This candidate
replaces those transitions with local states that make ordinary machine calls,
following Omega's machine-local transition rule. The copied Integer completion
diagnostic passes all four selections with the wrapper; its original missing-state
failure is also retained. Actual Program/session validation of this candidate
is pending, and the earlier equality trap remains unresolved. The original full
70-pair run was stopped before checked/behavior results; exact inputs and
termination evidence are retained. The 213-pair receipt remains historical.

This draft implements method execution intended to suspend on a normal Field read
and continue after an explicit caller-supplied native-word completion. It performs
no provider call, physical access, mapping, grant redemption or lock operation.
Verification results belong to the separate receipts under
`tools/ports/acpi/pipeline/field-reads`; this document alone claims no execution.

The composition reuses the licensed rust-osdev/acpi adaptations documented by
the field installation, geometry and read-assembly ports. The session, request
protocol and synthetic fixtures are original Cathedral code. The pinned upstream
revision remains `257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, MIT OR Apache-2.0,
copyright 2018 Isaac Woods. Existing notices and inventories remain authoritative;
whole `do_field_read`, provider integration and ACPI-005 are still incomplete.

`field_reads::begin` receives an owned Program snapshot, method/arguments,
IntegerSize, total method fuel through 1024, and a caller-chosen correlation
number. `advance(session, quantum)` executes at most that many AML turns (quantum
through 1024) and returns Paused, Read, Finished or Failure. Zero is a poll.
The total fuel is retained across every call. Preparation follows the existing
method/argument admission order. `run_program` remains the legacy no-provider
API and still rejects field evaluation with UnresolvedRegion.

`begin_limited` also accepts the existing Program result object/byte quotas.
`begin` uses the same defaults, 64 objects and 16384 bytes. Invalid quota bounds
reject before argument allocation. On method completion, the shared
`program::admit_result` applies the same reachable-result graph validation as
`run_program_limited`; its admitted or rejected result is retained once for
subsequent polls. Actual graph quota failure preserves earlier execution effects
and completed reads, and publishes no result root. Inline Integer results need
no graph quota. References to unsupported Field metadata remain nonreading but
fail public result graph admission.

The execution package adds a disabled-by-default decoded-value continuation,
containing the exact resolved field object and original/next instruction cursor.
Only the new retained-runtime entry enables it. Nested method frames inherit the
mode. The field decode is charged once; issuing/accepting its native reads and
contributing its completed value do not charge that decode again. The separate
transfer bound is at most 257 native observations for each of at most 1024 AML
turns, hence at most 263168 issued requests per session. Accepted words for the
current transfer and cumulative counters remain available after a failure.
Completed values and method effects remain in the runtime/store; the synthetic
caller retains the complete historical request log.

Ordinary Name/Local/Arg value admission can suspend. Metadata query dispatch is
separate. Explicit RefOf/Index values remain references unless an existing
operation explicitly dereferences them (such as the established explicit-reference
Arg read policy). Return, arithmetic and predicates consume the completed value;
a bare Field statement still performs the read and discards its contribution.
No target retirement or write protocol is added. Field writes, including
Increment/Decrement and CopyObject destinations, retain their existing rejection.

Before issuing the first request the pipeline captures the resolved canonical
FieldUnit object and its inline binding/declaration/field metadata. It validates
that a Region binding matches a normal Field declaration, retained source
coordinates and OperationRegion identity,
the synthetic SystemMemory profile (space zero), complete relative geometry,
unmet-lock rejection, absolute native alignment and nonwrapping base plus footprint.
The exclusive endpoint must be representable: if `base + plan.end` exceeds u64,
the read rejects, including the topmost-address case whose one-past endpoint
would overflow. Geometry alone checks relative bounds and supplies no base or
space permission. Consistent Bank/Index bindings, unsupported access metadata
and other spaces fail before any request; mismatched binding/declaration kinds
fail InvalidState. LockRule never becomes a held lock.

Integer results are inline. Wider results use the canonical owned Buffer layout;
one free object slot is checked before reads begin. Completed native words feed
the existing `field_values::read::assemble`, which independently checks geometry
and count and initializes all result bytes. New Buffer allocation, canonical
ByteBlock publication and saved-frame contribution are staged together and
committed only on success. Namespace bindings and installed field metadata retain
their identities. Earlier successful method stores remain committed if a later
read or method step fails.

There is exactly one outstanding `NativeRead`. A completion must match every
coordinate: correlation number, monotonically issued serial, source unit, field
and region object IDs, space, base, relative offset and width. Mismatched, stale
and duplicate completions return InvalidState without consuming the request,
changing observations, advancing the cursor or charging fuel. Polling returns
the same request. A matching failed completion terminates the session while
preserving its completed words and previous method effects. A Failure carrying
Success is invalid and does not consume the request.

Correlation numbers are ordinary caller-chosen data. Callers multiplexing sessions
must choose distinct numbers for simultaneously outstanding synthetic sessions.
Matching numeric coordinates is not authenticated cross-session provenance.
Neither a Session, cached Plan, request, successful completion nor region/base
number grants hardware access. The session owns its source/store throughout
continuation; callers must not replace or modify them between API calls. This
synthetic interface deliberately accepts caller-supplied observations and makes
no claim about their real-world origin or coherence. A later real provider must
satisfy the held-grant, device correspondence, observation, synchronization and
lifecycle obligations in `../ADAPTER.md` through its actual capability path.

The bridge-atomicity comparison helper is extended for the new Frame mode and
every DeferredRead field, and for canonical FieldUnit identity. Historical
receipts are not rewritten or relabeled. A new receipt must bind those helper
inputs before claiming full extended-Frame preservation.
