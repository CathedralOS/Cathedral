//! Host composition supplies a binary artifact, never a Rust dependency on the distro.
use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=CATHEDRAL_USER_ELF");
    if env::var_os("CARGO_FEATURE_BUNDLED_USER").is_none() {
        return;
    }
    let source = PathBuf::from(
        env::var_os("CATHEDRAL_USER_ELF")
            .expect("bundled-user requires CATHEDRAL_USER_ELF; use tools/boot-harness-rs/run.py"),
    );
    let source = source.canonicalize().expect("user ELF artifact is missing");
    println!("cargo:rerun-if-changed={}", source.display());
    let bytes = fs::read(source).expect("cannot read user ELF artifact");
    assert!(
        bytes.starts_with(b"\x7fELF") && bytes.len() <= 4 * 1024 * 1024,
        "invalid bundled ELF artifact"
    );
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("user.elf"),
        bytes,
    )
    .expect("cannot bundle user ELF");
}
