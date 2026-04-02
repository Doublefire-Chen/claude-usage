import { useEffect, useState } from "react";
import {
  fetchCurrent,
  fetchHistory,
  checkSession,
  logout,
} from "./api";
import type { UsageSnapshot } from "./api";
import { UsageGauge } from "./UsageGauge";
import { UsageChart } from "./UsageChart";
import { LoginPage } from "./LoginPage";
import "./App.css";

type TimeRange = "24h" | "7d" | "30d";

function rangeToFrom(range: TimeRange): string {
  const now = new Date();
  const hours = range === "24h" ? 24 : range === "7d" ? 168 : 720;
  return new Date(now.getTime() - hours * 3600_000).toISOString();
}

function App() {
  const [authed, setAuthed] = useState<boolean | null>(null); // null = loading
  const [current, setCurrent] = useState<UsageSnapshot | null>(null);
  const [history, setHistory] = useState<UsageSnapshot[]>([]);
  const [range, setRange] = useState<TimeRange>("24h");
  const [error, setError] = useState<string | null>(null);

  // Check session on mount
  useEffect(() => {
    checkSession().then(setAuthed);
  }, []);

  // Fetch data when authenticated
  useEffect(() => {
    if (!authed) return;
    fetchCurrent().then(setCurrent).catch((e) => setError(e.message));
  }, [authed]);

  useEffect(() => {
    if (!authed) return;
    const from = rangeToFrom(range);
    fetchHistory(from).then(setHistory).catch((e) => setError(e.message));
  }, [range, authed]);

  // Auto-refresh every 5 minutes
  useEffect(() => {
    if (!authed) return;
    const id = setInterval(() => {
      fetchCurrent().then(setCurrent).catch(() => {});
      fetchHistory(rangeToFrom(range)).then(setHistory).catch(() => {});
    }, 300_000);
    return () => clearInterval(id);
  }, [range, authed]);

  const handleLogout = async () => {
    await logout();
    setAuthed(false);
    setCurrent(null);
    setHistory([]);
  };

  if (authed === null) {
    return <div className="app loading">Loading...</div>;
  }

  if (!authed) {
    return <LoginPage onLogin={() => setAuthed(true)} />;
  }

  return (
    <div className="app">
      <header className="app-header">
        <h1>Claude Usage</h1>
        <button className="logout-btn" onClick={handleLogout}>
          Logout
        </button>
      </header>

      {error && <div className="error">{error}</div>}

      {current && (
        <div className="gauges">
          <UsageGauge
            label="Current session"
            value={current.five_hour_usage}
            resetsAt={current.five_hour_resets_at}
          />
          <UsageGauge
            label="All models"
            value={current.seven_day_usage}
            resetsAt={current.seven_day_resets_at}
          />
          <UsageGauge
            label="Sonnet only"
            value={current.seven_day_sonnet_usage}
            resetsAt={null}
          />
        </div>
      )}

      {current?.subscription_type && (
        <p className="sub-type">Plan: {current.subscription_type}</p>
      )}

      <div className="range-picker">
        {(["24h", "7d", "30d"] as TimeRange[]).map((r) => (
          <button
            key={r}
            className={range === r ? "active" : ""}
            onClick={() => setRange(r)}
          >
            {r}
          </button>
        ))}
      </div>

      <UsageChart data={history} />
    </div>
  );
}

export default App;
