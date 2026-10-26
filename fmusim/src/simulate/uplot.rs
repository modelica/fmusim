use std::fs;
use std::io;
use fmi_rs::model_description::fmi2::VariableIndex;
use fmi_rs::sim::fmi2::Trajectories;
use serde::Serialize;

#[derive(Serialize)]
struct Signal {
    name: String,
    unit: String,
    color: String,
    data: Vec<f64>,
}

#[derive(Serialize)]
struct SimulationResults {
    time: Vec<f64>,
    signals: Vec<Signal>,
}

fn generate_html_report(results: &SimulationResults, output_path: &str) -> io::Result<()> {
    // Serialize Rust data structures directly to JSON strings
    let time_json = serde_json::to_string(&results.time).unwrap();
    let signals_json = serde_json::to_string(&results.signals).unwrap();

    let html_content = format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Dynamic Multi-Signal Simulation Report</title>
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/uplot@1.6.25/dist/uPlot.min.css">
  <script src="https://cdn.jsdelivr.net/npm/uplot@1.6.25/dist/uPlot.iife.min.js"></script>
  <style>
    html, body {{
      height: 100%;
      margin: 0;
      padding: 20px;
      box-sizing: border-box;
      font-family: system-ui, -apple-system, sans-serif;
      background: #f4f4f9;
    }}
    .chart-container {{
      display: flex;
      flex-direction: column;
      gap: 12px;
      background: #ffffff;
      padding: 16px;
      border-radius: 8px;
      width: 100%;
      min-height: 500px;
      height: calc(100vh - 80px);
      box-sizing: border-box;
    }}
    .chart-wrapper {{
      display: flex;
      flex-direction: column;
    }}
    .chart-wrapper > div {{
      flex: 1;
    }}
  </style>
</head>
<body>

  <div class="chart-container" id="container"></div>

  <script>
    // Injected dynamically from Rust
    const timeValues = {time_json};
    const signals = {signals_json};
    const numPlots = signals.length;

    const container = document.getElementById("container");
    const sync = uPlot.sync("multi-signal-plots");
    const Y_AXIS_WIDTH = 70;
    const X_AXIS_SIZE = 35;
    const PADDING_V = 20;
    const GAP_SIZE = 12;

    // Dynamically create DOM wrappers and chart divs for each signal
    const wrappers = [];
    const chartDivs = [];
    const plots = [];

    signals.forEach((sig, i) => {{
      const wrapper = document.createElement("div");
      wrapper.className = "chart-wrapper";
      
      const chartDiv = document.createElement("div");
      chartDiv.id = `chart-${{i}}`;
      
      wrapper.appendChild(chartDiv);
      container.appendChild(wrapper);
      
      wrappers.push(wrapper);
      chartDivs.push(chartDiv);
    }});

    // Calculate height distributions proportional to the number of signals
    function getHeights() {{
      const totalInnerHeight = container.clientHeight - 32 - ((numPlots - 1) * GAP_SIZE);
      const h = Math.max(80, (totalInnerHeight - (numPlots * PADDING_V) - X_AXIS_SIZE) / numPlots);

      return signals.map((_, i) => {{
        let plotHeight = h + PADDING_V;
        if (i === numPlots - 1) {{
          plotHeight += X_AXIS_SIZE; // Bottom plot includes space for x-axis labels/title
        }}
        return Math.floor(plotHeight);
      }});
    }}

    function getWidth() {{
      return container.clientWidth - 32;
    }}

    let currentWidth = getWidth();
    let currentHeights = getHeights();

    // Initialize plots
    signals.forEach((sig, i) => {{
      const isLast = i === numPlots - 1;
      wrappers[i].style.flex = `${{currentHeights[i]}}`;

      const opts = {{
        width: currentWidth,
        height: currentHeights[i],
        padding: [10, 20, 10, 15],
        scales: {{ x: {{ time: false }} }},
        legend: {{ show: false }},
        cursor: {{ sync: {{ key: sync.key }} }},
        axes: [
          isLast ? {{
            size: X_AXIS_SIZE,
            space: 40,
            label: "Time (s)",
            grid: {{ width: 1, stroke: "#eee" }},
            ticks: {{ show: false }}
          }} : {{
            size: 0,
            grid: {{ show: true, width: 1, stroke: "#eee" }},
            ticks: {{ show: false }},
            values: () => []
          }},
          {{
            label: `${{sig.name}} (${{sig.unit}})`,
            size: Y_AXIS_WIDTH,
            grid: {{ width: 1, stroke: "#eee" }},
            ticks: {{ show: false }}
          }}
        ],
        series: [
          {{}},
          {{
            label: sig.name,
            stroke: sig.color,
            width: 2
          }}
        ]
      }};

      const u = new uPlot(opts, [timeValues, sig.data], chartDivs[i]);
      plots.push(u);
    }});

    // Handle responsive resizing
    const resizeObserver = new ResizeObserver(() => {{
      const newWidth = getWidth();
      const newHeights = getHeights();
      
      if (newWidth > 0) {{
        signals.forEach((_, i) => {{
          wrappers[i].style.flex = `${{newHeights[i]}}`;
          plots[i].setSize({{ width: newWidth, height: newHeights[i] }});
        }});
      }}
    }});

    resizeObserver.observe(container);
  </script>

</body>
</html>
"##,
        time_json = time_json,
        signals_json = signals_json
    );

    fs::write(output_path, html_content)
}

pub fn render_html(trajectories: &Trajectories) {

    let mut signals = vec![];

    for (i, variable) in trajectories.variables().enumerate() {
      if let Some(data) = trajectories.trajectory_by_index(i) {
        let values: Vec<f64> = data.iter().map(|v| v.to_f64().unwrap_or_default()).collect();
        
        let signal = Signal {
          name: variable.name.clone(),
          unit: "?".to_string(),
          color: "#3b82f6".to_string(),
          data: values,
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