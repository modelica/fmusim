mod common;

use common::{run_fmusim_output, workspace_root};
use rstest::*;
use std::path::Path;

#[rstest]
#[case(2, "fmi2DoStep")]
#[case(3, "fmi3DoStep")]
fn test_log_fmi_calls(
    workspace_root: &Path,
    #[case] fmi_major_version: u32,
    #[case] do_step_call: &str,
) {
    let fmu_file = workspace_root.join(format!(
        "fmusim/tests/resources/Reference-FMUs/{fmi_major_version}.0/Dahlquist.fmu"
    ));
    let fmu = fmu_file.to_string_lossy();

    let base_args = [
        "simulate",
        &fmu,
        "--stop-time=0.1",
        "--output-interval=0.1",
    ];

    let without_logging = run_fmusim_output(&base_args);
    let stderr_without = String::from_utf8_lossy(&without_logging.stderr);
    assert!(
        !stderr_without.contains("[FMI]"),
        "expected no FMI call log without --log-fmi-calls, got stderr:\n{stderr_without}"
    );
    assert!(
        !stderr_without.contains(do_step_call),
        "expected no {do_step_call} log without --log-fmi-calls, got stderr:\n{stderr_without}"
    );

    let with_logging = run_fmusim_output(&[
        "simulate",
        &fmu,
        "--log-fmi-calls",
        "--stop-time=0.1",
        "--output-interval=0.1",
    ]);
    let stderr_with = String::from_utf8_lossy(&with_logging.stderr);
    assert!(
        stderr_with.contains("[FMI]"),
        "expected FMI call log with --log-fmi-calls, got stderr:\n{stderr_with}"
    );
    assert!(
        stderr_with.contains(do_step_call),
        "expected {do_step_call} in log with --log-fmi-calls, got stderr:\n{stderr_with}"
    );
}
