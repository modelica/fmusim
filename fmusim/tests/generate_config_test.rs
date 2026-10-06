mod common;

use std::fs;
use std::path::Path;

use common::workspace_root;
use rstest::*;

use crate::common::run_fmusim;

#[rstest]
fn test_generate_config_for_resource_fmu(workspace_root: &Path) {
    let fmu_path = workspace_root.join("fmusim/tests/resources/Reference-FMUs/2.0/Resource.fmu");
    let config_path = workspace_root.join("target/tmp/tests/Resource_config.toml");

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    run_fmusim(&[
        "generate-config",
        &fmu_path.to_string_lossy(),
        "--config-file",
        &config_path.to_string_lossy(),
    ]);

    let config = fs::read_to_string(&config_path).unwrap();
    assert!(config.contains("fmu_file"));
    assert!(config.contains("output_variable"));

    run_fmusim(&["simulate-config", &config_path.to_string_lossy()]);
}
