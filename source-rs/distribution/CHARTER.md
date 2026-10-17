# Cathedral distribution

Owns the one built-in Cathedral experience: shell, settings, bundled applications, defaults and composition. A fork can replace this directory. There is no multi-distribution framework. profile.json selects the desktop owner, providers and installed app code. Depends on public platform packages, the public kernel contracts and foundation.
Kernel and platform never depend on this implementation. Trusted platform owners retain permission and recovery enforcement.

See [the source layout](../../wiki/architecture/repository_layout.md).

`lab/hello` is the first bundled user executable. It imports the shared
platform runtime and is compiled separately for `x86_64-unknown-none`. The host
harness selects it from `profile.json` and supplies its ELF bytes to boot;
kernel and platform have no Cargo dependency on this distribution package.

`lab/display` owns the lab's test-pattern layout and provider restart policy.
The profile also selects the independent `platform/services/display` executable;
composition chooses that provider without moving its implementation into the distro.

`desktop` is the ordinary initial userspace executable. It starts display, input,
database storage and a funded root matrix. Its isolated Programs menu requests
Notes or Colors; desktop policy admits the selected app in a nested workspace and
protects unsaved notes during replacement. Platform owners enforce capabilities,
clipping, focus and resource conservation. Follow the [composition map](README.md).

`init` and the older status/counter composition remain an explicit legacy fixture.
`user_programs` selects smoke fixtures; `startup` selects the ordinary desktop.
