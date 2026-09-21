// SPDX-License-Identifier: MIT OR Apache-2.0
//! Cathedral-only test consumer of the pinned Omega checked interpreter.
//! No native publication or live device/firmware authority is requested.
use checked_interpreter::{interpret_entry, InterpretOptions};
use package_manager::operations::{prepare_local_project, LocalProjectPreparationOptions,
    check_prepared_local_project_for_inspection};
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert!(args.len() >= 4, "checked_runner ROOT BUILD_DIR MACHINE=EXPECTED...");
    let target = target::TargetProfile::host();
    let prepared = prepare_local_project(Path::new(&args[1]),
        LocalProjectPreparationOptions { target, offline: true })
        .expect("prepare isolated test package").expect("authored build.omg");
    let checked = check_prepared_local_project_for_inspection(prepared, Path::new(&args[2]), target)
        .unwrap_or_else(|error| { eprintln!("{error}"); std::process::exit(2) });
    println!("CHECKED authored package and dependency bodies; native publication NOT requested");
    let mut failed = false;
    for selection in &args[3..] {
        let (machine, expected) = selection.rsplit_once('=').expect("machine=expected");
        let expected: i32 = expected.parse().expect("expected exit value");
        let result = interpret_entry(&checked, machine, &[], InterpretOptions::default());
        let valid = result.error.is_none() && result.exit_code == expected;
        println!("{} {} expected={} observed={} error={:?} usage={:?}",
            if valid { "PASS" } else { "FAIL" }, machine, expected, result.exit_code, result.error, result.usage);
        failed |= !valid;
    }
    if failed { std::process::exit(1); }
}
