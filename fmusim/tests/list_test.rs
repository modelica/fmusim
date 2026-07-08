mod common;

use common::workspace_root;
use rstest::*;
use std::{path::PathBuf, process::Command};

#[rstest]
fn test_list_fmu_archive_contents(workspace_root: PathBuf) {
    let fmu_file = workspace_root.join("fmusim/tests/resources/Reference-FMUs/2.0/Resource.fmu");
    run_fmusim(&["list", &fmu_file.to_string_lossy()]);
}
