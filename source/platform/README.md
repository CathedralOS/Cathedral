# Common platform in Omega

`contracts/` describes application/service boundaries. `libraries/` contains
reusable code and the temporary application host boundary. `drivers/` contains
hardware descriptions and driver work. This ownership follows the
[Rust platform](../../source-rs/platform/); none of it should depend on
the stock matrix, chooser or desktop layout.

For current desktop execution, start with [app-host](libraries/app-host/main.omg)
and its separately compiled app provider. [Service imports](contracts/imports/README.md)
own declaration decoding, bounded executable discovery and interface matching.
The Rust host still retains mappings, authenticated connections and grants.

The [Omega process runtime](libraries/user-runtime/README.md) is the path
toward removing that host. The installed chooser, Background, Colors and Notes
run on it, and it now admits child tasks and offers them services. The
[World](libraries/matrix/world/README.md) supplies a matrix's local authority:
grants, placement and attenuated input routing, with no kernel calls of its own.
The [independent Omega matrix](../distribution/lab/matrix-composition/README.md)
is the first program built from both; the stock matrix still runs its Rust host.

Many hardware packages are staged ports, not running drivers. The
[port queue](../../TASKS_OMEGA_PORTS.md) records their individual proof and runtime
scope. A directory or successful layout check is not evidence of a working driver.
