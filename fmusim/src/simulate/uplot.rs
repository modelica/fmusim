use std::fs;
use std::io;

use askama::Template;
use fmi_rs::sim::fmi2::Trajectories;
use serde::Serialize;

#[derive(Serialize)]
pub struct Signal {
    pub name: String,
    pub color: String,
    pub fill: Option<String>,
    pub discrete: bool,
    pub data: Vec<f64>,
    pub y_ticks: Option<Vec<(f64, String)>>,
    pub range: Option<(f64, f64)>,
}

#[derive(Serialize)]
pub struct SimulationResults {
    pub time: Vec<f64>,
    pub signals: Vec<Signal>,
    /// Times at state events (duplicate entries in `time`).
    pub events: Vec<f64>,
}

#[derive(Template)]
#[template(path = "multi_signal_report.html.askama")]
struct MultiSignalReportTemplate {
    plot_data_json: String,
}

fn generate_html_report(results: &SimulationResults, output_path: &str) -> io::Result<()> {
    let plot_data_json =
        serde_json::to_string(results).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let html_content = MultiSignalReportTemplate { plot_data_json }
    .render()
    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    fs::write(output_path, html_content)
}

pub fn render_html(results: SimulationResults) {

    generate_html_report(&results, "multi_signal_report.html").expect("Failed to write report");

    println!("Multi-signal report generated successfully at multi_signal_report.html");
}