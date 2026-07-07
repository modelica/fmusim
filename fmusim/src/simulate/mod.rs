use std::fs::read_to_string;

use fmi_rs::model_description::{DefaultExperiment, FMIMajorVersion};

use crate::{SimulateArgs, SimulateConfigArgs, prepare_fmu};

pub mod fmi2;
pub mod fmi3;

// calculates the simulation steps based on the provided arguments and the default experiment settings
pub fn calculate_simulation_steps(
    args: &SimulateArgs,
    default_experiment: Option<&DefaultExperiment>,
    fixed_step_size: Option<f64>,
) -> (f64, f64, Option<f64>, f64) {
    let (default_start_time, default_stop_time, default_tolerance, default_output_interval) =
        if let Some(default_experiment) = default_experiment {
            let start_time: Option<f64> = default_experiment
                .startTime
                .as_ref()
                .and_then(|v| v.parse().ok());
            let stop_time: Option<f64> = default_experiment
                .stopTime
                .as_ref()
                .and_then(|v| v.parse().ok());
            let tolerance: Option<f64> = default_experiment
                .tolerance
                .as_ref()
                .and_then(|v| v.parse().ok());
            let output_interval: Option<f64> = default_experiment
                .stepSize
                .as_ref()
                .and_then(|v| v.parse().ok());
            (start_time, stop_time, tolerance, output_interval)
        } else {
            (None, None, None, None)
        };

    let start_time = if let Some(start_time) = args.start_time {
        start_time
    } else if let Some(default_start_time) = default_start_time {
        default_start_time
    } else if let Some(stop_time) = args.stop_time
        && stop_time < 0.0
    {
        stop_time - 1.0
    } else {
        0.0
    };

    let stop_time = if let Some(stop_time) = args.stop_time {
        stop_time
    } else if let Some(default_stop_time) = default_stop_time {
        default_stop_time
    } else {
        start_time + 1.0
    };

    let tolerance = if args.tolerance.is_some() {
        args.tolerance
    } else {
        default_tolerance
    };

    let output_interval = if let Some(output_interval) = args.output_interval {
        output_interval
    } else if let Some(default_output_interval) = default_output_interval {
        default_output_interval
    } else if let Some(fixed_step_size) = fixed_step_size {
        fixed_step_size
    } else {
        (stop_time - start_time) / 500.0
    };

    (start_time, stop_time, tolerance, output_interval)
}

pub fn simulate_fmu(args: &SimulateArgs) -> Result<(), Box<dyn std::error::Error>> {
    if args.fmu_file.is_empty() {
        return Err("No FMU file specified.".into());
    }

    let (unzipdir, xml_path, fmi_major_version) = prepare_fmu(&args.fmu_file)?;

    let start_time = std::time::Instant::now();

    let result = match fmi_major_version {
        FMIMajorVersion::V2 => crate::simulate::fmi2::simulate_fmu(args, &unzipdir, &xml_path),
        FMIMajorVersion::V3 => crate::simulate::fmi3::simulate_fmu(args, &unzipdir, &xml_path),
    };

    let elapsed_time = start_time.elapsed();

    if args.show_stats {
        eprintln!("Simulation took {:.2?}.", elapsed_time);
    }

    result
}

pub fn simulate_config(args: &SimulateConfigArgs) -> Result<(), Box<dyn std::error::Error>> {
    let content = read_to_string(&args.config_path)
        .map_err(|e| format!("Failed to read config file {}: {e}", &args.config_path))?;

    let toml_args = toml::from_str::<SimulateArgs>(&content)
        .map_err(|e| format!("Failed to parse config file {}: {e}", &args.config_path))?;

    simulate_fmu(&toml_args)
}
