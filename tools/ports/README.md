# Port provenance workflow

`tools/ports/` owns host-side support for the
[rust-osdev queue](../../TASKS_RUST_OSDEV_PORTS.md). These tools never ship to
Cathedral. The [porting policy](../../wiki/architecture/prior_art_and_hardware_facts.md)
owns provenance and authority rules; source packages retain their usual owners.

Start a port by copying [PORT.template.md](PORT.template.md) to `PORT.md` beside
the destination package. Define the exact upstream slice and inventory its
files/public symbols before claiming a translation. Use the queue's full commit
pin, review the license and notices at that revision, retain the applicable
texts, and link the root `THIRD_PARTY_NOTICES.md` entry. Record source mapping,
modified files, specification citations, deviations, tests, and authority seams.

The optional pinned checkout belongs in gitignored
`reference_code/rust-osdev/<project>/`. It is reading and host-verification
input, never a committed Rust vendor tree or Cathedral dependency. Checks that
require it must fail clearly when it is absent. A pin update is a separate
reviewed change: compare source/symbol coverage and licensing, refresh mappings
and vectors, rerun available checks, and record any new blockers.

A port advances through `inventoried`, `transcribed`, `typechecked`, `tested`,
and `integrated` only with the evidence defined in the template. Record mixed
slice states rather than applying a small test's success to the whole package.
Fixtures awaiting an Omega consumer are unexecuted expectations. Host checks,
upstream Rust tests, Omega tests, hardware execution, and Cathedral integration
remain separate evidence. Untypeable packages stay out of production build
roots. Update the task checkbox and `PORT.md` alongside each completed slice.

Use `PORT-BLOCKED[omega:<short-name>]: <required behavior>` at unresolved source
seams. Document an actual language implementation gap or unsettled design with
current Omega specification/implementation evidence. Ordinary port engineering
remains work to complete; missing local setup does not by itself establish a
language blocker.

## Inventory checks

Requires Python 3.9+ and Git; the same scripts work on macOS and Windows
(use `python` instead of `python3` where appropriate). No Python packages are
required. Run from the repository root:

```sh
python3 tools/ports/inventory.py check tools/ports/fixtures/uefi-header.inventory.json --checkout reference_code/rust-osdev/uefi-rs --require-transcribed
python3 -m unittest discover -s tools/ports -p 'test_*.py'
```

`inventory.py snapshot --checkout PATH --revision FULL_SHA --url URL ROOT...`
prints a deterministic `cathedral-port-inventory-v1` JSON candidate. Review it
before saving it beside the port. Each `upstream.roots` entry names a tracked
file or directory; all Rust files below it must appear in `files`. The tool
requires the checkout HEAD and source bytes to equal the full pin, checks every
SHA-256, and rebuilds a conservative lexical symbol index. This includes named
Rust declarations (including private methods for deliberate review), public
fields, reexports, and assigned enum/flag members. It does not expand macros,
resolve reexports, interpret cfgs, or replace the human public-API inventory.
Full source hashes bind unsupported syntax too; reviewers must map generated
APIs and implicit traits explicitly in `PORT.md`. A hash alone is not evidence
that a symbol was translated.

Each file and indexed symbol has `disposition` (`pending`, `translated`,
`omitted`, or `blocked`). Untranslated entries require a specific `reason`.
Translated entries require `targets`, each with a repository-relative `path`
and exact source `anchor`; missing files/anchors fail. An anchor is a navigation
check, not semantic equivalence. `--require-transcribed` rejects pending and
blocked entries; ordinary checks report their counts so an inventoried or
partially transcribed package can be checked honestly. Omission reasons remain
review obligations, never automatic permission to omit required behavior.

The checked-in header fixture exercises the existing Cathedral `EfiTableHeader`
source mapping, including its different field names. It proves neither complete
UEFI coverage nor compiled ABI agreement.

## Layout and value vectors

`cathedral-port-vectors-v1` JSON retains an exact `target` object (`abi`,
`pointer_bits`, `endian`), `provenance` (`kind`, `description`, `sources`), and
nonempty named `measurements`. Every measurement has a `kind` and `value`:

- `size`, `alignment`, `offset`: nonnegative integer byte counts; alignment is
  a positive power of two.
- `value`: an exact JSON integer (including signed values), never a float.
- `bytes`: lowercase hex pairs in address order, suitable for GUID/fixture bytes.

Names identify the source type/member or constant, for example
`EfiTableHeader.crc32.offset`. Use sorted JSON object keys and a final newline
when writing checked-in vectors. Endianness and pointer width must be explicit;
never compare unrelated targets or substitute host pointer width implicitly.

```sh
python3 tools/ports/vectors.py tools/ports/fixtures/uefi-header.vectors.json
python3 tools/ports/vectors.py expected.json --observed omega-observed.json
```

The first command validates expectations and explicitly reports **Omega
comparison NOT RUN**. The second requires identical target and measurement
coverage, kinds, and values. Observation provenance must say `omega-inspection`
and identify `compiler_revision` (full Git SHA) and `artifact_sha256`. Those
identifiers aid review; the tool does not authenticate an observation. The
independent producer must inspect actual Omega layout/output and retain its
command/artifact. Copying expectations into an observation is not verification.

**Current state:** expected vectors and comparator available; a current Omega
inspection adapter is not implemented. Omega no longer emits the old optional
JSON dumps. No placeholder adapter fabricates observed values. A future adapter
must consume actual supported compiler inspection and bind its artifact before
this comparison can count as Omega ABI evidence. This limitation does not block
landing reviewed expectations under PORT-004.
