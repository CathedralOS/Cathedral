use std::{env, path::PathBuf, process::Command};
fn main() {
    for key in ["OMEGA_BIN", "PYTHON"] {
        println!("cargo:rerun-if-env-changed={key}");
    }
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("../../../..");
    let script = root.join("tools/omega-component/build.py");
    println!("cargo:rerun-if-changed={}", script.display());
    let output = Command::new(env::var_os("PYTHON").unwrap_or_else(|| "python".into()))
        .arg(script)
        .arg(root.join("source/distribution/applications/colors/bridge/native"))
        .arg("--cargo")
        .env("PYTHONIOENCODING", "utf-8")
        .output()
        .expect("prepare Omega Colors");
    assert!(
        output.status.success(),
        "Omega Colors preparation:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report = String::from_utf8(output.stdout).unwrap();
    for line in report.lines().filter(|line| line.starts_with("cargo:")) {
        println!("{line}");
    }
    let directory = PathBuf::from(report.lines().last().unwrap());
    for name in ["component.S", "component.rs", "app.bin", "receipt.json"] {
        println!("cargo:rerun-if-changed={}", directory.join(name).display());
    }
    println!(
        "cargo:rustc-env=OMEGA_APP_ASSEMBLY={}",
        directory.join("component.S").display()
    );
    println!(
        "cargo:rustc-env=OMEGA_APP_CONSTANTS={}",
        directory.join("component.rs").display()
    );
}
