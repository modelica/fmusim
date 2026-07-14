use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::Context;
use fmi_rs::{
    model_description::fmi3::{TypeDefinition, VariableType},
    sim::{
        euler::ForwardEulerFactory,
        fmi3::{Trajectories, csv::read_csv},
    },
};
use fmi_rs_sundials::solver::CVodeSolverFactory;
use plotly::{
    Configuration, Layout, Plot, Scatter,
    color::NamedColor,
    common::{Fill, Line, Mode},
    layout::{
        Axis, AxisRange, GridPattern, LayoutGrid, Margin, Shape, ShapeLayer, ShapeLine, ShapeType,
    },
};

use crate::{InterfaceType, SimulateArgs, SolverType, simulate::calculate_simulation_steps};

pub fn simulate_fmu(
    args: &SimulateArgs,
    unzipdir: &tempfile::TempDir,
    xml_path: &Path,
) -> anyhow::Result<()> {
    let model_description = fmi_rs::model_description::fmi3::ModelDescription::from_path(xml_path)?;

    let output_variables: Vec<&fmi_rs::model_description::fmi3::ModelVariable> =
        if args.output_variable.is_empty() {
            model_description
                .modelVariables
                .iter()
                .filter(|v| v.causality == fmi_rs::model_description::fmi3::Causality::Output)
                .collect()
        } else {
            let variable_map: HashMap<&str, &fmi_rs::model_description::fmi3::ModelVariable> =
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

    let default_experiment = model_description.defaultExperiment.as_ref();
    let fixed_step_size = model_description
        .coSimulation
        .as_ref()
        .and_then(|cs| cs.fixedInternalStepSize.as_ref())
        .and_then(|v| v.parse().ok());

    let (start_time, stop_time, tolerance, output_interval) =
        calculate_simulation_steps(args, default_experiment, fixed_step_size);

    let settings = fmi_rs::sim::fmi3::SimulationSettings {
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
        let file = File::open(path)?;
        let trajectories = fmi_rs::sim::fmi3::csv::read_csv(&file, settings.model_description)?;
        Some(fmi_rs::sim::fmi3::input::StaticInput::new(trajectories))
    } else {
        None
    };

    let mut trajectories =
        fmi_rs::sim::fmi3::Trajectories::new(&model_description, output_variables.clone());

    let mut recorder = fmi_rs::sim::fmi3::recorder::Recorder::new(&mut trajectories);

    let fixes_step_size = args.fixed_step_size.unwrap_or(output_interval);

    let result = match interface_type {
        InterfaceType::ModelExchange => match args.solver {
            SolverType::Euler => fmi_rs::sim::fmi3::simulate_me(
                &settings,
                &ForwardEulerFactory { fixes_step_size },
                input.as_ref(),
                &mut recorder,
            ),
            SolverType::Cvode => fmi_rs::sim::fmi3::simulate_me(
                &settings,
                &CVodeSolverFactory,
                input.as_ref(),
                &mut recorder,
            ),
        },
        InterfaceType::CoSimulation => {
            fmi_rs::sim::fmi3::simulate_cs(&settings, input.as_ref(), &mut recorder)
        }
    };

    if let Some(output_file) = args.output_file.as_ref() {
        fmi_rs::sim::fmi3::csv::write_csv(&trajectories, output_file)
            .with_context(|| format!("Failed to write '{}'", output_file))?;
    }

    if args.show_plot {
        let ref_trajectories = if let Some(path) = &args.reference_file {
            let reader = File::open(path)?;
            Some(read_csv(reader, &model_description)?)
        } else {
            None
        };

        let plot = plot_result(
            &trajectories,
            ref_trajectories.as_ref(),
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
    ref_trajectories: Option<&Trajectories<'_>>,
    show_markers: bool,
    show_events: bool,
) -> Plot {
    let mut plot = Plot::new();

    const COLORS: [&str; 8] = [
        "#229AEB", // Muted Blue
        "#FFA02C", // Orange
        "#33CC33", // Green
        "#CC3333", // Red
        "#9933CC", // Purple
        "#996633", // Brown
        "#FF66B2", // Pink
        "#999999", // Gray
    ];

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
                .range(AxisRange::new(0.0, 1.0))
                .fixed_range(true);
        }

        // Use item names as tick text for enumeration variables
        if let VariableType::Enumeration { declaredType, .. } = &variable.variableType {
            if let Some(TypeDefinition::Enumeration { items, .. }) = trajectories
                .model_description
                .get_type_definition(declaredType)
            {
                let tick_values: Vec<i64> = items.iter().map(|item| item.value).collect();
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

        if matches!(
            variable.variableType,
            VariableType::String { .. } | VariableType::Binary { .. }
        ) {
            continue;
        }

        let mut color_iter = COLORS.iter().cycle();

        if let Some(first_row) = trajectories.rows.first()
            && let Some(value) = first_row.get(i)
        {
            let size = value.len();

            for j in 0..size {
                let scalar_values: Vec<f64> = trajectories
                    .rows
                    .iter()
                    .map(|row| row[i].as_f64()[j])
                    .collect();

                let name = if size > 1 {
                    format!("{}[{}]", name, j)
                } else {
                    name.clone()
                };

                let current_color = color_iter.next().unwrap_or(&COLORS[0]);

                if let Some(ref_trajectories) = ref_trajectories {
                    let scalar_ref_values: Vec<f64> = ref_trajectories
                        .rows
                        .iter()
                        .map(|row| row[i].as_f64()[j])
                        .collect();

                    let mut ref_trace =
                        Scatter::new(ref_trajectories.time.clone(), scalar_ref_values)
                            .name(name.clone());

                    let ref_line = Line::new().width(3.0).color(format!("{current_color}44"));

                    ref_trace = ref_trace
                        .x_axis("x")
                        .y_axis(format!("y{row}"))
                        .line(ref_line);

                    plot.add_trace(ref_trace);
                }

                let mut trace = Scatter::new(time.clone(), scalar_values).name(name);

                // Use the shared x-axis ("x") for all subplots
                trace = trace
                    .x_axis("x")
                    .y_axis(format!("y{row}"))
                    .line(Line::new().width(1.5).color(*current_color));

                if show_markers {
                    trace = trace.mode(Mode::LinesMarkers);
                }

                if matches!(variable.variableType, VariableType::Boolean { .. }) {
                    trace = trace.fill(Fill::ToZeroY).fill_color(NamedColor::AliceBlue);
                }

                plot.add_trace(trace);
            }
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
