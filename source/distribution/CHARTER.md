# Cathedral distribution

Owns the one built-in Cathedral experience: shell, settings, bundled applications, defaults and composition. A fork can replace this directory. There is no multi-distribution framework. profile.json is consumed by the boot harness today; desktop packages arrive when implemented. Depends on public platform packages and shared contracts/foundation. Kernel and platform never depend on this implementation. Trusted platform owners retain permission and recovery enforcement.

See [the source layout](../../wiki/architecture/repository_layout.md).

Update chrome requests authorized installation and reports installed artifacts,
changed launch references and notification outcomes. It does not require every
running app to report live adoption. Matrices own cooperative restart, prompts
and deferral within their lifecycle authority; stopping a matrix still tears down
its owned subtree. See [the update contract](../../wiki/spec/execution/updates.md).
