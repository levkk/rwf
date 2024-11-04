import { Controller } from "hotwired/stimulus";
import "https://cdn.jsdelivr.net/npm/chart.js";

export default class extends Controller {
  static targets = ["requests", "chart", "duration"];

  connect() {
    const requestsData = JSON.parse(this.requestsTarget.innerHTML);
    const latencyData = JSON.parse(this.durationTarget.innerHTML);
    const labels = Array.from(
      new Set(
        requestsData.map((item) => new Date(item.created_at).toLocaleTimeString()),
      ),
    );
    const ok = requestsData
      .filter((item) => item.code === "ok")
      .map((item) => item.count);
    const warn = requestsData
      .filter((item) => item.code === "warn")
      .map((item) => item.count);
    const error = requestsData
      .filter((item) => item.code === "error")
      .map((item) => item.count);
    const latencyX = latencyData.map((item) => item.duration);

    const options = {
      scales: {
        x: {
          ticks: {
            callback: (t, i) => (i % 10 === 0 ? labels[i] : null),
          },
          stacked: true,
        },
        y: {
          stacked: true,
          position: 'left',
        },

        y1: {
          position: 'right',
          display: true,
          grid: {
            drawOnChartArea: false, // only want the grid lines for one axis to show up
          },
        }
      },
    };

    const chartData = {
      labels,
      datasets: [
        {
          label: "100-299",
          data: ok,
          yAxisID: 'y',
        },
        {
          label: "500-599",
          data: error,
          yAxisID: 'y',
        },
        {
          label: "300-499",
          data: warn,
          yAxisID: 'y',
        },
        {
          label: "Latency (ms)",
          data: latencyX,
          yAxisID: "y1",
          type: "line",
          hidden: true,
        }
      ],
    };

    new Chart(this.chartTarget, {
      type: "bar",
      data: chartData,
      options,
    });
  }
}
