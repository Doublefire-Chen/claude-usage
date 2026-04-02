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

function formatTime(iso: string): string {
  return new Date(iso).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  });
}

export function UsageChart({ data }: Props) {
  if (data.length === 0) {
    return <p className="no-data">No usage data yet.</p>;
  }

  const chartData = data.map((d) => ({
    time: formatTime(d.timestamp),
    "5h": d.five_hour_usage,
    "7d": d.seven_day_usage,
    "7d Sonnet": d.seven_day_sonnet_usage,
  }));

  return (
    <div className="chart-container">
      <ResponsiveContainer width="100%" height={360}>
        <LineChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
          <XAxis dataKey="time" stroke="var(--text)" fontSize={12} />
          <YAxis
            domain={[0, 100]}
            unit="%"
            stroke="var(--text)"
            fontSize={12}
          />
          <Tooltip
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
