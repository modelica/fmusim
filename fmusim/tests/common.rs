use std::{path::PathBuf, process::Command};

use fmi_rs::util::download_reference_fmus;
use rstest::*;

#[fixture]
pub fn workspace_root() -> PathBuf {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();

    let reference_fmus_dir = workspace_root.join("fmusim/tests/resources/Reference-FMUs");

    if !reference_fmus_dir.exists() {
        download_reference_fmus(&reference_fmus_dir).unwrap()
    }

    workspace_root
}

#[fixture]
pub fn temp_dir() -> PathBuf {
    let temp_path = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("tests")
        .to_path_buf();

    if !temp_path.exists() {
        std::fs::create_dir_all(&temp_path).unwrap();
    }

    temp_path
}

pub fn run_fmusim(args: &[&str]) {
    let workspace_root = workspace_root();
    let fmusim_path = workspace_root.join("target/debug/fmusim");

    let simulation_output = Command::new(&fmusim_path)
        .args(args)
        .current_dir(workspace_root)
        .output()
        .expect("Failed to run fmusim");

    if !simulation_output.status.success() {
        panic!(
            "fmusim failed (status: {:?})\n--- Stdout ---\n{}\n--- Stderr ---\n{}",
            simulation_output.status.code(),
            String::from_utf8_lossy(&simulation_output.stdout),
            String::from_utf8_lossy(&simulation_output.stderr)
        );
    }
}
