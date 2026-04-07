import { useEffect, useRef, useState } from "react";

const SPARKLE_CHARS = ["·", "✻", "✽", "✶", "✳", "✢"];

function ClaudeSparkle() {
  const [idx, setIdx] = useState(0);
  const ref = useRef<ReturnType<typeof setInterval>>(undefined);
  useEffect(() => {
    ref.current = setInterval(() => {
      setIdx((i) => (i + 1) % SPARKLE_CHARS.length);
    }, 120);
    return () => clearInterval(ref.current);
  }, []);
  return <span className="claude-sparkle">{SPARKLE_CHARS[idx]}</span>;
}
import { createChallenge, checkChallengeStatus } from "./api";
import "./LoginPage.css";

interface Props {
  onLogin: () => void;
}

export function LoginPage({ onLogin }: Props) {
  const [command, setCommand] = useState<string | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [expired, setExpired] = useState(false);
  const [copied, setCopied] = useState(false);
  const pollRef = useRef<ReturnType<typeof setInterval>>(undefined);

  const generateChallenge = async () => {
    setExpired(false);
    setCopied(false);
    const resp = await createChallenge();
    setToken(resp.token);
    setCommand(resp.command);
  };

  useEffect(() => {
    generateChallenge();
  }, []);

  // Poll for authentication
  useEffect(() => {
    if (!token) return;

    const start = Date.now();
    pollRef.current = setInterval(async () => {
      // Expire after 5 minutes
      if (Date.now() - start > 5 * 60 * 1000) {
        setExpired(true);
        clearInterval(pollRef.current);
        return;
      }
      try {
        const status = await checkChallengeStatus(token);
        if (status.status === "authenticated") {
          clearInterval(pollRef.current);
          onLogin();
        }
      } catch {
        // ignore polling errors
      }
    }, 2000);

    return () => clearInterval(pollRef.current);
  }, [token, onLogin]);

  const handleCopy = () => {
    if (command) {
      navigator.clipboard.writeText(command);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div className="login-page">
      <a href="https://github.com/Doublefire-Chen/claude-usage" target="_blank" rel="noopener noreferrer" className="github-link" aria-label="GitHub">
        <svg width="28" height="28" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
        </svg>
      </a>
      <h1><ClaudeSparkle /> Claude Usage</h1>
      <p className="login-subtitle">
        Authenticate by running this command in your terminal:
      </p>

      {command && !expired && (
        <div className="command-block">
          <code>{command}</code>
          <button className="copy-btn" onClick={handleCopy}>
            {copied ? "Copied!" : "Copy"}
          </button>
        </div>
      )}

      {!expired && command && (
        <p className="waiting">Waiting for authentication...</p>
      )}

      {expired && (
        <div className="expired">
          <p>Token expired.</p>
          <button onClick={generateChallenge}>Generate new command</button>
        </div>
      )}
    </div>
  );
}
