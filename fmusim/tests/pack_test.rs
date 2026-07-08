mod common;

use common::workspace_root;
use rstest::*;
use std::path::PathBuf;

use crate::common::{run_fmusim, temp_dir};

#[rstest]
fn test_unpack_and_pack(workspace_root: PathBuf, temp_dir: PathBuf) {
    let fmu_file = workspace_root.join("fmusim/tests/resources/Reference-FMUs/3.0/Resource.fmu");
    let unpack_dir = temp_dir.join("StateSpace");
    let packed_fmu = temp_dir.join("StateSpace.fmu");

    run_fmusim(&[
        "unpack",
        &fmu_file.to_string_lossy(),
        &unpack_dir.to_string_lossy(),
    ]);

    run_fmusim(&[
        "pack",
        &unpack_dir.to_string_lossy(),
        &packed_fmu.to_string_lossy(),
    ]);

    run_fmusim(&["validate", &packed_fmu.to_string_lossy()]);
}
