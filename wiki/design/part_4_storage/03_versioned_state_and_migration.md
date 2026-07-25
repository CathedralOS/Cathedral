# Chapter 03: Schema Lineages & State Migration

> Runtime identity, durable-format identity, and component-artifact identity are different things. Cathedral composes them explicitly instead of asking one magical versioning construct to stand for all three.

## The Legacy Model

Traditional systems mix several unrelated meanings of “version”: an in-memory struct revision, a file-format number, an API revision, and the binary that happened to implement old behavior. Hand-written ladders and migration scripts then try to recover which meaning applies at a particular disk, wire, or live-update seam. Silent in-place edits, reused tags, unhandled old data, and nondeterministic migrations are common outcomes.

## The Cathedral Model

Omega has no builtin `Versioned<T>`, `.prev` type path, or `replace` DSL.
Cathedral builds each lineage from ordinary immutable schema data, ordinary
sums, layout/codec policies, provenance domains, and migration machines. Live
replacement is a separate Cathedral protocol over admitted artifacts,
requirement bindings, era/liveness pins, candidate resource provision, and
runtime capabilities.

### External format lineages

Each published era has a permanent shape. The runtime type may evolve freely because it is never persisted directly:

```omega
data CounterV1 {
    counter: i32;
}

data CounterV2 {
    counter: i32;
    timestamp_ticks: u64;
}

data CounterEnvelope {
    case V1(value: CounterV1);
    case V2(value: CounterV2);
    case Unknown(era: EraId, bytes: Vec<u8>);
}

data Counter {
    counter: AtomicI32;
    timestamp: DateTime;
}
```

The external version set is open; one binary's knowledge is closed. Decode is the boundary where Cathedral must choose what happens when the world exceeds that knowledge: reject, preserve opaque bytes, or negotiate another representation. Exhaustive handling covers every known era, while the explicit unknown policy covers future eras.

A schema identity is not merely a shape hash. It is a normalized typed identity containing structural and authored nominal commitments. Compatibility and refinement are separate deterministic certificates connecting identities; compatible evolution necessarily produces a different identity.

### Migration

Migration remains an ordinary trait and ordinary machines. There is no magic era dispatch:

```omega
trait Upgradable<Old, New, Context = Nothing> {
    machine upgrade(old: Old, ctx: Context, out: &mut New)
        requires exclusive(old)
        ensures out in New::Valid;
}

machine upgrade_v1(old: CounterV1, ctx: CapturedClock, out: &mut Counter)
    satisfies Upgradable<CounterV1, Counter, CapturedClock>
{
    out.counter = AtomicI32::new(old.counter);
    out.timestamp = ctx.now;
}
```

Effects belong in a preceding capture phase. The upgrade machine consumes owned old state plus owned captured context, mutates only its exclusive output, and calls only deterministic callees. That makes the transform replayable and independently testable. If preparation can fail, it must do so while the old state remains recoverable; taking old state is an explicit point of no return after which the path is infallible-or-install.

Machines written today over `CounterV1` are current code over an old shape. They are not evidence of what the historical V1 artifact did. Historical behavior identity lives only in retained content-addressed component artifacts.

### Live component replacement

The operational protocol is a Cathedral/library concern:

1. validate and admit the candidate artifact;
2. provision peak coexistence and declare drain/disposition policy;
3. capture effectful external context;
4. run the deterministic state transform;
5. publish through an era-safe requirement binding, or restore while
   restoration remains meaningful;
6. drain, retain, migrate, restart/cancel, redirect, or acknowledge-transfer
   every old-era obligation; and
7. reclaim each lifetime cohort only when its residual population is empty.

Linear phase tokens can make local skipped cleanup or double completion compile
errors. The live population also includes runtime entities the source checker
cannot see, so Cathedral maintains an era ledger with provider receipts. The
runtime supplies artifact loading, requirement binding, liveness pins, era-safe
selection, resource admission, and reclamation; Cathedral supplies
orchestration policy.

## Safety Properties

- **Forgotten known era:** exhaustive matching or lineage-conformance law failure.
- **Changed published bytes under an old identity:** publish-time predecessor diff failure.
- **Reused retired identity/tag:** tombstone failure.
- **Future data misread as current:** explicit unknown-era policy.
- **Forged boundary provenance:** owner-evidenced decode domain; constructing a shape does not mint provenance.
- **Nondeterministic migration:** capture/upgrade separation plus ownership, access, and callee-contract checks.
- **Wrong live provider after replacement:** requirement/provider/era identity,
  admission refinement, and liveness pins.
- **Semantic drift without a changed declared contract:** residual risk; the language cannot infer durable meaning the author never stated.

## Ergonomics

The irreducible work is authoring historical shapes, durable meaning, migration logic, and unknown-era policy. Generators may transcribe traversal and codec glue, but never decide whether a field persists or what it means. Cathedral should build three serious format packages—save-game style opaque preservation, negotiated API evolution, and phased OS-state migration—and promote language surface only if all three expose the same unexpressible concept. Repeated verbosity alone is library/generator evidence, not automatic grounds for syntax.

## Open Questions

- For each store family, is migration eager at mount, lazy at load, or background and receipt-tracked?
- Which compatibility policies does Cathedral standardize for storage, IPC, and public APIs, and which remain package choices?
- Where should the generic replacement orchestration live once Omega's machine-parameter and monomorphization work can express it fully: Cathedral, a shared package Cathedral consumes, or eventually Omega core?
- Can a particular migration be certified lossless/reversible, or must rollback always name a separately checked inverse or retain the old artifact/state?

## Omega Leverage

- Ordinary `data`, sums, exhaustive matching, stable field identities, layout policies, and codecs define format lineages.
- Domains and sealed introduction distinguish constructibility from trusted decode provenance.
- `satisfies` inherits the migration requirement's contract; implementations do not restate it.
- Ownership, linearity, effects, and admission receipts enforce the phase protocol without a replacement DSL.
- Normalized typed identities and deterministic compatibility certificates make every seam report what it encountered.

## Related

- [[filesystem_as_database]] — durable records and compatibility policy.
- [[updates_and_hot_swap]] — live artifact coexistence, quiescence, and cutover.
- [[memory_and_persistence]] — runtime state vs. durable schema identity.
- [[ipc_and_service_invocation]] — format lineages at protocol boundaries.
- [[package_system]] — publication, predecessor diffs, tombstones, and admission.
