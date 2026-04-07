interface Props {
  label: string;
  value: number | null;
  resetsAt: string | null;
}

function formatResetTime(iso: string): string {
  const d = new Date(iso);
  const now = new Date();
  const diffMs = d.getTime() - now.getTime();
  if (diffMs <= 0) return "Resetting...";
  const days = Math.floor(diffMs / 86_400_000);
  const hours = Math.floor((diffMs % 86_400_000) / 3_600_000);
  const mins = Math.floor((diffMs % 3_600_000) / 60_000);
  if (days > 0) return `Resets in ${days} d ${hours} hr ${mins} min`;
  if (hours > 0) return `Resets in ${hours} hr ${mins} min`;
  return `Resets in ${mins} min`;
}

export function UsageGauge({ label, value, resetsAt }: Props) {
  const pct = value !== null && value !== undefined ? Math.min(value, 100) : null;

  return (
    <div className="gauge-row">
      <div className="gauge-info">
        <p className="gauge-label">{label}</p>
        {resetsAt ? (
          <p className="gauge-reset">{formatResetTime(resetsAt)}</p>
        ) : (
          <p className="gauge-reset">&nbsp;</p>
        )}
      </div>
      <div className="gauge-bar-wrapper">
        <div className="gauge-bar-track">
          <div
            className="gauge-bar-fill"
            style={{ width: pct !== null ? `${Math.max(pct, 1)}%` : "0%" }}
          />
        </div>
        <p className="gauge-value">
          {pct !== null ? `${value!.toFixed(0)}% used` : "N/A"}
        </p>
      </div>
    </div>
  );
}
