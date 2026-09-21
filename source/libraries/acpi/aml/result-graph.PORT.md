# Bounded result object graphs

Status: 49 pure-kernel, 27 Program-boundary and three supplemental checked
behavior/control pairs pass in the isolated
`/private/tmp/cathedral-acpi-result-graph` worktree.
The completed [ACPI-006 audit](../pipeline/resource-limits.PORT.md) records the
full bounded Program profile and its low-level API boundary.

`result_graph::check_graph` validates the reachable canonical object graph from
one allocated root ID under caller-supplied object and byte quotas. It returns
failure-first `GraphResult`, either a `ConversionFailure` or observed object count,
logical byte count and visited-ID bitmap. This is ordinary data, with no access
or execution authority. The validator accepts only a read-only ObjectStore and
never copies, allocates, repairs, resolves names or executes an object.

This is an original Cathedral resource-profile composition. The pinned Rust
interpreter has no corresponding quota algorithm. It reuses the canonical
[rust-osdev/acpi object-reference kernels](object-references.PORT.md) and
[byte storage](byte-storage.PORT.md), whose inventories retain the source hashes,
MIT OR Apache-2.0 notices and partial upstream mappings. No additional upstream
anchor is claimed complete. The [library charter](../../CHARTER.md) and
[porting policy](../../../../wiki/architecture/prior_art_and_hardware_facts.md)
apply. These limits are library policy, separate from compiler evaluator fuel,
Cathedral scheduler grants, and ACPI's method semantics.

A single pending/visited bitmap pair covers the entire traversal. Each allocated
ID is inspected once, including explicit and transparent reference carriers.
Shared children and references cannot reset the quota. The object quota is in
0..64. The byte quota is in 0..16384 and counts the sum of logical String/Buffer
lengths, once per distinct byte object's ID. Different objects sharing the same
immutable source span are charged separately. String terminators and inactive
arena bytes are excluded. These are graph quotas, not the size of a recursively
serialized package expansion.

Integer and Uninitialized payloads are structurally inert. Uninitialized Package
padding remains representable; this validation does not permit its use as an
AML operand. Lexical NameReference descriptors retain validated bounded names
and absolute declaration scopes without namespace lookup, so forward names stay
inert. All six stable-ID Reference kinds retain their identity and payload; their
edges are followed only for structural validation and accounting. No implicit
DerefOf, RefOf or Index execution is added.

Each nonempty Package uses `package_element`'s complete advertised-chain check
before its membership bitmap enters the shared work queue. Empty packages have
no edges. Repeated sibling links, dangling IDs, premature endings and extra tails
fail. Sharing and cycles between otherwise valid objects are finite graph edges
and are counted once; they do not request recursive value expansion. All graphs
still consume at most 64 object inspections and 64 bounded package-chain scans.
Methods, fields and service payloads remain explicitly UnsupportedValue in this
initial data-result profile.

String/Buffer admission validates complete backing, owner identity, source unit
and bounds, declared/initialized extent and ASCII String encoding before charging
its length. Malformed nested backing therefore cannot hide behind a valid outer
Package. Unreachable object slots and namespace entries are not admitted. Zero
object budget fails for an allocated root; zero byte budget admits graphs with
no logical byte payload. Oversized quotas/input/object counts return Capacity;
invalid roots return InvalidState; remaining reachable work returns WorkLimit;
an insufficient byte quota returns Capacity. Source errors retain canonical
Bounds, Encoding and other ConversionFailure distinctions.

The original [fixtures](../../../../tools/ports/acpi/aml/result-graph/README.md)
exercise nested and shared graphs, inert lexical references and padding, complete
package chains, malformed child storage, high-bit/MAX metadata, exact limits and
full 64-slot graphs. Controls change expected visited sets or failure kinds.
The pure-kernel receipt binds all 49 behavior/control pairs to the recorded
isolated worktree inputs. Native execution, public Rust quota equivalence and
unbounded or general-purpose resource accounting are not claimed.

## Program boundary

`program::run_program` applies the maximum supported graph quotas before returning
a successful object result. `run_program_limited` additionally accepts caller
object/byte quotas. Requests above the supported limits reject before method
execution or external-argument allocation. Integer intermediates are inline, so
Integer and void results require no allocated graph objects or byte payload.

Graph validation occurs once at the public Program boundary. Internal Return,
call argument semantics and existing copy rules remain unchanged. A failed graph
check exposes no result operand, scalar projection or has-value marker, while
retaining the execution step count and fault offset. Earlier AML writes and
private allocations stay in the Program; this is result rejection, not execution
rollback. The execution API retains its existing generic validation outcomes:
Capacity, WorkLimit and UnsupportedValue map directly; other structural/source
failures map to InvalidState. The direct GraphResult API preserves their finer
failure distinctions.

The separate [Program fixtures](../../../../tools/ports/acpi/pipeline/result-graph/README.md)
exercise these boundary semantics. All 27 behavior/control pairs pass through
the actual loader and engine, with source and runner hashes unchanged. The
receipts retain the exact isolated execution root; they are not relabeled as
canonical-checkout execution. This does not close pending bytecode work,
full deep package copying or reclamation. The subsequent ACPI-006 audit closes
the resource task for the bounded Program profile, retaining those exclusions.

Three additional Program behavior/control pairs follow RefOf and Index edges to
an extra four-byte Buffer under exact and one-short object/byte quotas. Each also
executes the same initial Program state directly through `run_method` and checks
that accepted and rejected results retain its exact `steps` and `fault_offset`.
All three pairs and their exact-input receipt verification pass. This checks
shared accounting beyond direct Package children and diagnostic preservation
without changing the original 27-pair fixture or its recorded observations.
