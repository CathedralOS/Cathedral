// Every consumer builds the same checked component, independently of app names.
use std::{env, path::PathBuf, process::Command};
fn main() {
    for name in [
        "OMEGA_UI_COMPONENT_DIR",
        "OMEGA_UI_REPO",
        "OMEGA_BIN",
        "PYTHON",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest directory"));
    let directory = if let Some(directory) = env::var_os("OMEGA_UI_COMPONENT_DIR") {
        PathBuf::from(directory)
    } else {
        let script = root.join("../../../../tools/omega-ui/build.py");
        println!("cargo:rerun-if-changed={}", script.display());
        let output = Command::new(env::var_os("PYTHON").unwrap_or_else(|| "python".into()))
            .arg(&script)
            .arg("--cargo")
            .env("PYTHONIOENCODING", "utf-8")
            .output()
            .expect("run the native UI build with Python 3");
        assert!(
            output.status.success(),
            "native UI preparation failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let report = String::from_utf8(output.stdout).expect("UTF-8 native UI build output");
        for line in report.lines().filter(|line| line.starts_with("cargo:")) {
            println!("{line}");
        }
        PathBuf::from(
            report
                .lines()
                .last()
                .expect("native UI component directory"),
        )
    };
    for name in [
        "receipt.json",
        "layout.bin",
        "frame.bin",
        "component.S",
        "component.rs",
    ] {
        let file = directory.join(name);
        assert!(
            file.is_file(),
            "missing compiled UI artifact {}",
            file.display()
        );
        println!("cargo:rerun-if-changed={}", file.display());
    }
    let directory = directory
        .canonicalize()
        .expect("native UI component directory");
    println!(
        "cargo:rustc-env=OMEGA_UI_ASSEMBLY={}",
        directory.join("component.S").display()
    );
    println!(
        "cargo:rustc-env=OMEGA_UI_CONSTANTS={}",
        directory.join("component.rs").display()
    );
}
