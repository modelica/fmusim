//! HTML plots using [uPlot](https://github.com/leeoniya/uPlot) (loaded from CDN in the generated file).

use std::ops::Range;
use std::path::Path;
use std::process::Command;

use anyhow::Context;
use serde::Serialize;

use crate::simulate::split_time_intervals_ranges;

pub const MAX_PLOTS: usize = 8;

const COLORS: [&str; 8] = [
    "#229AEB",
    "#FFA02C",
    "#33CC33",
    "#CC3333",
    "#9933CC",
    "#996633",
    "#FF66B2",
    "#999999",
];

const UPLOT_VERSION: &str = "1.6.24";

#[derive(Serialize)]
struct PlotDocument {
    title: String,
    log_x: bool,
    show_markers: bool,
    show_events: bool,
    events: Vec<f64>,
    panels: Vec<Panel>,
}

#[derive(Serialize)]
struct Panel {
    title: String,
    /// Shared time axis for all series in this panel (null breaks the line).
    x: Vec<Option<f64>>,
    series: Vec<Series>,
    y_min: Option<f64>,
    y_max: Option<f64>,
    y_values: Option<Vec<f64>>,
    y_labels: Option<Vec<String>>,
}

#[derive(Serialize)]
struct Series {
    label: String,
    color: String,
    width: f64,
    stepped: bool,
    dashed: bool,
    fill: bool,
    y: Vec<Option<f64>>,
}

pub fn show_html(path: &Path, html: &str) -> anyhow::Result<()> {
    std::fs::write(path, html)
        .with_context(|| format!("Failed to write plot file '{}'", path.display()))?;
    open_in_browser(path)
}

fn open_in_browser(path: &Path) -> anyhow::Result<()> {
    let path_str = path
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

fn aligned_series(time: &[f64], values: &[f64], ranges: &[Range<usize>]) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    let mut x = Vec::new();
    let mut y = Vec::new();
    for (i, range) in ranges.iter().enumerate() {
        if i > 0 {
            x.push(None);
            y.push(None);
        }
        for idx in range.clone() {
            if let (Some(t), Some(v)) = (time.get(idx), values.get(idx)) {
                x.push(Some(*t));
                y.push(Some(*v));
            }
        }
    }
    (x, y)
}

fn render_document(doc: &PlotDocument) -> anyhow::Result<String> {
    let json = serde_json::to_string(doc).context("Failed to serialize plot data")?;
    let css = format!(
        "https://cdn.jsdelivr.net/npm/uplot@{UPLOT_VERSION}/dist/uPlot.min.css"
    );
    let js = format!(
        "https://cdn.jsdelivr.net/npm/uplot@{UPLOT_VERSION}/dist/uPlot.iife.min.js"
    );

    Ok(format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1"/>
  <title>{title}</title>
  <link rel="stylesheet" href="{css}"/>
  <style>
    body {{ font-family: system-ui, sans-serif; margin: 0; padding: 12px; background: #fff; }}
    h1 {{ font-size: 1.1rem; font-weight: 600; margin: 0 0 12px 0; }}
    #charts {{ display: flex; flex-direction: column; gap: 0; }}
    .chart-wrap {{ width: 100%; }}
  </style>
</head>
<body>
  <h1>{title}</h1>
  <div id="charts"></div>
  <script src="{js}"></script>
  <script type="application/json" id="plot-data">{json}</script>
  <script>
(function () {{
  const doc = JSON.parse(document.getElementById("plot-data").textContent);
  const syncKey = "fmusim";
  const charts = document.getElementById("charts");
  const panelHeight = 220;
  const paths = uPlot.paths;
  const panelCount = doc.panels.length;

  function nullsToUplot(arr) {{
    return arr.map(v => (v === null ? null : v));
  }}

  function sharedXRange(panels) {{
    let min = Infinity;
    let max = -Infinity;
    for (const panel of panels) {{
      for (const v of panel.x) {{
        if (v != null && (!doc.log_x || v > 0)) {{
          min = Math.min(min, v);
          max = Math.max(max, v);
        }}
      }}
    }}
    if (!Number.isFinite(min) || !Number.isFinite(max)) {{
      return null;
    }}
    if (min === max) {{
      const pad = Math.abs(min) * 0.01 + 1e-9;
      return [min - pad, max + pad];
    }}
    return [min, max];
  }}

  const sharedX = sharedXRange(doc.panels);

  doc.panels.forEach((panel, pi) => {{
    const isBottom = pi === panelCount - 1;
    const wrap = document.createElement("div");
    wrap.className = "chart-wrap";
    const target = document.createElement("div");
    wrap.appendChild(target);
    charts.appendChild(wrap);

    const x = nullsToUplot(panel.x);
    const data = [x];
    const seriesOpts = [];
    panel.series.forEach((s) => {{
      if (s.y.length !== panel.x.length) {{
        console.warn("Skipping series with mismatched length:", s.label);
        return;
      }}
      data.push(nullsToUplot(s.y));
      const stroke = s.color;
      const seriesOpt = {{
        label: s.label,
        stroke,
        width: s.width,
        dash: s.dashed ? [6, 4] : null,
        points: {{ show: doc.show_markers, size: 5 }},
        fill: s.fill ? "rgba(240,248,255,0.5)" : null,
      }};
      if (s.stepped && paths.stepped) {{
        seriesOpt.paths = paths.stepped;
      }}
      seriesOpts.push(seriesOpt);
    }});

    const scales = {{
      x: {{
        time: false,
        ...(doc.log_x ? {{ distr: 3 }} : {{}}),
        ...(sharedX ? {{ range: sharedX }} : {{}}),
      }},
      y: {{
        auto: panel.y_min === null && panel.y_max === null,
        ...(panel.y_min !== null && panel.y_max !== null
          ? {{ range: [panel.y_min, panel.y_max] }}
          : {{}}),
      }},
    }};

    const axes = [
      {{
        scale: "x",
        stroke: "#d3d3d3",
        size: isBottom ? 36 : 0,
        show: true,
        grid: {{ show: true, stroke: "#d3d3d380" }},
        ticks: {{ show: isBottom }},
        values: isBottom ? undefined : () => [],
        label: isBottom ? "time [s]" : "",
      }},
      {{
        stroke: "#d3d3d3",
        grid: {{ show: true, stroke: "#d3d3d380" }},
        label: panel.title,
        size: 70,
        ...(panel.y_values && panel.y_labels
          ? {{ values: (u, vals) => panel.y_labels }}
          : {{}}),
      }},
    ];

    const eventTimes = doc.show_events ? doc.events : [];

    const opts = {{
      width: wrap.clientWidth || 900,
      height: panelHeight,
      title: "",
      scales,
      axes,
      series: [{{}}, ...seriesOpts],
      cursor: {{
        sync: {{
          key: syncKey,
          scales: ["x"],
        }},
        drag: {{
          x: true,
          y: false,
        }},
      }},
      hooks: {{
        draw: [
          (u) => {{
            if (!eventTimes.length) return;
            const {{ ctx }} = u;
            ctx.save();
            ctx.strokeStyle = "rgba(255, 215, 0, 0.9)";
            ctx.lineWidth = 1;
            for (const t of eventTimes) {{
              const x = u.valToPos(t, "x", true);
              if (x >= u.bbox.left && x <= u.bbox.left + u.bbox.width) {{
                ctx.beginPath();
                ctx.moveTo(x, u.bbox.top);
                ctx.lineTo(x, u.bbox.top + u.bbox.height);
                ctx.stroke();
              }}
            }}
            ctx.restore();
          }},
        ],
      }},
    }};

    const mount = () => {{
      opts.width = Math.max(wrap.clientWidth, 320);
      const u = new uPlot(opts, data, target);
      uPlot.sync(syncKey).sub(u);
      const ro = new ResizeObserver(() => {{
        u.setSize({{ width: Math.max(wrap.clientWidth, 320), height: panelHeight }});
      }});
      ro.observe(wrap);
    }};
    requestAnimationFrame(mount);
  }});
}})();
  </script>
</body>
</html>
"##,
        title = doc.title,
        css = css,
        js = js,
        json = json,
    ))
}

pub mod fmi2 {
    use fmi_rs::model_description::fmi2::{SimpleType, Variability, VariableType};
    use fmi_rs::sim::fmi2::Trajectories;

    use super::*;

    pub fn render_html(
        trajectories: &Trajectories,
        _reference: Option<&Trajectories>,
        show_markers: bool,
        show_events: bool,
    ) -> anyhow::Result<String> {
        let mut panels = Vec::new();

        for (variable_index, variable) in trajectories.variables().take(MAX_PLOTS).enumerate() {
            if matches!(variable.variableType, VariableType::String { .. }) {
                continue;
            }

            let mut axis_title = variable.name.clone();
            if let Some(unit) = trajectories.model_description.get_unit(variable) {
                axis_title.push_str(format!(" [{unit}]").as_str());
            }

            let (y_min, y_max, y_values, y_labels) =
                y_axis_hints_fmi2(trajectories, variable);
            if let VariableType::Enumeration { declaredType, .. } = &variable.variableType
                && trajectories
                    .model_description
                    .get_type_definition(declaredType)
                    .is_none()
            {
                continue;
            }

            let values: Vec<f64> = match trajectories
                .rows
                .iter()
                .map(|row| row.get(variable_index).and_then(|v| v.to_f64()))
                .collect()
            {
                Some(values) => values,
                None => continue,
            };

            let stepped = variable.variability == Variability::Discrete;
            let ranges = split_time_intervals_ranges(&trajectories.time);
            let (x, y) = aligned_series(&trajectories.time, &values, &ranges);

            let fill = matches!(variable.variableType, VariableType::Boolean { .. });

            let series = vec![Series {
                label: variable.name.clone(),
                color: COLORS[0].to_string(),
                width: 1.5,
                stepped,
                dashed: false,
                fill,
                y,
            }];

            panels.push(Panel {
                title: axis_title,
                x,
                series,
                y_min,
                y_max,
                y_values,
                y_labels,
            });
        }

        let doc = PlotDocument {
            title: trajectories.model_description.modelName.clone(),
            log_x: false,
            show_markers,
            show_events,
            events: trajectories.events(),
            panels,
        };

        render_document(&doc)
    }

    fn y_axis_hints_fmi2(
        trajectories: &Trajectories,
        variable: &fmi_rs::model_description::fmi2::ScalarVariable,
    ) -> (Option<f64>, Option<f64>, Option<Vec<f64>>, Option<Vec<String>>) {
        if matches!(variable.variableType, VariableType::Boolean { .. }) {
            return (
                Some(-0.01),
                Some(1.01),
                Some(vec![0.0, 1.0]),
                Some(vec!["false".to_string(), "true".to_string()]),
            );
        }

        if let VariableType::Enumeration { declaredType, .. } = &variable.variableType {
            if let Some(SimpleType::Enumeration { items, .. }) = trajectories
                .model_description
                .get_type_definition(declaredType)
            {
                let y_values: Vec<f64> = items.iter().map(|item| item.value as f64).collect();
                let y_labels: Vec<String> = items.iter().map(|item| item.name.clone()).collect();
                let y_min = items.iter().map(|i| i.value).min().map(|v| v as f64);
                let y_max = items.iter().map(|i| i.value).max().map(|v| v as f64);
                return (y_min, y_max, Some(y_values), Some(y_labels));
            }
        }

        (None, None, None, None)
    }
}

pub mod fmi3 {
    use fmi_rs::model_description::fmi3::{TypeDefinition, VariableType};
    use fmi_rs::sim::fmi3::Trajectories;

    use super::*;

    pub fn render_html(
        trajectories: &Trajectories,
        ref_trajectories: Option<&Trajectories>,
        show_markers: bool,
        show_events: bool,
        log_x: bool,
    ) -> anyhow::Result<String> {
        let mut panels = Vec::new();

        for (variable_index, variable) in trajectories.variables().take(MAX_PLOTS).enumerate() {
            if matches!(
                variable.variableType,
                VariableType::String { .. } | VariableType::Binary { .. }
            ) {
                continue;
            }

            let mut axis_title = variable.name.clone();
            if let Some(unit) = trajectories.model_description.get_unit(variable) {
                axis_title.push_str(format!(" [{unit}]").as_str());
            }

            let (y_min, y_max, y_values, y_labels) =
                y_axis_hints_fmi3(trajectories, variable);
            if let VariableType::Enumeration { declaredType, .. } = &variable.variableType
                && trajectories
                    .model_description
                    .get_type_definition(declaredType)
                    .is_none()
            {
                continue;
            }

            let time = trajectories.time.clone();
            let name = variable.name.clone();
            let ranges = split_time_intervals_ranges(&time);

            if let Some(first_row) = trajectories.rows.first()
                && let Some(value) = first_row.get(variable_index)
            {
                let size = value.len();
                let trajectory: Vec<Vec<f64>> = match trajectories
                    .rows
                    .iter()
                    .map(|row| row.get(variable_index).and_then(|v| v.as_f64()))
                    .collect()
                {
                    Some(variable_values) => variable_values,
                    None => continue,
                };

                let mut color_iter = COLORS.iter().cycle();
                let mut panel_x: Option<Vec<Option<f64>>> = None;
                let mut series_list = Vec::new();

                for j in 0..size {
                    let scalar_values: Vec<f64> =
                        match trajectory.iter().map(|v| v.get(j).cloned()).collect() {
                            Some(values) => values,
                            None => continue,
                        };

                    let label = if size > 1 {
                        format!("{}[{}]", name, j)
                    } else {
                        name.clone()
                    };

                    let current_color = color_iter.next().unwrap_or(&COLORS[0]);

                    let (x, y) = aligned_series(&time, &scalar_values, &ranges);
                    if panel_x.is_none() {
                        panel_x = Some(x);
                    }
                    let fill = matches!(variable.variableType, VariableType::Boolean { .. });

                    series_list.push(Series {
                        label: label.clone(),
                        color: (*current_color).to_string(),
                        width: 1.5,
                        stepped: false,
                        dashed: false,
                        fill,
                        y,
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

                        if let Some(scalar_ref_values) = scalar_ref_values {
                            let ref_y = if ref_trajectories.time == time {
                                let ref_ranges =
                                    split_time_intervals_ranges(&ref_trajectories.time);
                                aligned_series(&time, &scalar_ref_values, &ref_ranges).1
                            } else if ref_trajectories.time.len() == scalar_ref_values.len() {
                                aligned_series(&time, &scalar_ref_values, &ranges).1
                            } else {
                                continue;
                            };
                            if ref_y.len() == panel_x.as_ref().map(Vec::len).unwrap_or(0) {
                                series_list.push(Series {
                                    label: format!("{label} (reference)"),
                                    color: format!("{current_color}66"),
                                    width: 3.0,
                                    stepped: false,
                                    dashed: true,
                                    fill: false,
                                    y: ref_y,
                                });
                            }
                        }
                    }
                }

                if let Some(x) = panel_x
                    && !series_list.is_empty()
                {
                    panels.push(Panel {
                        title: axis_title,
                        x,
                        series: series_list,
                        y_min,
                        y_max,
                        y_values,
                        y_labels,
                    });
                }
            }
        }

        let doc = PlotDocument {
            title: trajectories.model_description.modelName.clone(),
            log_x,
            show_markers,
            show_events,
            events: trajectories.events(),
            panels,
        };

        render_document(&doc)
    }

    fn y_axis_hints_fmi3(
        trajectories: &Trajectories,
        variable: &fmi_rs::model_description::fmi3::ModelVariable,
    ) -> (Option<f64>, Option<f64>, Option<Vec<f64>>, Option<Vec<String>>) {
        if matches!(variable.variableType, VariableType::Boolean { .. }) {
            return (
                Some(-0.01),
                Some(1.01),
                Some(vec![0.0, 1.0]),
                Some(vec!["false".to_string(), "true".to_string()]),
            );
        }

        if let VariableType::Enumeration { declaredType, .. } = &variable.variableType {
            if let Some(TypeDefinition::Enumeration { items, .. }) = trajectories
                .model_description
                .get_type_definition(declaredType)
            {
                let y_values: Vec<f64> = items.iter().map(|item| item.value as f64).collect();
                let y_labels: Vec<String> = items.iter().map(|item| item.name.clone()).collect();
                let y_min = items.iter().map(|i| i.value).min().map(|v| v as f64);
                let y_max = items.iter().map(|i| i.value).max().map(|v| v as f64);
                return (y_min, y_max, Some(y_values), Some(y_labels));
            }
        }

        (None, None, None, None)
    }
}
