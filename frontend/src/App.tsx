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
import { ClaudeSparkle } from "./ClaudeSparkle";
import { LoginPage } from "./LoginPage";
import "./App.css";

function useAnimatedFavicon() {
  useEffect(() => {
    const canvas = document.createElement("canvas");
    canvas.width = 32;
    canvas.height = 32;
    const ctx = canvas.getContext("2d")!;
    const link = document.getElementById("favicon") as HTMLLinkElement;
    const img = new Image();
    img.src = "/favicon.svg";

    let t = 0;
    let raf: number;

    img.onload = () => {
      function draw() {
        const scale = 0.4 + 0.6 * (0.5 + 0.5 * Math.sin(t));
        const size = 32 * scale;
        const offset = (32 - size) / 2;
        ctx.clearRect(0, 0, 32, 32);
        ctx.globalAlpha = 0.4 + 0.6 * scale;
        ctx.drawImage(img, offset, offset, size, size);
        ctx.globalAlpha = 1;
        link.href = canvas.toDataURL("image/png");
        t += 0.06;
        raf = requestAnimationFrame(draw);
      }
      draw();
    };

    return () => cancelAnimationFrame(raf);
  }, []);
}

type TimeRange = "24h" | "7d" | "30d";

function rangeToFrom(range: TimeRange): string {
  const now = new Date();
  const hours = range === "24h" ? 24 : range === "7d" ? 168 : 720;
  return new Date(now.getTime() - hours * 3600_000).toISOString();
}

function App() {
  useAnimatedFavicon();
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

  // Auto-refresh every 15 minutes
  useEffect(() => {
    if (!authed) return;
    const id = setInterval(() => {
      fetchCurrent().then(setCurrent).catch(() => {});
      fetchHistory(rangeToFrom(range)).then(setHistory).catch(() => {});
    }, 900_000);
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
        <div className="header-left">
          <a href="https://github.com/Doublefire-Chen/claude-usage" target="_blank" rel="noopener noreferrer" className="github-link" aria-label="GitHub">
            <svg width="28" height="28" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
            </svg>
          </a>
        </div>
        <div className="header-center">
          <h1>Claude Usage</h1>
          <ClaudeSparkle />
        </div>
        <div className="header-right">
          {localStorage.getItem("username") && (
            <span className="username">{localStorage.getItem("username")}</span>
          )}
          <button className="logout-btn" onClick={handleLogout}>
            Logout
          </button>
        </div>
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

      <UsageChart data={history} range={range} />
    </div>
  );
}

export default App;
