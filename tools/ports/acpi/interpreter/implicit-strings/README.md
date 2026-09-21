# Primary String formatting checks

`python3 tools/ports/acpi/interpreter/implicit-strings/check.py` executes 334 Omega
behavior/control pairs in independent batches, then three constant pairs.
`--verify-record` checks exact used source/tool and compiler/runner hashes plus
every retained result. `--write-cases` regenerates the primary-rule vectors.

Integer expectations use Python's fixed-width uppercase hexadecimal formatting;
Buffer expectations join two-digit byte strings with spaces. These are primary
specification oracles, not Rust observations or copied private helper bodies.
Every successful Omega result is compared across all 256 initialized bytes,
including output tails. See the
[port record](../../../../../source/libraries/acpi/interpreter/implicit-strings.PORT.md)
for capacity limits and deferred integration.
