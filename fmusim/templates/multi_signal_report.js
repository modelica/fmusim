(function () {
  const { time: timeValues, signals } = JSON.parse(
    document.getElementById("plot-data").textContent
  );
  const numPlots = signals.length;

  const container = document.getElementById("container");
  const sync = uPlot.sync("multi-signal-plots");
  const Y_AXIS_WIDTH = 70;
  const X_AXIS_SIZE = 35;
  const PADDING_V = 20;
  const GAP_SIZE = 12;
  const MIN_PLOT_WIDTH = 320;

  const wrappers = [];
  const chartDivs = [];
  const plots = [];

  signals.forEach((sig, i) => {
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

    return signals.map((_, i) => {
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

  signals.forEach((sig, i) => {
    const isLast = i === numPlots - 1;
    wrappers[i].style.flex = `${currentHeights[i]}`;

    let valueMap = {};
    let splitValues = [];

    if (sig.y_ticks) {
      sig.y_ticks.forEach(([value, label]) => {
        valueMap[value] = label;
        splitValues.push(value);
      });
    }

    const seriesConfig = {
      label: sig.name,
      stroke: sig.color,
      fill: sig.fill,
      width: 2,
    };

    const scales = { x: { time: false } };

    if (sig.range) {
      scales.y = { range: () => sig.range };
    }

    if (sig.discrete) {
      seriesConfig.paths = uPlot.paths.stepped({ align: 1 });
    }

    const yAxis = {
      label: `${sig.name}`,
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

    const opts = {
      width: currentWidth,
      height: currentHeights[i],
      padding: [10, 20, 10, 15],
      scales: scales,
      legend: { show: false },
      cursor: { sync: { key: sync.key } },
      axes: axes,
      series: [{}, seriesConfig],
    };

    const u = new uPlot(opts, [timeValues, sig.data], chartDivs[i]);
    plots.push(u);
  });

  const resizeObserver = new ResizeObserver(() => {
    const newWidth = getWidth();
    const newHeights = getHeights();

    if (newWidth > 0) {
      signals.forEach((_, i) => {
        wrappers[i].style.flex = `${newHeights[i]}`;
        plots[i].setSize({ width: newWidth, height: newHeights[i] });
      });
    }
  });

  resizeObserver.observe(container);
})();
