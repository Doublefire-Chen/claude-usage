import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";
import type { UsageSnapshot } from "./api";

interface Props {
  data: UsageSnapshot[];
}

function formatLabel(iso: string): string {
  const d = new Date(iso);
  const month = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  const hour = String(d.getHours()).padStart(2, "0");
  const min = String(d.getMinutes()).padStart(2, "0");
  return `${month}-${day} ${hour}:${min}`;
}

export function UsageChart({ data }: Props) {
  if (data.length === 0) {
    return <p className="no-data">No usage data yet.</p>;
  }

  const chartData = data.map((d, i) => ({
    idx: i,
    label: formatLabel(d.timestamp),
    "5h": d.five_hour_usage,
    "7d": d.seven_day_usage,
    "7d Sonnet": d.seven_day_sonnet_usage,
  }));

  // Show ~10 evenly spaced tick labels
  const step = Math.max(1, Math.floor(chartData.length / 10));
  const ticks = chartData
    .filter((_, i) => i % step === 0)
    .map((d) => d.idx);

  return (
    <div className="chart-container">
      <ResponsiveContainer width="100%" height={360}>
        <LineChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
          <XAxis
            dataKey="idx"
            type="number"
            domain={[0, chartData.length - 1]}
            ticks={ticks}
            tickFormatter={(idx: number) => chartData[idx]?.label ?? ""}
            stroke="var(--text)"
            fontSize={12}
          />
          <YAxis
            domain={[0, 100]}
            unit="%"
            stroke="var(--text)"
            fontSize={12}
          />
          <Tooltip
            labelFormatter={(idx: number) => chartData[idx]?.label ?? ""}
            contentStyle={{
              background: "var(--card-bg)",
              border: "1px solid var(--border)",
              borderRadius: "8px",
            }}
          />
          <Legend />
          <Line
            type="monotone"
            dataKey="5h"
            stroke="#c15f3c"
            strokeWidth={2}
            dot={false}
          />
          <Line
            type="monotone"
            dataKey="7d"
            stroke="#d97757"
            strokeWidth={2}
            dot={false}
          />
          <Line
            type="monotone"
            dataKey="7d Sonnet"
            stroke="#a14a2f"
            strokeWidth={2}
            dot={false}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
