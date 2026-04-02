import { useEffect, useRef, useState } from "react";
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
      <h1>Claude Usage</h1>
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
