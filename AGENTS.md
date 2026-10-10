# Working in Cathedral

Preserve the discoverability architecture described in
[the architecture guide](wiki/architecture/discoverability.md). Entry points
show real sequencing or dispatch at one level of detail. Descending into a
subsystem reveals its next decisions. Split by responsibility, keep small
leaves together, and avoid generic dumping grounds or forwarding-only layers.

The [repository layout](wiki/architecture/repository_layout.md) defines ownership
and dependency direction. Keep the kernel, common platform and single replaceable
distribution distinct. Shared protocols describe boundaries; they do not import
their implementations. Maintain explicit navigation across executable/IPC seams.

Use existing tests appropriate to changed behavior. Rust host policy tests run
with `cargo test --locked` inside `source-rs/`; target-specific checks and QEMU
commands are documented in [the Rust lab guide](source-rs/README.md). Run
`python tools/source-layout/check.py` when moving packages or changing composition.

Record unresolved owner-level semantic or trust choices in
[OWNER_QUESTIONS.md](OWNER_QUESTIONS.md). Ordinary refactoring and documentation
corrections do not require new owner decisions.
