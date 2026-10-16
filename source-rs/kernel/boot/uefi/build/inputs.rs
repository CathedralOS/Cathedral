//! Cargo dependency declarations and bounded scalar values from host composition.
use std::env;

pub(super) fn watch() {
    for variable in [
        "CATHEDRAL_INIT_ELF",
        "CATHEDRAL_INIT_CLOCK",
        "CATHEDRAL_INIT_PRIVATE_PAGES",
        "CATHEDRAL_INIT_SHARED_PAGES",
        "CATHEDRAL_LINKS",
        "CATHEDRAL_CHANNEL_LIMITS",
        "CATHEDRAL_CPU_LIMITS",
        "CATHEDRAL_LAUNCH_COUNT",
        "CATHEDRAL_DYNAMIC_TASKS",
        "CATHEDRAL_IMAGE_COUNT",
    ] {
        println!("cargo:rerun-if-env-changed={variable}");
    }
    for index in 0..cathedral_kernel_contracts::user::MAX_LAUNCHES {
        println!("cargo:rerun-if-env-changed=CATHEDRAL_IMAGE_{index}_ELF");
        println!("cargo:rerun-if-env-changed=CATHEDRAL_IMAGE_{index}_ARGUMENT");
        for field in [
            "ELF",
            "FRAMEBUFFER",
            "KEYBOARD",
            "DISK",
            "CLOCK",
            "ARGUMENT",
            "PRIVATE_PAGES",
            "SHARED_PAGES",
        ] {
            println!("cargo:rerun-if-env-changed=CATHEDRAL_LAUNCH_{index}_{field}");
        }
    }
}

pub(super) fn pages(variable: &str) -> usize {
    let pages = env::var(variable)
        .unwrap_or_else(|_| "0".into())
        .parse()
        .expect("invalid page budget");
    assert!(
        pages <= cathedral_kernel_contracts::memory::MAX_PAGES,
        "page budget exceeds lab limit"
    );
    pages
}
pub(super) fn flag(variable: &str) -> bool {
    match env::var(variable).as_deref() {
        Ok("1") => true,
        Ok("0") | Err(_) => false,
        _ => panic!("invalid device grant"),
    }
}
