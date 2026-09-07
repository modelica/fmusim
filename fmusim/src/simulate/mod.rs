use std::{fs::read_to_string, ops::Range, vec};

use fmi_rs::model_description::{DefaultExperiment, FMIMajorVersion};

use crate::{SimulateArgs, prepare_fmu};
use itertools::Itertools;

pub mod fmi2;
pub mod fmi3;

// calculates the simulation steps based on the provided arguments and the default experiment settings
pub fn calculate_simulation_steps(
    args: &SimulateArgs,
    default_experiment: Option<&DefaultExperiment>,
    fixed_step_size: Option<f64>,
) -> (f64, f64, f64, f64) {
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

    let tolerance = args
        .tolerance
        .unwrap_or_else(|| default_tolerance.unwrap_or(1e-4));

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

pub fn simulate_fmu(args: &SimulateArgs) -> anyhow::Result<()> {
    if args.fmu_file.is_empty() {
        return Err(anyhow::anyhow!("No FMU file specified."));
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

    result?;

    Ok(())
}

pub fn simulate_config(config_file: &str) -> anyhow::Result<()> {
    let content = read_to_string(config_file)?;
    let toml_args = toml::from_str::<SimulateArgs>(&content)?;
    simulate_fmu(&toml_args)
}

/// Returns a list of Range<usize> for slicing.
/// The first element at an event (first duplicate) is included in the prior range.
/// Intermediate duplicates are skipped, and the next range starts at the last duplicate.
pub fn split_time_intervals_ranges(time_steps: &[f64]) -> Vec<Range<usize>> {
    let mut intervals = vec![];

    if let Some(start_time) = time_steps.first() {
        let mut last_idx = 0usize;
        let mut last_time = *start_time;

        for (idx, (t0, t1)) in time_steps.iter().copied().tuple_windows().enumerate() {
            if t0 == t1 {
                if t0 != last_time {
                    intervals.push(last_idx..idx + 1);
                }
                last_idx = idx + 1;
                last_time = t0;
            }
        }

        intervals.push(last_idx..time_steps.len());
    }

    intervals
}

#[cfg(test)]
mod tests {
    use super::split_time_intervals_ranges;
    use std::ops::Range;

    #[test]
    fn split_time_intervals_ranges_handles_empty_and_single_step_inputs() {
        assert_eq!(split_time_intervals_ranges(&[]), Vec::<Range<usize>>::new());
        assert_eq!(split_time_intervals_ranges(&[0.0]), vec![0..1]);
    }

    #[test]
    fn split_time_intervals_ranges_returns_one_range_without_events() {
        assert_eq!(split_time_intervals_ranges(&[0.0, 1.0, 2.0]), vec![0..3]);
    }

    #[test]
    fn split_time_intervals_ranges_splits_at_duplicate_times() {
        assert_eq!(
            split_time_intervals_ranges(&[0., 1., 2., 2., 2., 3., 4., 4., 5.]),
            vec![0..3, 4..7, 7..9]
        );
    }
}
