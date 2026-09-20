# Cathedral documentation

Cathedral separates current contracts from their rationale, explanations, and
possible future changes. The separation matters: a detailed design argument is
not automatically a platform promise.

| Location | Read it for | Authority |
| --- | --- | --- |
| [spec/](spec/README.md) | Accepted Cathedral contracts, organized by subject | Normative for the behavior it covers |
| [design/](design/design.md) | Goals, rationale, alternatives, and unresolved design space | Informative; does not fill gaps in the specification |
| [architecture/](architecture/repository_layout.md) | Repository structure and the enumerated trust boundary | Owns repository placement and TCB inventory, not platform behavior |
| [boot/](boot/boot_sequence.md) | End-to-end explanations of boot | Informative walkthroughs |
| [proposals/](proposals/README.md) | Concrete candidate changes to current contracts | Non-normative until accepted into the specification |
| [drafts/](drafts/README.md) | Temporary investigations and migration notes | Non-normative |
| [decisions/](decisions/0001-repository-layout.md) | Durable architectural decisions and their rationale | Records why a decision was made; the resulting contract still belongs to its owner |
| [speculation/](speculation/future_browser.md) | Coherent long-range explorations | Explicitly not committed design |

## Sources of truth

The subject-organized specification owns accepted Cathedral semantics. It may
lead implementation: a current contract can be partly or wholly unimplemented.
Every specification therefore reports specification coverage and implementation
coverage separately.

[`source/contracts/`](../source/contracts/CHARTER.md) owns machine-readable
interface and foreign-ABI shapes. For a property represented there, its exact
encoding is not duplicated in prose. The specification owns the Cathedral
meaning and invariants attached to those shapes. The two must agree; a mismatch
is a defect rather than a precedence escape hatch.

Code and code-adjacent charters report what is implemented. Design chapters
explain why the system is heading in a direction, but an omitted specification
rule cannot be inferred from design prose. A proposal or owner question changes
nothing until the accepted answer is written into the specification and any
machine-readable contract it affects.

## Missing contracts

The [specification index](spec/README.md) owns the coverage map, including
subjects that are not yet written. Do not create empty specification pages.
Create a page once at least one accepted rule exists, mark its coverage honestly,
and state what the current contract does not yet establish.

Questions requiring an owner-level semantic or trust decision belong in
[`OWNER_QUESTIONS.md`](../OWNER_QUESTIONS.md). Extraction work, implementation
work, and deliberately deferred research do not.
