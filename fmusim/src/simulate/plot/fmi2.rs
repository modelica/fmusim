use fmi_rs::model_description::fmi2::{Variability, VariableType};
use fmi_rs::sim::fmi2::Trajectories;

use super::{Plot, SimulationResults, TimeSeries};

pub fn simulation_results(
    trajectories: &Trajectories,
    ref_trajectories: Option<&Trajectories>,
) -> SimulationResults {
    let time = trajectories.time.clone();
    let mut plots = Vec::new();

    for (i, variable) in trajectories.variables().enumerate() {
        if let Some(values) = trajectories.trajectory_by_index(i) {
            let data: Vec<f64> = values
                .iter()
                .map(|v| v.to_f64().unwrap_or_default())
                .collect();
            if data.len() != time.len() {
                continue;
            }

            let mut title = variable.name.clone();
            if let Some(unit) = trajectories.model_description.get_unit(variable) {
                title = format!("{title} [{unit}]");
            }

            let (fill, y_ticks, range) =
                if matches!(variable.variableType, VariableType::Boolean { .. }) {
                    let y_ticks = Some(vec![(0.0, "false".to_string()), (1.0, "true".to_string())]);
                    (
                        Some("rgba(0, 0, 255, 0.1)".to_string()),
                        y_ticks,
                        Some((-0.1, 1.1)),
                    )
                } else {
                    (None, None, None)
                };

            let discrete = variable.variability != Variability::Continuous;
            let series_name = variable.name.clone();

            let mut series = vec![TimeSeries {
                name: series_name.clone(),
                time: None,
                fill,
                discrete,
                reference: false,
                data,
            }];

            if let Some(ref_trajectories) = ref_trajectories
                && let Some(ref_y) = ref_trajectories.trajectory_by_index(i).map(|values| {
                    values
                        .iter()
                        .map(|v| v.to_f64().unwrap_or_default())
                        .collect::<Vec<f64>>()
                })
            {
                series.push(TimeSeries {
                    name: format!("{series_name} (reference)"),
                    time: Some(ref_trajectories.time.clone()),
                    fill: None,
                    discrete,
                    reference: true,
                    data: ref_y,
                });
            }

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
        log_x: false,
    }
}
