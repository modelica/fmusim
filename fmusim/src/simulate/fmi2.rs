use crate::simulate::{calculate_simulation_steps, split_time_intervals_indices};
use crate::{InterfaceType, SimulateArgs, SolverType};
use anyhow::Context;
use fmi_rs::model_description::fmi2::{SimpleType, Variability, VariableType};
use fmi_rs::sim::euler::ForwardEulerFactory;
use fmi_rs::sim::fmi2::Trajectories;
use fmi_rs::sim::fmi2::csv::read_csv;
use fmi_rs::sundials::solver::CVodeSolverFactory;
use plotly::layout::AxisRange;
use plotly::{
    Configuration, Layout, Plot, Scatter,
    color::NamedColor,
    common::{Fill, Line, LineShape, Mode},
    layout::{Axis, GridPattern, LayoutGrid, Margin, Shape, ShapeLayer, ShapeLine, ShapeType},
};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

pub fn simulate_fmu(
    args: &SimulateArgs,
    unzipdir: &tempfile::TempDir,
    xml_path: &Path,
) -> anyhow::Result<()> {
    let model_description = fmi_rs::model_description::fmi2::ModelDescription::from_path(xml_path)?;

    let output_variables: Vec<&fmi_rs::model_description::fmi2::ScalarVariable> =
        if args.output_variable.is_empty() {
            model_description
                .modelVariables
                .iter()
                .filter(|v| v.causality == fmi_rs::model_description::fmi2::Causality::Output)
                .collect()
        } else {
            let variable_map: HashMap<&str, &fmi_rs::model_description::fmi2::ScalarVariable> =
                model_description
                    .modelVariables
                    .iter()
                    .map(|var| (var.name.as_str(), var))
                    .collect();

            let mut output_variables = vec![];

            for variable_name in &args.output_variable {
                if let Some(&variable) = variable_map.get(variable_name.as_str()) {
                    output_variables.push(variable);
                } else {
                    return Err(anyhow::anyhow!(
                        "The requested output variable {variable_name:?} does not exist."
                    ));
                }
            }

            output_variables
        };

    let (start_time, stop_time, tolerance, output_interval) = calculate_simulation_steps(
        args,
        model_description.defaultExperiment.as_ref(),
        args.fixed_step_size,
    );

    let settings = fmi_rs::sim::fmi2::SimulationSettings {
        unzipdir: unzipdir.path(),
        model_description: &model_description,
        start_time,
        stop_time,
        set_stop_time: args.set_stop_time,
        output_interval,
        tolerance,
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
        let trajectories = fmi_rs::sim::fmi2::csv::read_csv(&file, settings.model_description)
            .with_context(|| format!("Failed to parse input file '{path}'"))?;
        Some(fmi_rs::sim::fmi2::input::StaticInput::new(trajectories))
    } else {
        None
    };

    let mut trajectories =
        fmi_rs::sim::fmi2::Trajectories::new(&model_description, output_variables.clone());

    let mut recorder = fmi_rs::sim::fmi2::recorder::Recorder::new(&mut trajectories);

    let fixes_step_size = args.fixed_step_size.unwrap_or(output_interval);

    let result = match interface_type {
        InterfaceType::ModelExchange => match args.solver {
            SolverType::Euler => fmi_rs::sim::fmi2::me::simulate(
                &settings,
                &ForwardEulerFactory { fixes_step_size },
                input.as_ref(),
                &mut recorder,
            ),
            SolverType::Cvode => fmi_rs::sim::fmi2::me::simulate(
                &settings,
                &CVodeSolverFactory,
                input.as_ref(),
                &mut recorder,
            ),
        },
        InterfaceType::CoSimulation => {
            fmi_rs::sim::fmi2::cs::simulate(&settings, input.as_ref(), &mut recorder)
        }
    };

    if let Some(output_file) = args.output_file.as_ref() {
        fmi_rs::sim::fmi2::csv::write_csv(&trajectories, output_file)
            .with_context(|| format!("Failed to write output file '{}'", output_file))?;
    }

    if args.show_plot {
        let reference = if let Some(path) = &args.reference_file {
            let reader = File::open(path)
                .with_context(|| format!("Failed to read reference file '{path}'"))?;
            let trajectories = read_csv(reader, &model_description)
                .with_context(|| format!("Failed to parse reference file '{path}'"))?;
            Some(trajectories)
        } else {
            None
        };

        let plot = plot_result(
            &trajectories,
            reference.as_ref(),
            args.show_markers,
            args.show_events,
        );

        // Generate a unique path in the temp directory starting with the model name
        let temp_path = tempfile::Builder::new()
            .prefix(&format!("{}_", model_description.modelName))
            .suffix(".html")
            .tempfile()?
            .into_temp_path();

        let path = temp_path.to_path_buf();
        // Prevent the file from being deleted immediately so the browser can read it
        let _ = temp_path.persist(&path);

        plot.show_html(path);
    }

    result?;

    Ok(())
}

pub fn plot_result(
    trajectories: &Trajectories<'_>,
    _reference: Option<&Trajectories<'_>>,
    show_markers: bool,
    show_events: bool,
) -> Plot {
    let mut plot = Plot::new();

    let plot_height = 250 * trajectories.variables.len().max(1);

    let grid_color = "rgba(211, 211, 211, 0.5)";

    let mut x_axis = Axis::new()
        .title("time [s]")
        .zero_line_color(grid_color)
        .grid_color(grid_color);

    if let (Some(first), Some(last)) = (trajectories.time.first(), trajectories.time.last()) {
        x_axis = x_axis.range(AxisRange::new(*first, *last));
    }

    let mut layout = Layout::new()
        .title(trajectories.model_description.modelName.clone())
        .x_axis(x_axis)
        .grid(
            LayoutGrid::new()
                .rows(trajectories.variables.len())
                .columns(1)
                .pattern(GridPattern::Coupled), // Link X axes in the same column
        )
        .height(plot_height)
        .auto_size(true)
        .show_legend(false)
        .margin(Margin::new().top(30).bottom(40).left(65).right(30));

    let mode = if show_markers {
        Mode::LinesMarkers
    } else {
        Mode::Lines
    };

    for (i, variable) in trajectories.variables.iter().enumerate() {
        let mut axis_title = variable.name.clone();

        if let Some(unit) = trajectories.model_description.get_unit(variable) {
            axis_title.push_str(format!(" [{unit}]").as_str());
        }

        let mut y_axis = Axis::new()
            .title(axis_title.as_str())
            .zero_line_color(grid_color)
            .grid_color(grid_color);

        if matches!(variable.variableType, VariableType::Boolean { .. }) {
            y_axis = y_axis
                .tick_values(vec![0.0, 1.0])
                .tick_text(vec!["false", "true"])
                .range(AxisRange::new(-0.01, 1.01))
                .fixed_range(true);
        }

        // Use item names as tick text for enumeration variables
        if let VariableType::Enumeration { declaredType, .. } = &variable.variableType {
            if let Some(SimpleType::Enumeration { items, .. }) = trajectories
                .model_description
                .get_type_definition(declaredType)
            {
                let tick_values: Vec<i32> = items.iter().map(|item| item.value).collect();
                let tick_text: Vec<String> = items.iter().map(|item| item.name.clone()).collect();

                let min_value = tick_values.iter().min();
                let max_value = tick_values.iter().max();

                let tick_values = tick_values.iter().map(|v| *v as f64).collect();

                y_axis = y_axis.tick_values(tick_values).tick_text(tick_text);

                if let (Some(min_value), Some(max_value)) = (min_value, max_value) {
                    y_axis = y_axis.range(AxisRange::new(*min_value, *max_value));
                }
            } else {
                continue;
            }
        }

        // Set y-axis titles for subplots (Plotly uses y1, y2, y3... internally)
        layout = match i {
            0 => layout.y_axis(y_axis),
            1 => layout.y_axis2(y_axis),
            2 => layout.y_axis3(y_axis),
            3 => layout.y_axis4(y_axis),
            4 => layout.y_axis5(y_axis),
            5 => layout.y_axis6(y_axis),
            6 => layout.y_axis7(y_axis),
            7 => layout.y_axis8(y_axis),
            _ => layout, // The plotly crate typed API typically supports up to y_axis8
        };

        let time = trajectories.time.clone();
        let name = variable.name.clone();
        let row = i + 1;

        if matches!(variable.variableType, VariableType::String { .. }) {
            continue;
        }

        let values: Vec<f64> = trajectories
            .rows
            .iter()
            .map(|row| row[i].to_f64())
            .collect();

        let mut line = Line::new().width(1.5).color("#229AEB");

        if variable.variability == Variability::Discrete {
            line = line.shape(LineShape::Hv);
        }

        for (start, end) in split_time_intervals_indices(&time) {
            let time_slice = &time[start..end];
            let values_slice = &values[start..end];

            let mut trace =
                Scatter::new(time_slice.to_owned(), values_slice.to_owned()).name(&name);

            // Use the shared x-axis ("x") for all subplots
            trace = trace
                .x_axis("x")
                .y_axis(format!("y{row}"))
                .line(line.clone())
                .mode(mode.clone());

            if matches!(variable.variableType, VariableType::Boolean { .. }) {
                trace = trace.fill(Fill::ToZeroY).fill_color(NamedColor::AliceBlue);
            }

            plot.add_trace(trace);
        }
    }

    if show_events {
        let mut shapes = Vec::new();
        for event_time in trajectories.events() {
            let shape = Shape::new()
                .shape_type(ShapeType::Line)
                .x0(event_time)
                .x1(event_time)
                .y0(0.0) // Start from bottom of plot in paper coordinates
                .y1(1.0) // End at top of plot in paper coordinates
                .x_ref("x") // Reference the x-axis for horizontal position
                .y_ref("paper") // Reference paper coordinates for vertical position
                .layer(ShapeLayer::Below) // Draw the shape behind the traces
                .line(ShapeLine::new().color(NamedColor::Gold).width(1.0));
            shapes.push(shape);
        }
        layout = layout.shapes(shapes);
    }

    plot.set_layout(layout);

    plot.set_configuration(Configuration::new().responsive(true));

    plot
}
