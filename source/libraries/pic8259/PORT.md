# pic8259 reconciliation and pure protocol recipes

## Scope and status

PIC-000 inventories the complete pinned `src/lib.rs` against existing Cathedral
facts, port sequences, and acknowledgement contracts. Pure missing behavior is
translated separately from live I/O. Status: **tested** for the pure fact,
validation and protocol-recipe slice; all upstream declarations reconciled,
with live Rust wrappers deliberately omitted. Existing core PIC adapter passes
isolated source checking. No hardware or whole-production-root claim.
Reviewed 2026-09-20. The canonical runner also passes on a fresh build of Omega
`eaa7993a23623cd8fabf45350340479c5c9c7879`, binary SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
No ordinary implementation work remains in this bounded audit.

## Upstream pin and licensing

[pic8259 0.11.0](https://github.com/rust-osdev/pic8259) at
`136052bcbf081b9382fc3d72ba04b83b30f664b4`, local
`reference_code/rust-osdev/pic8259`; MIT OR Apache-2.0. Translation headers identify
modifications. Preserved [notices](../../../licenses/rust-osdev/pic8259/UPSTREAM-NOTICES.md),
[MIT](../../../licenses/rust-osdev/pic8259/LICENSE-MIT), and
[Apache](../../../licenses/rust-osdev/pic8259/LICENSE-APACHE) accompany the
[root entry](../../../THIRD_PARTY_NOTICES.md). Existing Cathedral facts/core
remain independently authored; new algorithm/recipe work retains upstream provenance.

## Source and public-symbol map

[inventory.json](inventory.json) binds the complete pinned `src/lib.rs`: 17 lexical
declaration anchors, 15 translated and two deliberate wrapper omissions; no
pending or language-blocked anchors. Entire-file hash binds private fields too.

| Upstream declaration | Cathedral mapping / disposition |
| --- | --- |
| CMD_INIT, CMD_END_OF_INTERRUPT, MODE_8086 | Existing facts ICW1_PC_CASCADE, OCW2_END_OF_INTERRUPT, ICW4_8086_MODE. |
| Pic.offset, ChainedPics.pics offsets | PicOffsets stores only the two inert bases. |
| Pic.command/data, Pic and ChainedPics Port wrappers | Deliberately omitted live Rust ownership wrappers; existing core Pic8259 retains explicit PortIo. |
| ChainedPics::new | PicOffsets construction plus separately callable pic_offsets_valid; PC ports remain existing facts. |
| ChainedPics::new_contiguous | pic_contiguous_offsets with alignment/nonoverflow requirement and pic_contiguous_valid predicate. |
| Pic::handles_interrupt, ChainedPics::handles_interrupt | pic_handles_interrupt and pic_chain_handles_interrupt, widened exclusive upper bound. |
| ChainedPics::initialize | pic_initialization_plan preserves ten writes, saved masks and eight delay requirements as data. Existing remap_masked is the narrower live policy. |
| Pic::read_mask, ChainedPics::read_masks | pic_read_masks_plan names master/slave read ports; values still require authorized reads. |
| Pic::write_mask, ChainedPics::write_masks, disable | pic_write_masks_plan, pic_disable_plan; no writes executed. |
| Pic::end_of_interrupt, ChainedPics::notify_end_of_interrupt | pic_eoi_plan preserves slave-before-master routing, master-only and unrelated-vector cases; existing complete_timer_acknowledgement handles admitted IRQ0. |

New records have no asserted native layout. Existing facts own every reused
command/mask/vector constant; no competing controller provider is introduced.

## Primary specifications and representation vectors

Intel 8259A datasheet, December 1988, order 231468-003:
[original Intel publication](https://www.pcjs.org/documents/datasheets/intel/INTEL_8259A_PIC.pdf),
pages 10–14 (initialization, vector bits, masks, EOI), page 19 (cascade).
Port addresses cross-check pinned PC wiring. Generic recipes describe numeric
operations and establish no native ABI layout or port access. Primary vector
placement requires eight-vector alignment; pure membership uses widened
arithmetic to avoid the upstream u8 upper-bound overflow.
[vectors.json](vectors.json) records 22 numeric expectations in the shared vector
format. They are reviewed source/spec fixtures, not an Omega memory-layout
measurement. No bit-width-dependent aggregate ABI is claimed.

## Translated tests and fixtures

Upstream has no tests in its tracked Rust source. Locally authored Omega
[tests](../../../tools/ports/pic8259/main.omg) execute all pure helper bodies
through compile-time semantic evaluation: first/last/outside vectors, top block
248–255, malformed offsets, overlapping/unaligned ranges, contiguous upper
bound, both mask extremes, distinct saved masks, exact ten initialization writes
and eight delay requirements, master/slave/unrelated EOI routing and order.
The [negative fixture](../../../tools/ports/pic8259/negative.omg) changes the
expected first initialization command from 0x11 to 0x12 inside the test while
keeping the final assertion unchanged. It evaluates failure 1 and rejects
`require_success` with `1 == 0`; positive tests evaluate success 0.

[check_upstream.py](../../../tools/ports/pic8259/check_upstream.py) compiles the
exact pinned controller implementation after changing crate framing and
substituting recording Port methods. It verifies mask reads occur before init,
all writes/delays/restores, read/write/disable and 1024 membership/EOI routes
across four offset pairs. It executes upstream Rust only, without privileged
instructions; this evidence is separate from the translated Omega tests.

[check_existing.py](../../../tools/ports/pic8259/check_existing.py) source-checks
the existing core PIC file byte for byte with real facts and modern isolated
package metadata. A separate source regression check pins ordered port writes,
explicit PortIo reach, Pending input and completion after EOI. This is not
compiler-artifact validation. The older initialization/root artifact canaries
currently stop before compilation because their package symlink escapes the
package source root; they do not establish a compiler feature blocker.

## Omega blockers and boundary seams

No language blocker identified for this pure scope. Actual I/O requires the
existing explicit PortIo service and provider ownership; saved mask reads,
legacy delay writes, remapping, and interrupt acknowledgement are not implied
by possessing a recipe. A recipe neither mints nor settles a linear interrupt
acknowledgement. Generalizing live provider lifecycle remains outside PIC-000.

## Deliberate deviations

Existing `Pic8259::remap_masked` writes all masks rather than restoring previous
masks; `unmask_timer` admits only IRQ0. Existing core omits upstream's eight
port-0x80 delay writes. These are explicit first-QEMU-provider choices, not a
claim of equivalent initialization timing on legacy physical hardware.
For valid blocks ending at vector255, widening fixes upstream `offset + 8`
overflow (panic with overflow checks or wraparound otherwise). Malformed
bases249–255 are rejected by membership, not truncated. Separate validators
reject non-eight-aligned/overlapping ranges; generic membership intentionally
retains upstream interval semantics for lower unaligned bases. Contiguous
construction requires aligned primary<=240. Zero-based ranges are legitimate
generic geometry; Cathedral's architectural exception exclusion remains provider
policy. `pic_initialization_plan` is an unchecked inert recipe and does not
implicitly validate/admit offsets.

Upstream EOI routing accepts a vector alone; pure routing retains that decision,
while live Cathedral timer acknowledgement additionally consumes Pending
InterruptAcknowledgement and settles it only after the master EOI. General
slave routing stays inert. Neither upstream nor this port handles spurious
IRQ7/IRQ15 by querying ISR; the generic recipe must not be used to infer that
such an interrupt is real.

## Cathedral integration and authority

[Library charter](../CHARTER.md) owns pure helpers; existing
[hardware facts](../../drivers/facts/pic_8259.omg) remain zero-authority data.
[Core adapter](../../core/pic_8259.omg) retains its exact PortIo reach and Pending
acknowledgement lifecycle. New package has no production dependency. `source/drivers/facts/build.omg` now
names `builder: &mut Build` and publishes `cathedral-hardware-facts`, the minimal
package-front-door modernization needed to import the existing facts. It grants
no service. The production core build/adapter is unchanged.

## Verification commands and results

Run from Cathedral root. Host aarch64-apple-darwin, Rust
`1.94.0-nightly (f6a07efc8 2026-01-16)`, the identified Omega release binary
above. `check.py` accepts an
optional explicit compiler path as its sole positional argument. The fresh-build
run used `/tmp/cathedral-omega-eaa7993/release/omega`; the default sibling release
binary may be an earlier local build.

| Command | Result and limits |
| --- | --- |
| `python3 tools/ports/pic8259/check.py` | Complete canonical runner passes: inventory, vector format, recorded upstream, translated positive/negative semantic tests, exact existing adapter source. |
| `python3 tools/ports/inventory.py check source/libraries/pic8259/inventory.json --checkout reference_code/rust-osdev/pic8259` | One file, 15 translated anchors, 2 deliberate wrapper omissions, 0 pending/blocked. |
| `python3 tools/ports/vectors.py source/libraries/pic8259/vectors.json` | 22 expected numeric fixtures valid; not a measured Omega layout. |
| `python3 tools/ports/pic8259/check_upstream.py` | Upstream recording-port tests pass; 1024 vector routes plus initialization/masks. |
| `../Omega/target/release/omega --check source/libraries/pic8259/plans.omg` | Pass 11 sources. |
| `../Omega/target/release/omega --check tools/ports/pic8259/main.omg` | Pass 12 sources; real pure helper semantic execution. |
| `../Omega/target/release/omega --check tools/ports/pic8259/negative.omg` | Expected rejection: altered real test condition computes1, final contract cannot prove `1 == 0`. |
| `python3 tools/ports/pic8259/check_existing.py` | Source regression and byte-identical core PIC check pass 15 sources; no I/O. |
| `OMEGA_BIN=../Omega/target/release/omega tools/pic-8259-initialization-contract-canary/run.sh` | Harness stops: source symlink `project/core` resolves outside package root. |
| `OMEGA_BIN=../Omega/target/release/omega tools/legacy-timer-root-canary/run.sh` | Same pre-compilation harness symlink error. Full root contract closure not newly revalidated. |
| Hardware/simulator/production image | Not run; no live provider change or new production reach. |

No language blocker was found in the claimed pure translation. Existing canary
harness modernization and whole-root integration are separate follow-up work,
not requirements silently waived for a firmware/hardware compatibility claim.
