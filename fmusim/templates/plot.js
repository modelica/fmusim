(function () {
  const { time: timeValues, plots, events: eventTimes = [], log_x: logX = false } =
    JSON.parse(document.getElementById("plot-data").textContent);
  const numPlots = plots.length;

  const container = document.getElementById("container");
  const sync = uPlot.sync("fmusim-plot");
  const Y_AXIS_WIDTH = 70;
  const X_AXIS_SIZE = 35;
  const X_AXIS_LABEL_SPACE = 40;
  const BOTTOM_X_RESERVE = X_AXIS_SIZE + X_AXIS_LABEL_SPACE;
  const PADDING_V = 20;
  const GAP_SIZE = 12;
  const MIN_PLOT_WIDTH = 320;
  const SERIES_COLORS = [
    "#229AEB",
    "#FFA02C",
    "#33CC33",
    "#CC3333",
    "#9933CC",
    "#996633",
    "#FF66B2",
    "#999999",
  ];

  const REFERENCE_ALPHA = 0.45;

  function seriesColor(hex, alpha) {
    const h = hex.replace("#", "");
    const r = parseInt(h.slice(0, 2), 16);
    const g = parseInt(h.slice(2, 4), 16);
    const b = parseInt(h.slice(4, 6), 16);
    return `rgba(${r}, ${g}, ${b}, ${alpha})`;
  }

  function buildPlotData(globalTime, series) {
    const tables = series.map((s) => {
      let x = s.time ?? globalTime;
      let y = s.data;
      if (logX) {
        const fx = [];
        const fy = [];
        for (let i = 0; i < x.length; i++) {
          if (x[i] > 0) {
            fx.push(x[i]);
            fy.push(y[i]);
          }
        }
        x = fx;
        y = fy;
      }
      return [x, y];
    });
    const data = uPlot.join(tables);
    for (let i = 1; i < data.length; i++) {
      data[i] = data[i].map((v) => (v === undefined ? null : v));
    }
    return data;
  }

  const wrappers = [];
  const chartDivs = [];
  const uplots = [];

  plots.forEach((_, i) => {
    const wrapper = document.createElement("div");
    wrapper.className = "chart-wrapper";

    const chartDiv = document.createElement("div");
    chartDiv.id = `chart-${i}`;

    wrapper.appendChild(chartDiv);
    container.appendChild(wrapper);

    wrappers.push(wrapper);
    chartDivs.push(chartDiv);
  });

  function getHeights() {
    const totalInnerHeight = container.clientHeight - 32 - ((numPlots - 1) * GAP_SIZE);
    const h = Math.max(
      80,
      (totalInnerHeight - numPlots * PADDING_V - BOTTOM_X_RESERVE) / numPlots
    );

    return plots.map((_, i) => {
      let plotHeight = h + PADDING_V;
      if (i === numPlots - 1) {
        plotHeight += BOTTOM_X_RESERVE;
      }
      return Math.floor(plotHeight);
    });
  }

  function getWidth() {
    const inner = container.clientWidth - 32;
    return Math.max(MIN_PLOT_WIDTH, inner);
  }

  let currentWidth = getWidth();
  let currentHeights = getHeights();

  plots.forEach((plot, i) => {
    const isLast = i === numPlots - 1;
    wrappers[i].style.flex = `${currentHeights[i]}`;

    let valueMap = {};
    let splitValues = [];

    if (plot.y_ticks) {
      plot.y_ticks.forEach(([value, label]) => {
        valueMap[value] = label;
        splitValues.push(value);
      });
    }

    let seriesColorIndex = 0;
    const seriesOpts = plot.series.map((s) => {
      const colorIdx = s.reference
        ? Math.max(0, seriesColorIndex - 1)
        : seriesColorIndex;
      const baseColor = SERIES_COLORS[colorIdx % SERIES_COLORS.length];
      if (!s.reference) {
        seriesColorIndex += 1;
      }
      const seriesConfig = {
        label: s.name,
        stroke: s.reference
          ? seriesColor(baseColor, REFERENCE_ALPHA)
          : baseColor,
        fill: s.reference ? null : s.fill,
        width: s.reference ? 6 : 2,
        dash: null,
      };
      if (s.discrete && !s.reference) {
        seriesConfig.paths = uPlot.paths.stepped({ align: 1 });
      }
      return seriesConfig;
    });

    const scales = {
      x: {
        time: false,
        ...(logX ? { distr: 3 } : {}),
      },
    };

    if (plot.range) {
      scales.y = { range: () => plot.range };
    }

    const yAxis = {
      label: `${plot.title}`,
      size: Y_AXIS_WIDTH,
      grid: { width: 1, stroke: "#eee" },
      ticks: { show: false },
    };

    if (splitValues.length > 0) {
      yAxis.splits = () => splitValues;
      yAxis.values = (u, splits) => splits.map((v) => valueMap[v] ?? v);
    }

    const axes = [
      isLast
        ? {
            size: X_AXIS_SIZE,
            space: X_AXIS_LABEL_SPACE,
            label: "time [s]",
            grid: { width: 1, stroke: "#eee" },
            ticks: { show: false },
          }
        : {
            size: 0,
            grid: { show: true, width: 1, stroke: "#eee" },
            ticks: { show: false },
            values: () => [],
          },
      yAxis,
    ];

    const data = buildPlotData(timeValues, plot.series);

    const opts = {
      width: currentWidth,
      height: currentHeights[i],
      padding: [10, 20, 10, 15],
      scales: scales,
      legend: { show: false },
      cursor: { sync: { key: sync.key } },
      axes: axes,
      series: [{}, ...seriesOpts],
      hooks: {
        drawClear: [
          (u) => {
            if (!eventTimes.length) return;
            const { ctx } = u;
            ctx.save();
            ctx.strokeStyle = "rgba(255, 215, 0, 0.9)";
            ctx.lineWidth = 1;
            for (const t of eventTimes) {
              if (logX && t <= 0) continue;
              const x = u.valToPos(t, "x", true);
              if (x >= u.bbox.left && x <= u.bbox.left + u.bbox.width) {
                ctx.beginPath();
                ctx.moveTo(x, u.bbox.top);
                ctx.lineTo(x, u.bbox.top + u.bbox.height);
                ctx.stroke();
              }
            }
            ctx.restore();
          },
        ],
      },
    };

    const u = new uPlot(opts, data, chartDivs[i]);
    uplots.push(u);
  });

  const resizeObserver = new ResizeObserver(() => {
    const newWidth = getWidth();
    const newHeights = getHeights();

    if (newWidth > 0) {
      plots.forEach((_, i) => {
        wrappers[i].style.flex = `${newHeights[i]}`;
        uplots[i].setSize({ width: newWidth, height: newHeights[i] });
      });
    }
  });

  resizeObserver.observe(container);
})();
