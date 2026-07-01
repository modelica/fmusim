mod common;

use common::{run_fmusim, temp_dir, workspace_root};
use rstest::*;
use std::{fs::read_to_string, path::PathBuf};

#[rstest]
#[cfg(not(target_os = "macos"))]
fn test_input_interpolation_fmi2(workspace_root: PathBuf, temp_dir: PathBuf) {
    let fmu_file = workspace_root.join("fmusim/tests/resources/Reference-FMUs/2.0/Feedthrough.fmu");
    let input_file =
        workspace_root.join("fmusim/tests/resources/fmi2/Feedthrough_interpolation_in.csv");
    let output_file = temp_dir.join("Feedthrough_interpolation_fmi2_out.csv");
    let expected_file =
        workspace_root.join("fmusim/tests/resources/fmi2/Feedthrough_interpolation_expected.csv");

    run_fmusim(&[
        "simulate",
        &fmu_file.to_string_lossy(),
        "--input-file",
        &input_file.to_string_lossy(),
        "--output-file",
        &output_file.to_string_lossy(),
        "--stop-time=3",
        "--output-interval=0.25",
        "--output-variable=Float64_continuous_input",
        "--output-variable=Float64_discrete_input",
    ]);

    let expected = read_to_string(expected_file).unwrap();
    let actual = read_to_string(output_file).unwrap();

    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();

    assert_eq!(expected_lines, actual_lines);
}

#[rstest]
fn test_input_interpolation_fmi3(workspace_root: PathBuf, temp_dir: PathBuf) {
    let fmu_file = workspace_root.join("fmusim/tests/resources/Reference-FMUs/3.0/Feedthrough.fmu");
    let input_file =
        workspace_root.join("fmusim/tests/resources/fmi3/Feedthrough_interpolation_in.csv");
    let output_file = temp_dir.join("Feedthrough_interpolation_fmi3_out.csv");
    let expected_file =
        workspace_root.join("fmusim/tests/resources/fmi3/Feedthrough_interpolation_expected.csv");

    run_fmusim(&[
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
    ]);

    let expected = read_to_string(expected_file).unwrap();
    let actual = read_to_string(output_file).unwrap();

    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();

    assert_eq!(expected_lines, actual_lines);
}
