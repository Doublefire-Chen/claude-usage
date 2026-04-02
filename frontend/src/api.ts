const API_BASE_URL = import.meta.env.VITE_API_URL || "";

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
  const res = await fetch(`${API_BASE_URL}/usage/current`);
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
}

export async function createChallenge(): Promise<ChallengeResponse> {
  const res = await fetch(`${API_BASE_URL}/auth/challenge`, { method: "POST" });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json();
}

export async function checkChallengeStatus(
  token: string
): Promise<StatusResponse> {
  const res = await fetch(`${API_BASE_URL}/auth/status?token=${token}`);
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json();
}

export async function checkSession(): Promise<boolean> {
  const res = await fetch(`${API_BASE_URL}/usage/current`);
  return res.ok;
}

export async function logout(): Promise<void> {
  await fetch(`${API_BASE_URL}/auth/logout`, { method: "POST" });
}

// --- Usage ---

export async function fetchHistory(
  from?: string,
  to?: string
): Promise<UsageSnapshot[]> {
  const params = new URLSearchParams();
  if (from) params.set("from", from);
  if (to) params.set("to", to);
  const res = await fetch(`${API_BASE_URL}/usage/history?${params}`);
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json();
}
