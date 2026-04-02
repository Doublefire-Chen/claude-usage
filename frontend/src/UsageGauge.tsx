interface Props {
  label: string;
  value: number | null;
  resetsAt: string | null;
}

function formatResetTime(iso: string): string {
  const d = new Date(iso);
  const now = new Date();
  const diffMs = d.getTime() - now.getTime();
  if (diffMs <= 0) return "resetting...";
  const hours = Math.floor(diffMs / 3_600_000);
  const mins = Math.floor((diffMs % 3_600_000) / 60_000);
  if (hours > 0) return `resets in ${hours}h ${mins}m`;
  return `resets in ${mins}m`;
}

function gaugeColor(value: number): string {
  if (value >= 80) return "var(--color-danger)";
  if (value >= 50) return "var(--color-warning)";
  return "var(--color-ok)";
}

export function UsageGauge({ label, value, resetsAt }: Props) {
  if (value === null || value === undefined) {
    return (
      <div className="gauge">
        <div className="gauge-label">{label}</div>
        <div className="gauge-value">N/A</div>
      </div>
    );
  }

  const pct = Math.min(value, 100);

  return (
    <div className="gauge">
      <div className="gauge-label">{label}</div>
      <div className="gauge-bar-track">
        <div
          className="gauge-bar-fill"
          style={{ width: `${pct}%`, backgroundColor: gaugeColor(value) }}
        />
      </div>
      <div className="gauge-value" style={{ color: gaugeColor(value) }}>
        {value.toFixed(1)}%
      </div>
      {resetsAt && (
        <div className="gauge-reset">{formatResetTime(resetsAt)}</div>
      )}
    </div>
  );
}
