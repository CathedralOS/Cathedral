# Source layout check

Run `python tools/source-layout/check.py` from any directory. Requires Python
3.10+ and the Rust workspace toolchain. It resolves Omega package paths, inspects
Cargo's workspace graph, checks both distribution profiles and rejects forbidden
implementation dependencies. It does not compile staged Omega ports or claim
runtime isolation.

The Rust boot-to-UART edge and kernel-to-pure-hardware-facts edges are explicit
bootstrap allowances. Platform packages cannot import kernel implementations;
kernel/platform packages cannot import distribution implementations.
