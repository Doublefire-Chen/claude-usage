const API_BASE_URL = import.meta.env.VITE_API_URL || "";

function getToken(): string | null {
  return localStorage.getItem("session_token");
}

function api(url: string, options?: RequestInit): Promise<Response> {
  const token = getToken();
  const headers: Record<string, string> = {
    ...(options?.headers as Record<string, string>),
  };
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }
  return fetch(`${API_BASE_URL}${url}`, { ...options, headers });
}

export interface UsageSnapshot {
  id: number;
  timestamp: string;
  five_hour_usage: number | null;
  seven_day_usage: number | null;
  seven_day_sonnet_usage: number | null;
  five_hour_resets_at: string | null;
  seven_day_resets_at: string | null;
  subscription_type: string | null;
}

export async function fetchCurrent(): Promise<UsageSnapshot | null> {
  const res = await api("/usage/current");
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json();
}

// --- Auth ---

export interface ChallengeResponse {
  token: string;
  command: string;
}

export interface StatusResponse {
  status: "pending" | "authenticated";
  session_token?: string;
}

export async function createChallenge(): Promise<ChallengeResponse> {
  const res = await api("/auth/challenge", { method: "POST" });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json();
}

export async function checkChallengeStatus(
  token: string
): Promise<StatusResponse> {
  const res = await api(`/auth/status?token=${token}`);
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  const data: StatusResponse = await res.json();
  if (data.status === "authenticated" && data.session_token) {
    localStorage.setItem("session_token", data.session_token);
  }
  return data;
}

export async function checkSession(): Promise<boolean> {
  if (!getToken()) return false;
  const res = await api("/usage/current");
  return res.ok;
}

export async function logout(): Promise<void> {
  const token = getToken();
  if (token) {
    await api("/auth/logout", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ token }),
    });
    localStorage.removeItem("session_token");
  }
}

// --- Usage ---

export async function fetchHistory(
  from?: string,
  to?: string
): Promise<UsageSnapshot[]> {
  const params = new URLSearchParams();
  if (from) params.set("from", from);
  if (to) params.set("to", to);
  const res = await api(`/usage/history?${params}`);
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json();
}
