import { goto } from "$app/navigation";

const TOKEN_KEY = "yass_token";

export function authToken(): string | null {
  try {
    return localStorage.getItem(TOKEN_KEY);
  } catch {
    return null;
  }
}

export function setAuthToken(token: string | null) {
  try {
    if (token) localStorage.setItem(TOKEN_KEY, token);
    else localStorage.removeItem(TOKEN_KEY);
  } catch {}
}

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message);
  }
}

async function errorMessage(response: Response): Promise<string> {
  const text = await response.text();
  try {
    return JSON.parse(text).error ?? text;
  } catch {
    return text || `HTTP ${response.status}`;
  }
}

export async function api<T>(
  path: string,
  init: RequestInit & { json?: unknown } = {},
): Promise<T> {
  const headers = new Headers(init.headers);
  const token = authToken();
  if (token) headers.set("Authorization", `Bearer ${token}`);

  let body = init.body;
  if (init.json !== undefined) {
    headers.set("Content-Type", "application/json");
    body = JSON.stringify(init.json);
  }

  const response = await fetch(path, { ...init, headers, body });

  if (response.status === 401) {
    setAuthToken(null);
    await goto("/login");
    throw new ApiError(401, "登入已過期");
  }
  if (!response.ok) {
    throw new ApiError(response.status, await errorMessage(response));
  }
  return response.json() as Promise<T>;
}

export async function checkSession(): Promise<boolean> {
  if (!authToken()) return false;
  try {
    const response = await fetch("/api/auth/me", {
      headers: { Authorization: `Bearer ${authToken()}` },
    });
    return response.ok;
  } catch {
    return false;
  }
}

export async function login(username: string, password: string): Promise<void> {
  const response = await fetch("/api/auth/login", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ username, password }),
  });
  if (!response.ok)
    throw new ApiError(response.status, await errorMessage(response));
  const data: { token: string } = await response.json();
  setAuthToken(data.token);
}

export async function logout() {
  setAuthToken(null);
  await goto("/login");
}
