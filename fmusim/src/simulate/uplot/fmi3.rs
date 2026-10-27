use fmi_rs::model_description::fmi3::{TypeDefinition, VariableType};
use fmi_rs::sim::fmi3::Trajectories;

use super::reference::resample_to_sim_time;
use super::{Plot, SimulationResults, TimeSeries};

pub const MAX_PLOTS: usize = 8;

pub fn simulation_results(
    trajectories: &Trajectories,
    ref_trajectories: Option<&Trajectories>,
) -> SimulationResults {
    let time = trajectories.time.clone();
    let mut plots = Vec::new();

    for (variable_index, variable) in trajectories.variables().take(MAX_PLOTS).enumerate() {
        if matches!(
            variable.variableType,
            VariableType::String { .. } | VariableType::Binary { .. }
        ) {
            continue;
        }

        if let VariableType::Enumeration { declaredType, .. } = &variable.variableType
            && trajectories
                .model_description
                .get_type_definition(declaredType)
                .is_none()
        {
            continue;
        }

        let mut title = variable.name.clone();
        if let Some(unit) = trajectories.model_description.get_unit(variable) {
            title = format!("{title} [{unit}]");
        }

        let (fill, y_ticks, range) = panel_axis_hints(trajectories, variable);

        let trajectory: Option<Vec<Vec<f64>>> = trajectories
            .rows
            .iter()
            .map(|row| row.get(variable_index).and_then(|v| v.as_f64()))
            .collect();

        let trajectory = match trajectory {
            Some(t) => t,
            None => continue,
        };

        let size = trajectory.first().map(|v| v.len()).unwrap_or(0);
        if size == 0 {
            continue;
        }

        let name = variable.name.clone();
        let mut series = Vec::new();

        for j in 0..size {
            let scalar_values: Option<Vec<f64>> = trajectory.iter().map(|v| v.get(j).cloned()).collect();
            let data = match scalar_values {
                Some(values) if values.len() == time.len() => values,
                _ => continue,
            };

            let series_name = if size > 1 {
                format!("{name}[{j}]")
            } else {
                name.clone()
            };

            series.push(TimeSeries {
                name: series_name.clone(),
                fill: fill.clone(),
                discrete: false,
                dashed: false,
                data,
            });

            if let Some(ref_trajectories) = ref_trajectories {
                let scalar_ref_values: Option<Vec<f64>> = ref_trajectories
                    .rows
                    .iter()
                    .map(|row| {
                        row.get(variable_index)
                            .and_then(|v| v.as_f64())
                            .and_then(|v| v.get(j).cloned())
                    })
                    .collect();

                if let Some(ref_y) = scalar_ref_values
                    && let Some(ref_data) =
                        resample_to_sim_time(&time, &ref_trajectories.time, &ref_y)
                {
                    series.push(TimeSeries {
                        name: format!("{series_name} (reference)"),
                        fill: None,
                        discrete: false,
                        dashed: true,
                        data: ref_data,
                    });
                }
            }
        }

        if !series.is_empty() {
            plots.push(Plot {
                title,
                y_ticks,
                range,
                series,
            });
        }
    }

    SimulationResults {
        time,
        plots,
        events: trajectories.events(),
    }
}

fn panel_axis_hints(
    trajectories: &Trajectories,
    variable: &fmi_rs::model_description::fmi3::ModelVariable,
) -> (Option<String>, Option<Vec<(f64, String)>>, Option<(f64, f64)>) {
    if matches!(variable.variableType, VariableType::Boolean { .. }) {
        return (
            Some("rgba(0, 0, 255, 0.1)".to_string()),
            Some(vec![(0.0, "false".to_string()), (1.0, "true".to_string())]),
            Some((-0.1, 1.1)),
        );
    }

    if let VariableType::Enumeration { declaredType, .. } = &variable.variableType {
        if let Some(TypeDefinition::Enumeration { items, .. }) = trajectories
            .model_description
            .get_type_definition(declaredType)
        {
            let y_ticks: Vec<(f64, String)> = items
                .iter()
                .map(|item| (item.value as f64, item.name.clone()))
                .collect();
            let y_min = items.iter().map(|i| i.value).min().map(|v| v as f64);
            let y_max = items.iter().map(|i| i.value).max().map(|v| v as f64);
            return (
                None,
                Some(y_ticks),
                match (y_min, y_max) {
                    (Some(min), Some(max)) => Some((min, max)),
                    _ => None,
                },
            );
        }
    }

    (None, None, None)
}
