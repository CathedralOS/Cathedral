//! Host composition supplies a binary artifact, never a Rust dependency on the distro.
mod artifacts;
mod composition;
mod inputs;
use artifacts::bundle;
use std::env;

fn main() {
    inputs::watch();
    if env::var_os("CARGO_FEATURE_BUNDLED_USER").is_none() {
        return;
    }
    if env::var_os("CARGO_FEATURE_SMOKE_TEST").is_none() {
        composition::generate();
        return;
    }
    for (variable, destination) in [
        ("CATHEDRAL_HELLO_ELF", "user.elf"),
        ("CATHEDRAL_IPC_ELF", "ipc.elf"),
        ("CATHEDRAL_MEMORY_ELF", "memory.elf"),
        ("CATHEDRAL_RENDERING_ELF", "rendering.elf"),
        ("CATHEDRAL_SUPERVISION_ELF", "supervision.elf"),
        ("CATHEDRAL_DISPLAY_SERVICE_ELF", "display-service.elf"),
        ("CATHEDRAL_DISPLAY_LAB_ELF", "display-lab.elf"),
    ] {
        println!("cargo:rerun-if-env-changed={variable}");
        bundle(variable, destination);
    }
}
