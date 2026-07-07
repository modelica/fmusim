mod common;

use common::{run_fmusim, workspace_root};
use rstest::*;
use std::path::PathBuf;

#[rstest]
fn test_build_platform_binary_fmi2(workspace_root: PathBuf) {
    let fmu_file =
        workspace_root.join("fmusim/tests/resources/Reference-FMUs/2.0/BouncingBall.fmu");
    run_fmusim(&["build", &fmu_file.to_string_lossy()]);
}

#[rstest]
fn test_build_platform_binary_fmi3(workspace_root: PathBuf) {
    let fmu_file =
        workspace_root.join("fmusim/tests/resources/Reference-FMUs/3.0/BouncingBall.fmu");
    run_fmusim(&["build", &fmu_file.to_string_lossy()]);
}
