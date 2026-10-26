use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
    sync::Arc,
    vec,
};

use anyhow::{Context, bail};
use fmi_rs::{
    sim::{
        fmi3::{Trajectories, csv::read_csv, input::StaticInput, recorder::Recorder},
        solver::ForwardEulerFactory,
    },
    sundials::solver::ida::IdaSolverFactory,
};
use fmi_rs::{sim::fmi3::SimulationSettings, sundials::solver::cvode::CVodeSolverFactory};

use crate::{
    InterfaceType, SimulateArgs, SolverType, simulate::{calculate_simulation_steps, plot, uplot},
};

pub fn simulate_fmu(
    args: &SimulateArgs,
    unzipdir: &tempfile::TempDir,
    xml_path: &Path,
) -> anyhow::Result<()> {
    let model_description =
        Arc::new(fmi_rs::model_description::fmi3::ModelDescription::from_path(xml_path)?);

    let output_variable_indices: Vec<usize> = if args.output_variables.is_empty() {
        model_description
            .modelVariables
            .iter()
            .enumerate()
            .filter(|(_i, v)| v.causality == fmi_rs::model_description::fmi3::Causality::Output)
            .map(|(i, _)| i)
            .collect()
    } else {
        let variable_map: HashMap<&str, usize> = model_description
            .modelVariables
            .iter()
            .enumerate()
            .map(|(i, v)| (v.name.as_str(), i))
            .collect();

        let mut output_variable_indices = vec![];

        for variable_name in &args.output_variables {
            if let Some(variable_index) = variable_map.get(variable_name.as_str()) {
                output_variable_indices.push(*variable_index);
            } else {
                return Err(anyhow::anyhow!(
                    "The requested output variable {variable_name:?} does not exist."
                ));
            }
        }

        output_variable_indices
    };

    let default_experiment = model_description.defaultExperiment.as_ref();
    let fixed_step_size = model_description
        .coSimulation
        .as_ref()
        .and_then(|cs| cs.fixedInternalStepSize.as_ref())
        .and_then(|v| v.parse().ok());

    let (start_time, stop_time, tolerance, output_interval) =
        calculate_simulation_steps(args, default_experiment, fixed_step_size);

    let settings = SimulationSettings {
        unzipdir: unzipdir.path().to_path_buf(),
        model_description: model_description.clone(),
        library_kind: args
            .library_kind
            .clone()
            .unwrap_or(crate::LibraryKind::Native)
            .into(),
        enable_dae: args.enable_dae,
        start_time,
        stop_time,
        set_stop_time: args.set_stop_time,
        output_interval,
        log_time_scale: args.log_time_scale,
        tolerance,
        set_tolerance: args.set_tolerance,
        start_values: args.start_values.clone(),
        log_fmi_calls: args.log_fmi_calls,
        intermediate_update: args.intermediate_update,
        input_file: args.input_file.as_ref().map(PathBuf::from),
        early_return_allowed: args.early_return_allowed,
        event_mode_used: args.event_mode_used,
        visible: args.visible,
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
            SolverType::Euler => fmi_rs::sim::fmi3::me::simulate(
                &settings,
                &ForwardEulerFactory {
                    fixed_step_size: fixes_step_size,
                },
                input,
                recorder.clone(),
            ),
            SolverType::Cvode => fmi_rs::sim::fmi3::me::simulate(
                &settings,
                &CVodeSolverFactory,
                input,
                recorder.clone(),
            ),
            SolverType::Ida => fmi_rs::sim::fmi3::me::simulate(
                &settings,
                &IdaSolverFactory,
                input,
                recorder.clone(),
            ),
        },
        InterfaceType::CoSimulation => {
            fmi_rs::sim::fmi3::cs::simulate(&settings, input, recorder.clone())
        }
    };

    let trajectories = if let Ok(recorder) = Arc::try_unwrap(recorder) {
        recorder.into_trajectories()
    } else {
        bail!("Failed to unwrap recorder");
    };

    if let Some(output_file) = args.output_file.as_ref() {
        fmi_rs::sim::fmi3::csv::write_csv(&trajectories, output_file)
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

        // uplot::render_html(&trajectories);

        // let html = plot::fmi3::render_html(
        //     &trajectories,
        //     ref_trajectories.as_ref(),
        //     args.show_markers,
        //     args.show_events,
        //     args.log_time_scale,
        // )?;

        // let temp_path = tempfile::Builder::new()
        //     .prefix(&format!("{}_", model_description.modelName))
        //     .suffix(".html")
        //     .tempfile()?
        //     .into_temp_path();

        // let path = temp_path.to_path_buf();
        // let _ = temp_path.persist(&path);

        // plot::show_html(&path, &html)?;
    }

    result?;

    Ok(())
}
