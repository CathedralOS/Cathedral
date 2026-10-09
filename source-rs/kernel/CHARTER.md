# Kernel

Owns boot/ firmware entry, core/ memory/scheduling/authority mechanisms, and arch/ CPU backends. Contracts stay at the shared source root. No dependency on distribution implementations. Pure platform hardware facts and the explicitly recorded bootstrap console are the only current platform exceptions.

See [the source layout](../../wiki/architecture/repository_layout.md).
