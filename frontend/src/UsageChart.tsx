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
  range: string;
}

function formatLabel(iso: string, range: string): string {
  const d = new Date(iso);
  const hour = String(d.getHours()).padStart(2, "0");
  const min = String(d.getMinutes()).padStart(2, "0");
  const month = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  if (range === "24h") return `${hour}:${min}`;
  if (range === "7d") return `${month}-${day}`;
  return `${month}-${day}`;
}

export function UsageChart({ data, range }: Props) {
  if (data.length === 0) {
    return <p className="no-data">No usage data yet.</p>;
  }

  const chartData = data.map((d, i) => ({
    idx: i,
    label: formatLabel(d.timestamp, range),
    "5h": d.five_hour_usage,
    "7d": d.seven_day_usage,
    "7d Sonnet": d.seven_day_sonnet_usage,
  }));

  // Pick ticks at round time boundaries
  const ticks: number[] = [];
  const seen = new Set<string>();
  for (const d of data) {
    const date = new Date(d.timestamp);
    let key: string;
    if (range === "24h") {
      // Every whole hour
      key = `${date.getHours()}`;
      if (date.getMinutes() > 15) continue;
    } else if (range === "7d") {
      // Once per day at ~00:00
      key = `${date.getMonth()}-${date.getDate()}`;
      if (date.getHours() > 1) continue;
    } else {
      // Every Monday at ~00:00
      if (date.getDay() !== 1) continue;
      key = `${date.getMonth()}-${date.getDate()}`;
      if (date.getHours() > 1) continue;
    }
    if (!seen.has(key)) {
      seen.add(key);
      ticks.push(data.indexOf(d));
    }
  }

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
