# Native app requests

[main.omg](main.omg) names operations and constructs packets for the temporary
[Rust Omega app host](../../../../source-rs/platform/libraries/omega-app/README.md).
This package supplies no services or ambient authority; each request uses the
connections granted to that process by its immediate matrix.
