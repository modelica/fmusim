use fmi_rs::util::download_reference_fmus;
use rstest::*;
use std::{fs::read_to_string, path::PathBuf, process::Command};

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

#[rstest]
fn test_input_interpolation_fmi2(workspace_root: PathBuf) {
    let fmu_file = 
        workspace_root.join("fmusim/tests/resources/Reference-FMUs/2.0/Feedthrough.fmu");
    let fmusim_path = workspace_root.join("target/debug/fmusim");
    let input_file =
        workspace_root.join("fmusim/tests/resources/fmi2/Feedthrough_interpolation_in.csv");
    let output_file =
        workspace_root.join("fmusim/tests/resources/fmi2/Feedthrough_interpolation_out.csv");
    let expected_file =
        workspace_root.join("fmusim/tests/resources/fmi2/Feedthrough_interpolation_expected.csv");

    let simulation_output = Command::new(&fmusim_path)
        .args(&[
            "simulate",
            &fmu_file.to_string_lossy(),
            "--input-file",
            &input_file.to_string_lossy(),
            "--output-file",
            &output_file.to_string_lossy(),
            // "--log-fmi-calls",
            "--stop-time=3",
            "--output-interval=0.25",
            "--output-variable=Float64_continuous_input",
            "--output-variable=Float64_discrete_input",
        ])
        .current_dir(&workspace_root)
        .output()
        .expect("Failed to run fmusim");

    // let out = String::from_utf8_lossy(&simulation_output.stdout);
    // print!("{out}");

    // let err = String::from_utf8_lossy(&simulation_output.stderr);
    // print!("{err}");

    if !simulation_output.status.success() {
        panic!(
            "Simulation failed: {}\nStderr: {}",
            String::from_utf8_lossy(&simulation_output.stdout),
            String::from_utf8_lossy(&simulation_output.stderr)
        );
    }

    let expected = read_to_string(expected_file).unwrap();
    let actual = read_to_string(output_file).unwrap();

    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();

    assert_eq!(expected_lines, actual_lines);
}

#[rstest]
fn test_input_interpolation_fmi3(workspace_root: PathBuf) {
    let fmu_file = 
        workspace_root.join("fmusim/tests/resources/Reference-FMUs/3.0/Feedthrough.fmu");
    let fmusim_path = workspace_root.join("target/debug/fmusim");
    let input_file =
        workspace_root.join("fmusim/tests/resources/fmi3/Feedthrough_interpolation_in.csv");
    let output_file =
        workspace_root.join("fmusim/tests/resources/fmi3/Feedthrough_interpolation_out.csv");
    let expected_file =
        workspace_root.join("fmusim/tests/resources/fmi3/Feedthrough_interpolation_expected.csv");

    let simulation_output = Command::new(&fmusim_path)
        .args(&[
            "simulate",
            &fmu_file.to_string_lossy(),
            "--input-file",
            &input_file.to_string_lossy(),
            "--output-file",
            &output_file.to_string_lossy(),
            "--log-fmi-calls",
            "--stop-time=3",
            "--output-interval=0.25",
            "--output-variable=Float64_continuous_input",
            "--output-variable=Float64_discrete_input",
        ])
        .current_dir(&workspace_root)
        .output()
        .expect("Failed to run fmusim");

    if !simulation_output.status.success() {
        panic!(
            "Simulation failed: {}\nStderr: {}",
            String::from_utf8_lossy(&simulation_output.stdout),
            String::from_utf8_lossy(&simulation_output.stderr)
        );
    }

    let expected = read_to_string(expected_file).unwrap();
    let actual = read_to_string(output_file).unwrap();

    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();

    assert_eq!(expected_lines, actual_lines);
}
