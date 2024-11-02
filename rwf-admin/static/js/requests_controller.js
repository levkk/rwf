import { Controller } from "hotwired/stimulus";
import "https://cdn.jsdelivr.net/npm/chart.js";

export default class extends Controller {
  static targets = ["requestsOk", "chart"];

  connect() {
    const data = JSON.parse(this.requestsOkTarget.innerHTML);
    const labels = Array.from(
      new Set(
        data.map((item) => new Date(item.created_at).toLocaleTimeString()),
      ),
    );
    const ok = data
      .filter((item) => item.code === "ok")
      .map((item) => item.count);
    const warn = data
      .filter((item) => item.code === "warn")
      .map((item) => item.count);
    const error = data
      .filter((item) => item.code === "error")
      .map((item) => item.count);

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
        },
      },
    };

    const chartData = {
      labels,
      datasets: [
        {
          label: "100-299",
          data: ok,
        },
        {
          label: "500-599",
          data: error,
          // backgroundColor: "red",
        },
        {
          label: "300-499",
          data: warn,
        },
      ],
    };

    new Chart(this.chartTarget, {
      type: "bar",
      data: chartData,
      options,
    });
  }
}
