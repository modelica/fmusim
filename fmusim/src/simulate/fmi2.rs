use crate::simulate::{calculate_simulation_steps, uplot};
use crate::{InterfaceType, SimulateArgs, SolverType};
use anyhow::{Context, bail};
use fmi_rs::model_description::fmi2::{Causality, ModelDescription};
use fmi_rs::sim::fmi2::csv::{read_csv, write_csv};
use fmi_rs::sim::fmi2::input::StaticInput;
use fmi_rs::sim::fmi2::recorder::Recorder;
use fmi_rs::sim::fmi2::{SimulationSettings, Trajectories, cs, me};
use fmi_rs::sim::solver::ForwardEulerFactory;
use fmi_rs::sundials::solver::cvode::CVodeSolverFactory;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub fn simulate_fmu(
    args: &SimulateArgs,
    unzipdir: &tempfile::TempDir,
    xml_path: &Path,
) -> anyhow::Result<()> {
    let model_description = Arc::new(ModelDescription::from_path(xml_path)?);

    let output_variable_indices: Vec<usize> = if args.output_variables.is_empty() {
        model_description
            .modelVariables
            .iter()
            .enumerate()
            .filter(|(_i, v)| v.causality == Causality::Output)
            .map(|(i, _v)| i)
            .collect()
    } else {
        args.output_variables
            .iter()
            .map(|name| model_description.variable_index_by_name(name))
            .collect::<Result<Vec<_>, _>>()?
    };

    let (start_time, stop_time, tolerance, output_interval) = calculate_simulation_steps(
        args,
        model_description.defaultExperiment.as_ref(),
        args.fixed_step_size,
    );

    let settings = SimulationSettings {
        unzipdir: unzipdir.path().to_path_buf(),
        model_description: model_description.clone(),
        start_time,
        stop_time,
        set_stop_time: args.set_stop_time,
        output_interval,
        log_time_scale: args.log_time_scale,
        tolerance,
        set_tolerance: args.set_tolerance,
        start_values: args.start_values.clone(),
        log_fmi_calls: args.log_fmi_calls,
        input_file: args.input_file.as_ref().map(PathBuf::from),
        early_return_allowed: args.early_return_allowed,
        event_mode_used: args.event_mode_used,
        logging_on: args.logging_on,
        log_file: args.log_file.as_ref().map(PathBuf::from),
        initial_fmu_state_file: args.initial_fmu_state_file.as_ref().map(PathBuf::from),
        final_fmu_state_file: args.final_fmu_state_file.as_ref().map(PathBuf::from),
    };

    let interface_type = match &args.interface_type {
        Some(t) => t.clone(),
        None => {
            if model_description.coSimulation.is_some() {
                InterfaceType::CoSimulation
            } else {
                InterfaceType::ModelExchange
            }
        }
    };

    let input = if let Some(path) = &args.input_file {
        let file =
            File::open(path).with_context(|| format!("Failed to read input file '{path}'"))?;
        let trajectories = read_csv(&file, settings.model_description.clone())
            .with_context(|| format!("Failed to parse input file '{path}'"))?;
        Some(Arc::new(StaticInput::new(trajectories, tolerance)))
    } else {
        None
    };

    let trajectories = Trajectories::new(model_description.clone(), output_variable_indices);

    #[allow(clippy::arc_with_non_send_sync)]
    let recorder = Arc::new(Recorder::new(trajectories));

    let fixes_step_size = args.fixed_step_size.unwrap_or(output_interval);

    let result = match interface_type {
        InterfaceType::ModelExchange => match args.solver {
            SolverType::Euler => me::simulate(
                &settings,
                &ForwardEulerFactory {
                    fixed_step_size: fixes_step_size,
                },
                input,
                recorder.clone(),
            ),
            SolverType::Cvode => {
                me::simulate(&settings, &CVodeSolverFactory, input, recorder.clone())
            }
            SolverType::Ida => bail!("DAEs are not supported in FMI 2.0"),
        },
        InterfaceType::CoSimulation => cs::simulate(&settings, input, recorder.clone()),
    };

    let trajectories = if let Ok(recorder) = Arc::try_unwrap(recorder) {
        recorder.into_trajectories()
    } else {
        bail!("Failed to unwrap recorder");
    };

    if let Some(output_file) = args.output_file.as_ref() {
        write_csv(&trajectories, output_file)
            .with_context(|| format!("Failed to write output file '{}'", output_file))?;
    }

    if args.show_plot {
        let ref_trajectories = if let Some(path) = &args.reference_file {
            let reader = File::open(path)
                .with_context(|| format!("Failed to read reference file '{path}'"))?;
            let trajectories = read_csv(reader, model_description.clone())
                .with_context(|| format!("Failed to parse reference file '{path}'"))?;
            Some(trajectories)
        } else {
            None
        };

        let results =
            uplot::fmi2::simulation_results(&trajectories, ref_trajectories.as_ref());
        uplot::render_html(results);
    }

    result?;

    Ok(())
}
