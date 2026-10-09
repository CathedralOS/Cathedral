//! Host composition supplies a binary artifact, never a Rust dependency on the distro.
use std::{env, fs, path::PathBuf};

fn main() {
    for (variable, destination) in [
        ("CATHEDRAL_HELLO_ELF", "user.elf"),
        ("CATHEDRAL_IPC_ELF", "ipc.elf"),
        ("CATHEDRAL_SUPERVISION_ELF", "supervision.elf"),
    ] {
        println!("cargo:rerun-if-env-changed={variable}");
        if env::var_os("CARGO_FEATURE_BUNDLED_USER").is_none() {
            continue;
        }
        bundle(variable, destination);
    }
}
fn bundle(variable: &str, destination: &str) {
    let source = PathBuf::from(env::var_os(variable).unwrap_or_else(|| {
        panic!("bundled-user requires {variable}; use tools/boot-harness-rs/run.py")
    }));
    let source = source.canonicalize().expect("user ELF artifact is missing");
    println!("cargo:rerun-if-changed={}", source.display());
    let bytes = fs::read(source).expect("cannot read user ELF artifact");
    assert!(
        bytes.starts_with(b"\x7fELF") && bytes.len() <= 4 * 1024 * 1024,
        "invalid bundled ELF artifact"
    );
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join(destination),
        bytes,
    )
    .expect("cannot bundle user ELF");
}
