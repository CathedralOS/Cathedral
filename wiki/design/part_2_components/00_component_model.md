# Chapter 00: Component Model

> The unit Cathedral actually runs, isolates, restarts, and upgrades — and the recognition that those are *not one unit* but a family of overlapping ones.

## The Legacy Model

The Unix process is one of computing's great overloaded nouns. A single `pid` simultaneously names an address space, a permission identity (uid/gid), a file descriptor table, an environment block, a signal-handling context, a lifecycle (fork/exec/wait/exit), a scheduling entity, and a crash boundary. These are welded together because in 1970 they were cheap to weld. The cost shows up everywhere after: you cannot restart the crash boundary without losing the address space; you cannot upgrade the code without killing the identity; you cannot isolate authority more finely than the process because authority *is* the process. Everything that wants a different granularity — threads, containers, namespaces, sessions, transactions — is bolted on as a partial, ad-hoc escape.

## The Cathedral Model

Stop pretending one noun fits. Cathedral models an explicit **family** of units and lets each concern pick the granularity it needs:

- **Component** — the unit of *code identity and deployment* (what the store ships, what the loader instantiates).
- **Instance** — a live running occurrence of a component, with its own state.
- **Task** — the unit of *concurrent execution*; a single running state machine. An instance is one or more tasks.
- **Actor / Service** — the unit of *protocol identity* (what others invoke).
- **Driver** — a component bound to a device, with a distinct trust and restart story ([[driver_model]]).
- **Session / Transaction / Job** — units of *work* with their own lifetime.
- **Tenant** — the unit of *isolation and accounting* across users/orgs.

The point is that the unit of isolation, of restart, of hot swap, of authority, of scheduling, of persistence, and of upgrade are *separately chosen axes*, not synonyms forced to coincide. A component therefore carries far richer identity than a `pid`:

```omega
data ComponentIdentity {
    code:      CodeId;        // which versioned component artifact
    state:     StateId;       // which live state lineage (survives upgrade)
    authority: PrincipalRef;  // who it acts as / what it holds
    protocol:  ProtocolId;    // the interface contract it serves
    version:   Version;       // current code+schema version
    budget:    ResourceBudget;// its scheduling/resource envelope
    upgrade:   UpgradePath;   // how it moves between versions
    quiesce:   QuiescePolicy; // how it reaches a swappable rest state
}
```

### Concurrency: tasks that share nothing

The smallest member of the family is the **task**: a single running state machine. A component runs as one or more tasks, and concurrency is many tasks making progress at once. The model has no locks, because tasks share no mutable state: ownership and borrowing forbid two tasks from holding a mutable reference to the same data, so data races are impossible by construction.

Tasks coordinate two ways:

- **Message-passing** over channels (the shared-region primitive, [[ipc_and_service_invocation]]) and **ownership transfer**, which hands data from one task to another with no copy and no sharing.
- **Owning actors** for shared mutable state: the state is owned by one task and others send it messages, so the serialization point a lock would provide is the owner's mailbox. Trivial shared cells can use lock-free atomics directly.

Tasks are **structured**: a task is owned by a scope and cannot outlive it, so a parent that exits accounts for its children instead of leaking them. Cancellation is cooperative, taking effect where a task waits, and it carries the deadline from the call that started the work ([[ipc_and_service_invocation]]). Blocking itself is parking on a condition, owned by the scheduler ([[scheduler_and_resources]]).

## Concerns & Design Space

- **Decoupling the axes.** State identity must outlive code identity (that is what makes hot swap possible — [[updates_and_hot_swap]]). Authority identity must be assignable independently of code (a component runs *as* a principal, [[capability_model]]). Crash boundary must be choosable smaller than address space.
- **Composition.** Is a component a tree, a graph, or flat? Can components nest (a service containing drivers), and does restarting a parent restart children?
- **Components as hosts.** A component can implement OS interfaces for the children it spawns, becoming their runtime: their compositor, their network, their clock, their realm. With realm-authority it runs a miniature Cathedral, sandboxing its own children with the same machinery that sandboxes it. This is the execution-level form of the recursive-provider pattern ([[capability_model]]).
- **Instance creation.** An instance is created by spawning a component with an explicit initial state and an explicitly granted capability set, so a child starts from a known, declared state and holds only what it was passed.
- **Instance lifecycle.** start / ready / running / quiescing / migrating / draining / stopped / failed — modeled as an Omega state graph the OS can inspect and schedule, not as opaque process states.
- **Crash boundary vs. restart unit.** Erlang's lesson: the thing that fails and the thing that restarts it (a supervisor) are different components on purpose ([[error_model_and_recovery]]).
- **Persistence identity.** Which components are stateless and respawnable, which own durable state, and which *are* their state (a database).
- **Authority binding.** A component holds capabilities; spawning a child grants *nothing* ambient — every authority is explicitly passed ([[capability_model]]).
- **Resource identity.** Each instance is a billable, budgetable entity, and the schedulable execution unit within it is the **task** ([[scheduler_and_resources]]); scheduling granularity need not equal isolation granularity.

## Key Questions

- What is the *minimal* component — and is there a single base concept the rest are refinements of, or are these genuinely distinct kinds?
- Does the OS isolate components by address space, by Omega's language-level isolation in one space, or a mix decided per component ([[kernel_architecture]])?
- When code identity and state identity diverge during an upgrade, who owns the bridge, and how long may old and new coexist?
- Can authority, scheduling, and persistence identity be reassigned on a live instance, or only at instantiation?

## Omega Leverage

- A component's internal lifecycle is an Omega **`machine`** with an explicit **`state`/`transition`** graph — quiescing and draining are real states, not flags ([../../../../Omega/wiki/language_guide/chapter_4_states_transitions.md](../../../../Omega/wiki/language_guide/chapter_4_states_transitions.md)).
- **`effects`** give each component a behavior ceiling; **authority flow** gives its accepts/uses/stores report — both are per-component, so the component is the natural granularity for both audits ([[capability_model]]).
- **Versioned `data` + migration** make state identity a first-class lineage that survives code replacement ([[versioned_state_and_migration]]).
- Omega does **not** yet define how one running `machine` instance is named, addressed, and supervised by another *as an OS-managed entity* — the component registry and supervision tree are Cathedral runtime structure over Omega values.

## Open Questions

- Is "component" one type with many roles, or a small zoo of related types? The whole of Part 2 leans on this answer.
- Can the crash boundary be strictly smaller than the address space without hardware isolation, relying on Omega's safety alone — and is that trusted enough for drivers?
- How does a component's identity persist across reboot and device migration without becoming an ambient, forgeable handle?

## Related
- [[capability_model]] — a component runs *as* a principal holding capabilities.
- [[scheduler_and_resources]] — the component as a scheduling/budget entity.
- [[error_model_and_recovery]] — crash boundary, restart unit, supervision.
- [[updates_and_hot_swap]] — code identity vs. state identity over upgrades.
- [[driver_model]] — the device-bound specialization of a component.
