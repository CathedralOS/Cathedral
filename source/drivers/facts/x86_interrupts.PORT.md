# x86 interrupt values and byte codecs

## Scope and status

Pure interrupt values implemented and source checked: canonical gate encoding,
EntryOptions operations, saved-frame values, page-fault flags, exception/index
classification, range checks, and detached256-entry table default/reset/access
and byte conversion. Fixed-sized hardware bytes remain separate from semantic
alternatives (`StackIndex` and explicit result cases). The existing canonical
`X86IdtGate`, selector/flags wrappers, exception facts and IST policy are reused.

Omega body execution passes, including separate complete4096-byte table encode
and decode fixtures. A combined fixture exceeds the compiler's100,000-step
constant-initializer budget; splitting tests is an ordinary harness solution.
No native Omega ABI or CPU integration claim is made.

## Upstream pin and licensing

x86_64 0.15.5 `cc35c876d3badb57df54a66e22f7768a52be95f2`, MIT OR Apache-2.0.
[License and original IDT copyright notice](../../../licenses/rust-osdev/x86_64/SOURCE-NOTICES.md).
Derived source preserves the upstream Philipp Oppermann notice.

## Source and public-symbol map

[x86_interrupts-inventory.json](x86_interrupts-inventory.json) audits the complete
pinned `src/structures/idt.rs`:158 lexical anchors,104 translated and54 omitted,
plus explicit private-representation mappings. These are source dispositions,
not a count of independently tested APIs. Named IDT fields map to numeric array
slots; typed Rust borrowed indexing is not claimed. Selector-error-code behavior
reuses the completed descriptor slice. Existing exception delivery policy stays
separate from the upstream enum's23 numeric identities and indexing rules.

Handler calling conventions, live CS reads, pointer/lifetime management,
volatile saved-frame mutation, handler macro generation, `lidt` and `iretq`
remain explicit integration boundaries. Numeric gate construction takes its
selector as input and grants no handler authority.

## Primary specifications and representation vectors

[Intel SDM Volume3A](https://cdrdv2-public.intel.com/835754/253668-sdm-vol-3a.pdf)
and [AMD APM Volume2 revision3.44](https://docs.amd.com/v/u/en-US/24593_3.44_APM_Vol2)
provide the architectural gate, saved-frame and page-fault context. The exact
vectors here are independently measured from the pinned Rust implementation,
not presented as manual quotations or universal feature availability.

[x86_interrupts.vectors.json](x86_interrupts.vectors.json) contains73 actual Rust
host observations, each also checked by a UEFI-x64 compile-time assertion:
10 page-fault flags,23 exception numbers, size/alignment of six types, five
public frame offsets and23 public IDT field offsets. Private offsets are not
fabricated. Rust Entry is16 bytes/alignment4; Cathedral's existing gate policy
selects16 bytes/alignment16. EntryOptions is4/alignment2, raw frame40/alignment8,
and upstream IDT4096/alignment16. DetachedIdt is semantic storage; its explicit
codec produces4096 bytes without asserting a native table layout.

## Translated tests and fixtures

Four Rust tests execute actual upstream constructors/defaults/index/range
behavior. EntryOptions' private numeric methods are extracted verbatim except
for the mirror owner name; safe public setters are also exercised on the actual
type. Test-only raw byte observations never install or invoke a handler.

Omega's positive semantic fixture executes112 option combinations, all256
exception/index classifications,23 reused canonical exception identities,
4/40-byte record codecs including reserved bytes, exact16-byte gate fragments,
reserved-bit rejection, canonical48-bit address projection and range results.
A second fixture copies the exact production codecs (only module/import names
change), exposing private scan bodies to a test bridge. It executes the final
three table slots and confirms success, vector255 IST rejection and vector254
reserved-tail rejection latched through the next iteration. Separate full256-entry encode and valid-decode fixtures also execute the actual
public implementation. The combined encode-plus-two-decodes fixture remains
retained as a diagnostic probe; its budget limit does not block the port.

Six expected-value body mutations require the evaluated result to become1,
covering a gate byte, range-result case, saved-frame byte, bounded table error
vector, full-table final output byte and full-table final error vector.
The final `requires value == 0` assertion itself remains unchanged.

## Omega blockers and boundary seams

On Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`, the combined4096-byte fixture
fails with `constant initializer: step budget exceeded`. Omega sets a fixed
100,000-step const budget independently of its runtime interpreter budget.
Separate complete-table encode/decode fixtures and smaller exact-body scans
pass. This is a resolved harness-sizing issue, not a port/design blocker.

The canonical gate's layout was separated into `x86_idt_gate_layout.omg` to keep
layout accessor generation out of pure semantic imports. All original fields,
constants and seven placements are source-audited against commit
`d4b8fa5aae189e9ee10768a1e6de5c1370fb5dcb`. The modern policy uses a named Layout
witness and local entries array. Real imported consumers reproduce
`selects private data` for generated gate/frame fields. Equivalent local
schemas consume the actual plans successfully. This establishes source-level
plan normalization and access, not observed native bytes.

Runtime byte indices use computed locals with bounded u16 domains. Measured
scan recursion keeps the decreasing transition in the machine entry. These
ordinary source forms pass current proof checking; no new language requirement
is inferred from earlier rejected expression forms.

## Deliberate deviations

Raw saved-frame addresses and flags preserve bits without asserting canonical
addresses, active selectors or return-state validity. Gate parsing retains the
existing zero reserved-tail/domain constraints and returns explicit errors for
incompatible bytes. A raw EntryOptions value preserves all16 bits; conversion
to the constrained canonical gate checks reserved IST bits. Software IST indices
are0..6, hardware encodings1..7, and the current-stack alternative is a distinct
case. The complete Rust handler API, generic type markers and formatting traits
are deliberately outside this detached value layer.

## Cathedral integration and authority

No production core or PTE changes. The canonical legacy gate import surface
remains; its separate layout module is added to the isolated layout canary.
That harness now uses current dependency syntax and actual source/consumer
checks. Retired JSON dumps and the nonexistent omega-cli package are no longer
requirements; the historical jq script is not claimed to pass.

## Verification commands and results

```sh
python3 tools/ports/x86_64-interrupts/check.py --omega /tmp/cathedral-omega-eaa7993/release/omega
OMEGA_BIN=/tmp/cathedral-omega-eaa7993/release/omega tools/x86-idt-gate-layout-canary/run.sh
```

Compiler SHA-256:
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
Producer17 sources, main semantic19, detached storage17, private scan18, separate
full-table fixtures19 each, local layout14 and legacy canary14 pass. The runner
also checks source freshness, pinned inventory,73 Rust observations/target
assertions, four Rust tests, six body controls and exact expected diagnostics.
Expected-diagnostic probes are not counted as successful compilation.
