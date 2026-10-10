# Cathedral distribution

Owns the one built-in Cathedral experience: shell, settings, bundled applications, defaults and composition. A fork can replace this directory. There is no multi-distribution framework. profile.json is consumed by the boot harness today; desktop packages arrive when implemented. Depends on public platform packages and shared contracts/foundation. Kernel and platform never depend on this implementation. Trusted platform owners retain permission and recovery enforcement.

See [the source layout](../../wiki/architecture/repository_layout.md).

`programs/hello` is the first bundled user executable. It imports the shared
platform runtime and is compiled separately for `x86_64-unknown-none`. The host
harness selects it from `profile.json` and supplies its ELF bytes to boot;
kernel and platform have no Cargo dependency on this distribution package.

`programs/display` owns the lab's test-pattern layout and provider restart policy.
The profile also selects the independent `platform/services/display` executable;
composition chooses that provider without moving its implementation into the distro.

`init` is the ordinary initial userspace executable. It receives bounded grants,
launches independent display/input providers and `applications/status`, and owns
restart decisions. The application draws through `libraries/boot-scene`, owns
navigation/toggle state and reconnects after provider replacement. Init and the
providers survive application failure. `user_programs` in the profile selects
smoke fixtures; `startup` selects ordinary composition and peer links.
