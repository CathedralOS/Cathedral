//! Host composition supplies a binary artifact, never a Rust dependency on the distro.
use std::{env, fs, path::PathBuf};

fn main() {
    for variable in [
        "CATHEDRAL_INIT_ELF",
        "CATHEDRAL_LAUNCH_ELF",
        "CATHEDRAL_LAUNCH_FRAMEBUFFER",
        "CATHEDRAL_LAUNCH_ARGUMENT",
    ] {
        println!("cargo:rerun-if-env-changed={variable}");
    }
    if env::var_os("CARGO_FEATURE_BUNDLED_USER").is_none() {
        return;
    }
    if env::var_os("CARGO_FEATURE_SMOKE_TEST").is_none() {
        startup();
        return;
    }
    for (variable, destination) in [
        ("CATHEDRAL_HELLO_ELF", "user.elf"),
        ("CATHEDRAL_IPC_ELF", "ipc.elf"),
        ("CATHEDRAL_SUPERVISION_ELF", "supervision.elf"),
        ("CATHEDRAL_DISPLAY_SERVICE_ELF", "display-service.elf"),
        ("CATHEDRAL_DISPLAY_LAB_ELF", "display-lab.elf"),
    ] {
        println!("cargo:rerun-if-env-changed={variable}");
        bundle(variable, destination);
    }
}
fn startup() {
    bundle("CATHEDRAL_INIT_ELF", "initial.elf");
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let framebuffer = match env::var("CATHEDRAL_LAUNCH_FRAMEBUFFER").as_deref() {
        Ok("1") => true,
        Ok("0") | Err(_) => false,
        _ => panic!("invalid framebuffer grant"),
    };
    let argument: u64 = env::var("CATHEDRAL_LAUNCH_ARGUMENT")
        .unwrap_or_else(|_| "0".into())
        .parse()
        .expect("invalid launch argument");
    if env::var_os("CATHEDRAL_LAUNCH_ELF").is_some() {
        bundle("CATHEDRAL_LAUNCH_ELF", "launch.elf");
    } else {
        assert!(
            !framebuffer && argument == 0,
            "child authority requires a child executable"
        );
        fs::write(output.join("launch.elf"), []).unwrap();
    }
    fs::write(
        output.join("startup_config.rs"),
        format!(
            "const FRAMEBUFFER: bool = {framebuffer};\nconst LAUNCH_ARGUMENT: u64 = {argument};\n"
        ),
    )
    .unwrap();
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
