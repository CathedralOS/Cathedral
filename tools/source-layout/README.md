# Source layout check

Run `python tools/source-layout/check.py` from any directory. Requires Python
3.10+ and the Rust workspace toolchain. It resolves Omega package paths, inspects
Cargo's workspace graph, checks both distribution profiles and rejects forbidden
implementation dependencies. It does not compile staged Omega ports or claim
runtime isolation.

The Rust boot-to-UART edge and kernel-to-pure-hardware-facts edges are explicit
bootstrap allowances. Platform and distribution packages may import the independent `kernel/contracts`
package. They cannot import other kernel packages or lookalike paths. Public
contract packages cannot import implementations, and kernel packages cannot import
platform protocols. The legacy Omega UEFI declarations remain supported.

Run `python tools/source-layout/check_test.py` for the dependency-law regression
tests.

Platform packages cannot import kernel implementations;
kernel/platform packages cannot import distribution implementations.

Run `python tools/source-layout/format-rust.py --check` from the repository root
to check all Rust formatting; omit `--check` to format. The command runs Cargo's
formatter and explicitly formats the named bodies included by `mod.rs`, which
`cargo fmt` alone does not follow. Both use the workspace's pinned toolchain.
