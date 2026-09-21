# Encryption-profile recursive constructor observations

Status: tested with all 610 checked-interpreter observations and ten changed-body
controls; disabled and repeated-profile const/control pairs also pass. This pure component reuses
`mapper_topology::RecursiveObservation` and its ordered constructor checks with
an explicit copyable `memory_encryption::State`. No recursive page-table object,
reference or ownership token is constructed.

Modified observations derive from `RecursivePageTable::new` at x86_64 revision
`cc35c876d3badb57df54a66e22f7768a52be95f2`, recursive_page_table.rs:57. The full
constructor documentation/body, PhysAddr/PhysFrame admission, PTE frame decoding,
existing topology helpers and encryption state contract were consulted. Exact
[MIT OR Apache-2.0 notices](../../../licenses/rust-osdev/x86_64/) are retained.

The input is a virtual table address, a previously observed CR3 **frame address**,
and the selected recursive entry word. Raw CR3 word extraction has separate
[register-expression evidence](encrypted-registers.PORT.md). This helper reads
no register or table. First it checks canonical virtual geometry and equal four
page-table indices, before admitting the observed frame. Misaligned, out-of-range
or current-encryption-bit-bearing observed frames fail before entry classification.
A not-present entry takes priority over its HUGE bit; otherwise HUGE fails, then
profile-decoded entry address must match the observed frame.

The profile's latest bit constrains observed physical inputs. Its accumulated PTE
address mask independently controls entry decoding. Removing only excluded default
address positions permits reuse of the original observation helper without
changing PRESENT/HUGE flags or geometry. Polarity does not alter these operations.
Only profiles obtained through initial/configure fall within pinned equivalence;
forged masks and arbitrary profile invariants are outside that contract.

The Rust witness copies the complete ordered check portion of `new`. Pointer
acquisition is replaced by a supplied VirtAddr, the CR3 read by strict observed
PhysFrame construction at the same expression position, selected table access
by a supplied actual PTE, and final Self by its numeric recursive index. It calls
actual public PhysAddr, PhysFrame, Page and PTE operations but **does not call the
public RecursivePageTable constructor**. Generation binds every substitution to
the pinned body. Panics in strict physical-frame construction become the existing
InvalidObservedFrame case. Virtual-address rejection is observed separately.

Ten fresh processes configure before creating physical/PTE data. Their 610
observations cover recursive indices 0/255/256/511, low/high halves, offset within
the containing page, nonrecursive and noncanonical addresses, frame mismatch,
PRESENT/HUGE priority, invalid observed addresses before entry failures, both
polarities, low/high encryption positions and repeated bit47/48/47 configuration.
Index 511 and nonzero page offsets remain inert numeric scenarios; they establish
no valid actual Rust reference or recursive alias. No LA57, hardware encryption,
CPU root, native Omega layout or publication/flush capability is inferred.

The canonical default-profile helper remains unchanged. Tests execute the actual
composition and mutate computed frame equality under unchanged success contracts.
The tools records separate checked-interpreter execution from representative
constant evaluation; neither is native execution.

The complete compact checked run took 16.681 seconds, with a maximum 58,684
evaluator fuel units per profile. All 61 rows per profile execute through one
bounded loop, retaining the same observations as expanded fixtures. The selected
Cathedral source/harness/runner hashes are unchanged before and after each run.
See `tools/ports/x86_64-encrypted-constructor/verify_record.py` for retained checks.
Compiler revision is `eaa7993a23623cd8fabf45350340479c5c9c7879`, binary SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
