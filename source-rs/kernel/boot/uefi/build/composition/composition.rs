// Build startup_config.rs: reserve capacity, bundle launch grants and executable images, bind links.
// The target consumer is boot/uefi/startup.rs; profile selection stays in host tooling.
use crate::{
    artifacts::bundle,
    inputs::{flag, pages},
};
use std::{env, fs, path::PathBuf};

pub(super) fn generate() {
    bundle("CATHEDRAL_INIT_ELF", "initial.elf");
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let reservations::Budget {
        count,
        dynamic,
        limits,
        cpu_limits,
    } = reservations::read();
    let mut source = format!(
        "const INITIAL_CLOCK: bool = {};\nstatic LAUNCHES: &[InitialLaunch] = &[\n",
        flag("CATHEDRAL_INIT_CLOCK")
    );
    let initial_private = pages("CATHEDRAL_INIT_PRIVATE_PAGES");
    let initial_shared = pages("CATHEDRAL_INIT_SHARED_PAGES");
    append_launches(&mut source, count);
    source.push_str(&format!(
        "];\nconst CHANNEL_LIMITS: &[usize] = &{limits:?};\nconst CPU_LIMITS: &[u64] = &{cpu_limits:?};\n"
    ));
    source.push_str(&format!(
        "const DYNAMIC_TASKS: usize = {dynamic};\nconst INITIAL_PAGES: [usize; 2] = [{initial_private}, {initial_shared}];\n"
    ));
    append_executables(&mut source, count);
    append_links(&mut source, count);
    fs::write(output.join("startup_config.rs"), source).unwrap();
}

fn append_launches(source: &mut String, count: usize) {
    for index in 0..count {
        let prefix = format!("CATHEDRAL_LAUNCH_{index}");
        bundle(&format!("{prefix}_ELF"), &format!("launch-{index}.elf"));
        let framebuffer = flag(&format!("{prefix}_FRAMEBUFFER"));
        let clock = flag(&format!("{prefix}_CLOCK"));
        let keyboard = flag(&format!("{prefix}_KEYBOARD"));
        let disk = flag(&format!("{prefix}_DISK"));
        let private_pages = pages(&format!("{prefix}_PRIVATE_PAGES"));
        let shared_pages = pages(&format!("{prefix}_SHARED_PAGES"));
        let argument: u64 = env::var(format!("{prefix}_ARGUMENT"))
            .unwrap_or_else(|_| "0".into())
            .parse()
            .expect("invalid launch argument");
        source.push_str(&format!("InitialLaunch {{ elf: include_bytes!(concat!(env!(\"OUT_DIR\"), \"/launch-{index}.elf\")), framebuffer: {framebuffer}, keyboard: {keyboard}, disk: {disk}, clock: {clock}, private_pages: {private_pages}, shared_pages: {shared_pages}, argument: {argument} }},\n"));
    }
}

fn append_executables(source: &mut String, count: usize) {
    let images: usize = env::var("CATHEDRAL_IMAGE_COUNT")
        .unwrap_or_else(|_| "0".into())
        .parse()
        .unwrap();
    assert!(images + count <= cathedral_contracts::user::MAX_LAUNCHES);
    source.push_str("static EXECUTABLES: &[Program<'static>] = &[\n");
    for index in 0..images {
        bundle(
            &format!("CATHEDRAL_IMAGE_{index}_ELF"),
            &format!("image-{index}.elf"),
        );
        let argument: u64 = env::var(format!("CATHEDRAL_IMAGE_{index}_ARGUMENT"))
            .unwrap()
            .parse()
            .unwrap();
        source.push_str(&format!("Program {{ executable: Executable::Elf(include_bytes!(concat!(env!(\"OUT_DIR\"), \"/image-{index}.elf\"))), arguments: [{argument}, 0] }},\n"));
    }
    source.push_str("];\n");
}

fn append_links(source: &mut String, count: usize) {
    source.push_str("static LINKS: &[cathedral_core::ipc::link::LinkSpec] = &[\n");
    // Initial resource ownership is supplied by the same host composition as children.
    let graph = env::var("CATHEDRAL_LINKS").unwrap_or_default();
    for edge in graph.split(',').filter(|edge| !edge.is_empty()) {
        let (client, service) = edge.split_once(':').expect("link must be client:service");
        let client: usize = client.parse().unwrap();
        let service: usize = service.parse().unwrap();
        assert!(
            client > 0 && client <= count && service > 0 && service <= count && client != service
        );
        source.push_str(&format!(
            "cathedral_core::ipc::link::LinkSpec {{ client: {client}, service: {service} }},\n"
        ));
    }
    source.push_str("];\n");
}
