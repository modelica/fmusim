use std::fs;
use std::io;

use askama::Template;
use fmi_rs::model_description::fmi2::Variability;
use fmi_rs::model_description::fmi2::VariableType;
use fmi_rs::sim::fmi2::Trajectories;
use serde::Serialize;

#[derive(Serialize)]
struct Signal {
    name: String,
    color: String,
    fill: Option<String>,
    discrete: bool,
    data: Vec<f64>,
    y_ticks: Option<Vec<(f64, String)>>,
    range: Option<(f64, f64)>,
}

#[derive(Serialize)]
struct SimulationResults {
    time: Vec<f64>,
    signals: Vec<Signal>,
}

#[derive(Template)]
#[template(path = "multi_signal_report.html.askama")]
struct MultiSignalReportTemplate {
    time_json: String,
    signals_json: String,
}

fn generate_html_report(results: &SimulationResults, output_path: &str) -> io::Result<()> {
    let time_json = serde_json::to_string(&results.time).unwrap();
    let signals_json = serde_json::to_string(&results.signals).unwrap();

    let html_content = MultiSignalReportTemplate {
        time_json,
        signals_json,
    }
    .render()
    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    fs::write(output_path, html_content)
}

pub fn render_html(trajectories: &Trajectories) {

    let mut signals = vec![];

    for (i, variable) in trajectories.variables().enumerate() {
      if let Some(values) = trajectories.trajectory_by_index(i) {
        let data: Vec<f64> = values.iter().map(|v| v.to_f64().unwrap_or_default()).collect();
        let mut name = variable.name.clone();
        if let Some(unit) = trajectories.model_description.get_unit(variable) {
          name = format!("{name} [{unit}]");
        }
        let (fill, y_ticks, range) = if matches!(variable.variableType, VariableType::Boolean { .. }) {
          let y_ticks = Some(vec![
            (0.0, "false".to_string()), 
            (1.0, "true".to_string()),
          ]);
          (Some("rgba(0, 0, 255, 0.1)".to_string()), y_ticks, Some((-0.1, 1.1)))
        } else {
          (None, None, None)
        };

        let discrete = variable.variability !=  Variability::Continuous;

        let signal = Signal {
          name,
          color: "#3b82f6".to_string(),
          fill,
          discrete,
          data,
          y_ticks,
          range,
        };

        signals.push(signal);
      }

    }

    let results = SimulationResults {
        time: trajectories.time.clone(), // vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0],
        signals,
        // signals: vec![
        //     Signal {
        //         name: "Temperature".to_string(),
        //         unit: "°C".to_string(),
        //         color: "#3b82f6".to_string(),
        //         data: vec![20.0, 22.5, 27.1, 35.4, 42.0, 48.2, 51.0, 53.5, 52.0, 50.1],
        //     },
        //     Signal {
        //         name: "Pressure".to_string(),
        //         unit: "kPa".to_string(),
        //         color: "#10b981".to_string(),
        //         data: vec![101.3, 102.0, 104.5, 108.2, 112.0, 115.4, 116.8, 118.0, 117.2, 115.0],
        //     },
        //     Signal {
        //         name: "Velocity".to_string(),
        //         unit: "m/s".to_string(),
        //         color: "#f59e0b".to_string(),
        //         data: vec![0.0, 1.2, 3.5, 6.8, 9.1, 10.5, 9.8, 7.2, 4.1, 1.0],
        //     },
        // ],
    };

    generate_html_report(&results, "multi_signal_report.html").expect("Failed to write report");

    println!("Multi-signal report generated successfully at multi_signal_report.html");
}