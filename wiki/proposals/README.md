# Proposals

Concrete candidate changes to Cathedral contracts belong here. A proposal does
not define current behavior.

Use `NNNN_short_descriptive_name.md`, beginning with `0000`. A proposal names
its status, affected specification owners, concrete requirement, proposed
behavior, compatibility and trust consequences, viable alternatives, unresolved
questions, and acceptance evidence.

IDs are stable and are not reused after removal. The next available ID is
`0002`. A substantial proposal may have a same-named companion directory. Its
entry page maps the responsibilities; companion pages do not become independent
contracts or competing sources of truth.

Acceptance requires an owner decision. The accepting change updates
the specification, machine-readable contracts, implementation work, and links
that the decision affects. Once incorporated, remove the proposal; Git retains
its history.

Temporary investigations belong in [drafts](../drafts/README.md). A narrow
semantic or trust choice blocking a concrete requirement belongs in
[`OWNER_QUESTIONS.md`](../../OWNER_QUESTIONS.md).

## Open proposals

- [0000: Rendering, composition and resource custody](0000_rendering_and_composition.md)
  separates retained scene submission, app-funded backing, read leases, resource
  budgets and software/GPU presentation. Platform text placement, residency policy
  and direct-write admission remain proposed.
- [0001: Input sources and scoped pointer leases](0001_input_and_pointer_leases.md)
  separates source bindings, logical pointers, cursor visuals and action rights.
  Pointer-to-seat association and keyboard-focus authority remain proposed.
