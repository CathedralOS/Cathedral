//! Host composition supplies a binary artifact, never a Rust dependency on the distro.
use std::{env, fs, path::PathBuf};

fn main() {
    for variable in [
        "CATHEDRAL_INIT_ELF",
        "CATHEDRAL_INIT_CLOCK",
        "CATHEDRAL_LINKS",
        "CATHEDRAL_LAUNCH_COUNT",
    ] {
        println!("cargo:rerun-if-env-changed={variable}");
    }
    for index in 0..3 {
        for field in ["ELF", "FRAMEBUFFER", "KEYBOARD", "CLOCK", "ARGUMENT"] {
            println!("cargo:rerun-if-env-changed=CATHEDRAL_LAUNCH_{index}_{field}");
        }
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
    let count: usize = env::var("CATHEDRAL_LAUNCH_COUNT")
        .unwrap_or_else(|_| "0".into())
        .parse()
        .unwrap();
    assert!(count <= 3, "at most three launch grants");
    let mut source = format!(
        "const INITIAL_CLOCK: bool = {};\nstatic LAUNCHES: &[InitialLaunch] = &[\n",
        flag("CATHEDRAL_INIT_CLOCK")
    );
    for index in 0..count {
        let prefix = format!("CATHEDRAL_LAUNCH_{index}");
        bundle(&format!("{prefix}_ELF"), &format!("launch-{index}.elf"));
        let framebuffer = flag(&format!("{prefix}_FRAMEBUFFER"));
        let clock = flag(&format!("{prefix}_CLOCK"));
        let keyboard = flag(&format!("{prefix}_KEYBOARD"));
        let argument: u64 = env::var(format!("{prefix}_ARGUMENT"))
            .unwrap_or_else(|_| "0".into())
            .parse()
            .expect("invalid launch argument");
        source.push_str(&format!("InitialLaunch {{ elf: include_bytes!(concat!(env!(\"OUT_DIR\"), \"/launch-{index}.elf\")), framebuffer: {framebuffer}, keyboard: {keyboard}, clock: {clock}, argument: {argument} }},\n"));
    }
    source.push_str("];\nstatic LINKS: &[cathedral_core::link::LinkSpec] = &[\n");
    let graph = env::var("CATHEDRAL_LINKS").unwrap_or_default();
    for edge in graph.split(',').filter(|edge| !edge.is_empty()) {
        let (client, service) = edge.split_once(':').expect("link must be client:service");
        let client: usize = client.parse().unwrap();
        let service: usize = service.parse().unwrap();
        assert!(
            client > 0 && client <= count && service > 0 && service <= count && client != service
        );
        source.push_str(&format!(
            "cathedral_core::link::LinkSpec {{ client: {client}, service: {service} }},\n"
        ));
    }
    source.push_str("];\n");
    fs::write(output.join("startup_config.rs"), source).unwrap();
}
fn flag(variable: &str) -> bool {
    match env::var(variable).as_deref() {
        Ok("1") => true,
        Ok("0") | Err(_) => false,
        _ => panic!("invalid device grant"),
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
