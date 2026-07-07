mod common;

use common::{run_fmusim, temp_dir, workspace_root};
use rstest::*;
use std::path::PathBuf;

#[rstest]
#[cfg_attr(target_os = "macos", ignore)]
#[case(2)]
#[case(3)]
fn test_serialize_fmu_state(
    workspace_root: PathBuf,
    temp_dir: PathBuf,
    #[case] fmi_major_version: u32,
) {
    let fmu_file = workspace_root.join(format!(
        "fmusim/tests/resources/Reference-FMUs/{fmi_major_version}.0/BouncingBall.fmu"
    ));
    let fmu_state_file = temp_dir.join(format!("BouncingBall_fmi{fmi_major_version}_state.bin"));

    // simulate to 1.5 seconds and save the FMU state to a file
    run_fmusim(&[
        "simulate",
        &fmu_file.to_string_lossy(),
        "--log-fmi-calls",
        "--output-interval=0.1",
        "--stop-time=1.5",
        "--final-fmu-state-file",
        &fmu_state_file.to_string_lossy(),
    ]);

    // simulate from 1.5 seconds to 3.0 seconds using the saved FMU state
    run_fmusim(&[
        "simulate",
        &fmu_file.to_string_lossy(),
        "--log-fmi-calls",
        "--output-interval=0.1",
        "--start-time=1.5",
        "--stop-time=3.0",
        "--initial-fmu-state-file",
        &fmu_state_file.to_string_lossy(),
    ]);
}
