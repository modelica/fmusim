(function () {
  const { time: timeValues, plots, events: eventTimes = [] } = JSON.parse(
    document.getElementById("plot-data").textContent
  );
  const numPlots = plots.length;

  const container = document.getElementById("container");
  const sync = uPlot.sync("multi-signal-plots");
  const Y_AXIS_WIDTH = 70;
  const X_AXIS_SIZE = 35;
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
    const h = Math.max(80, (totalInnerHeight - (numPlots * PADDING_V) - X_AXIS_SIZE) / numPlots);

    return plots.map((_, i) => {
      let plotHeight = h + PADDING_V;
      if (i === numPlots - 1) {
        plotHeight += X_AXIS_SIZE;
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
      const colorIdx = s.dashed
        ? Math.max(0, seriesColorIndex - 1)
        : seriesColorIndex;
      const baseColor = SERIES_COLORS[colorIdx % SERIES_COLORS.length];
      if (!s.dashed) {
        seriesColorIndex += 1;
      }
      const seriesConfig = {
        label: s.name,
        stroke: s.dashed
          ? seriesColor(baseColor, REFERENCE_ALPHA)
          : baseColor,
        fill: s.dashed ? null : s.fill,
        width: s.dashed ? 5 : 2,
        dash: null,
      };
      if (s.discrete && !s.dashed) {
        seriesConfig.paths = uPlot.paths.stepped({ align: 1 });
      }
      return seriesConfig;
    });

    const scales = { x: { time: false } };

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
            space: 40,
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

    const data = [timeValues, ...plot.series.map((s) => s.data)];

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
