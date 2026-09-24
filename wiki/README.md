# Cathedral documentation

The wiki separates what Cathedral promises from why, and from what it is still exploring. A design argument, however detailed, is not a platform promise until it is written into the specification.

| Location | Read it for | Authority |
| --- | --- | --- |
| [spec/](spec/README.md) | Accepted Cathedral contracts, organized by subject | Normative for the behavior it covers |
| [design/](design/design.md) | Goals, rationale, alternatives, and unresolved design space | Informative; does not fill gaps in the specification |
| [architecture/](architecture/repository_layout.md) | Repository structure and the enumerated trust boundary | Owns repository placement and the TCB inventory, not platform behavior |
| [boot/](boot/boot_sequence.md) | End-to-end walkthroughs of boot | Informative |
| [proposals/](proposals/README.md) | Concrete candidate changes to current contracts | Non-normative until accepted into the specification |
| [drafts/](drafts/README.md) | Temporary investigations and migration notes | Non-normative |
| [decisions/](decisions/0001-repository-layout.md) | Durable architectural decisions and their rationale | Records why; the resulting contract still belongs to its owner |
| [speculation/](speculation/future_browser.md) | Coherent long-range explorations | Not committed design |

## Sources of truth

The specification owns accepted Cathedral behavior. It may run ahead of the code: a current contract can be partly or wholly unimplemented, so every specification page reports specification coverage and implementation coverage separately.

[`source/contracts/`](../source/contracts/CHARTER.md) owns machine-readable interface and foreign-ABI shapes. The specification owns the meaning and invariants attached to those shapes and does not repeat their encoding. The two must agree. A mismatch is a defect, not a precedence rule.

Code and its charters report what is implemented. Design chapters explain where the system is heading, but a rule the specification omits cannot be inferred from design prose. A proposal or an owner question changes nothing until the accepted answer is written into the specification and any machine-readable contract it touches.

## Missing contracts

The [specification index](spec/README.md) owns the coverage map, including subjects not yet written. Do not create empty specification pages. Create one when at least one accepted rule exists, mark its coverage, and state what the contract does not yet establish.

Questions that need an owner-level semantic or trust decision go in [`OWNER_QUESTIONS.md`](../OWNER_QUESTIONS.md). Extraction work, implementation work, and deferred research do not.

## How these docs are written

Every page is written so that a reader who stops early still has the gist, and one who keeps going gets the detail.

- **Lead with the claim.** The first sentence of a page or a section says what the thing is or does. Detail follows in decreasing order of importance.
- **One idea per sentence.** Prefer a period to an em-dash, a semicolon, or a parenthetical. An em-dash marks a real aside, not a joint between two clauses.
- **Bold a term once, where it is defined.** Never bold for emphasis. Italics for emphasis, and rarely.
- **State the design, not its history.** Write the mechanism as it is. "Decided", "settled", "rejected", "now holds", and "previously" belong in commit messages and the [gap register](design/gap_register.md), not in chapter prose. An alternative is stated as an alternative: "We do not X because Y."
- **Cut the filler adverbs.** Honestly, explicitly, deliberately, genuinely, precisely, simply, first-class, orthogonal. Delete the word, or replace it with the fact it was standing in for.
- **End when the content ends.** No closing one-liner that restates the paragraph, and no "So:" or "The point is" summary. If a paragraph needed a summary, lead with it.
- **Structure over walls.** A table for a comparison or a status. A diagram for a flow or a nesting. A list for parallel items. Prose for an argument.
- **Real words.** Capability, principal, quiescence, attenuation. No abbreviations only an insider would recognize.
- **Name the legacy contract before the replacement.** Cathedral's value is the delta from what exists.
- **Omega sketches over prose, and only real Omega.** Before naming a keyword or construct, confirm it exists in `../Omega/wiki`. Syntax may be provisional; the obligation it expresses is the point.
- **Cross-link by slug.** `[[capability_model]]`, not `[[00_capability_model]]`, so links survive renumbering.

Each directory has its own register. The specification states rules and does not persuade. Design chapters explain and argue. Boot pages walk a sequence in order. Index pages are maps.
