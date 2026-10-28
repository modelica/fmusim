use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use anyhow::Context;
use askama::Template;
use serde::Serialize;

pub mod fmi2;
pub mod fmi3;

/// One trace in a subplot (shared x / y scale with siblings in the same plot).
#[derive(Serialize)]
pub struct TimeSeries {
    pub name: String,
    /// When set (reference traces), x values for this series; simulation uses global `time`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<Vec<f64>>,
    pub fill: Option<String>,
    pub discrete: bool,
    pub dashed: bool,
    pub data: Vec<f64>,
}

/// Stacked subplot: one shared time axis and y scale, one or more series.
#[derive(Serialize)]
pub struct Plot {
    pub title: String,
    pub y_ticks: Option<Vec<(f64, String)>>,
    pub range: Option<(f64, f64)>,
    pub series: Vec<TimeSeries>,
}

#[derive(Serialize)]
pub struct SimulationResults {
    pub time: Vec<f64>,
    pub plots: Vec<Plot>,
    /// Times at state events (duplicate entries in `time`).
    pub events: Vec<f64>,
}

#[derive(Template)]
#[template(path = "multi_signal_report.html.askama")]
struct MultiSignalReportTemplate {
    plot_data_json: String,
}

pub fn render_html(results: &SimulationResults, output_path: impl AsRef<Path>) -> io::Result<()> {
    let plot_data_json = serde_json::to_string(results)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let html_content = MultiSignalReportTemplate { plot_data_json }
        .render()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    fs::write(output_path, html_content)
}

// pub fn render_html(results: SimulationResults) {
//     generate_html_report(&results, "multi_signal_report.html").expect("Failed to write report");

//     println!("Multi-signal report generated successfully at multi_signal_report.html");
// }

pub fn open_in_browser(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let path_str = path
        .as_ref()
        .to_str()
        .context("Plot path is not valid UTF-8 for opening in browser")?;

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", path_str])
            .spawn()
            .context("Failed to open plot in browser")?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path_str)
            .spawn()
            .context("Failed to open plot in browser")?;
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(path_str)
            .spawn()
            .context("Failed to open plot in browser")?;
    }

    Ok(())
}
