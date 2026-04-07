import { useState } from "react";
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

const LINES = [
  { key: "5h", label: "5h", color: "#4889f4" },
  { key: "7d", label: "7d", color: "#d97757" },
  { key: "7d Sonnet", label: "7d Sonnet", color: "#22c55e" },
];

function formatLabel(iso: string, range: string): string {
  const d = new Date(iso);
  const hour = String(d.getHours()).padStart(2, "0");
  const min = String(d.getMinutes()).padStart(2, "0");
  const month = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  if (range === "24h") return `${hour}:${min}`;
  return `${month}-${day}`;
}

export function UsageChart({ data, range }: Props) {
  const [hidden, setHidden] = useState<Set<string>>(new Set());

  if (data.length === 0) {
    return <p className="no-data">No usage data yet.</p>;
  }

  const chartData = data.map((d, i) => {
    const date = new Date(d.timestamp);
    const mm = String(date.getMonth() + 1).padStart(2, "0");
    const dd = String(date.getDate()).padStart(2, "0");
    const hh = String(date.getHours()).padStart(2, "0");
    const mi = String(date.getMinutes()).padStart(2, "0");
    return {
      idx: i,
      label: formatLabel(d.timestamp, range),
      tooltipLabel: `${mm}-${dd} ${hh}:${mi}`,
      "5h": d.five_hour_usage,
      "7d": d.seven_day_usage,
      "7d Sonnet": d.seven_day_sonnet_usage,
    };
  });

  // Pick ticks at round time boundaries
  const ticks: number[] = [];
  const seen = new Set<string>();
  for (const d of data) {
    const date = new Date(d.timestamp);
    let key: string;
    if (range === "24h") {
      key = `${date.getHours()}`;
      if (date.getMinutes() > 15) continue;
    } else if (range === "7d") {
      key = `${date.getMonth()}-${date.getDate()}`;
      if (date.getHours() > 1) continue;
    } else {
      if (date.getDay() !== 1) continue;
      key = `${date.getMonth()}-${date.getDate()}`;
      if (date.getHours() > 1) continue;
    }
    if (!seen.has(key)) {
      seen.add(key);
      ticks.push(data.indexOf(d));
    }
  }

  const toggleLine = (dataKey: string) => {
    setHidden((prev) => {
      const next = new Set(prev);
      if (next.has(dataKey)) {
        next.delete(dataKey);
      } else {
        next.add(dataKey);
      }
      return next;
    });
  };

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
            labelFormatter={(idx: number) => chartData[idx]?.tooltipLabel ?? ""}
            contentStyle={{
              background: "var(--card-bg)",
              border: "1px solid var(--border)",
              borderRadius: "8px",
            }}
          />
          <Legend
            content={() => (
              <div className="chart-legend">
                {LINES.map((line) => (
                  <label key={line.key} className="chart-legend-item">
                    <input
                      type="checkbox"
                      checked={!hidden.has(line.key)}
                      onChange={() => toggleLine(line.key)}
                    />
                    <svg width="20" height="10" style={{ marginRight: 4 }}>
                      <line x1="0" y1="5" x2="20" y2="5" stroke={line.color} strokeWidth="2" />
                      <circle cx="10" cy="5" r="3" fill={line.color} />
                    </svg>
                    <span style={{ color: hidden.has(line.key) ? "var(--border)" : "var(--text)" }}>
                      {line.label}
                    </span>
                  </label>
                ))}
              </div>
            )}
          />
          {LINES.map((line) => (
            <Line
              key={line.key}
              type="monotone"
              dataKey={line.key}
              stroke={line.color}
              strokeWidth={2}
              dot={false}
              hide={hidden.has(line.key)}
            />
          ))}
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
