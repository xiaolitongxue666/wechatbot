import type {
  BotDetail,
  BotListResponse,
  CreateBotResponse,
  ForwardPolicy,
  Overview,
  SessionHistory,
  SystemLogPayload,
} from "./types";

type HttpMethod = "GET" | "POST" | "PUT" | "DELETE";

type ApiError = {
  error?: string;
};

async function requestJson<T>(
  token: string,
  path: string,
  method: HttpMethod = "GET",
  body?: unknown,
): Promise<T> {
  const response = await fetch(path, {
    method,
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (!response.ok) {
    let errorText = `${response.status}`;
    try {
      const payload = (await response.json()) as ApiError;
      if (payload.error && payload.error.length > 0) {
        errorText = payload.error;
      }
    } catch {
      // Keep HTTP status as fallback.
    }
    throw new Error(errorText);
  }
  if (response.status === 204) {
    return undefined as T;
  }
  const contentType = response.headers.get("content-type") ?? "";
  if (!contentType.includes("application/json")) {
    const bodyText = await response.text();
    const preview = bodyText.slice(0, 120).replace(/\s+/g, " ");
    throw new Error(`Expected JSON response, got ${contentType || "unknown"}: ${preview}`);
  }
  try {
    return (await response.json()) as T;
  } catch (error) {
    const reason = error instanceof Error ? error.message : "unknown parse error";
    throw new Error(`Invalid JSON response: ${reason}`);
  }
}

export function fetchOverview(token: string): Promise<Overview> {
  return requestJson<Overview>(token, "/admin/api/overview");
}

export async function fetchBots(token: string) {
  const payload = await requestJson<BotListResponse>(token, "/admin/api/bots");
  return payload.bots;
}

export function fetchBotDetail(token: string, botId: string): Promise<BotDetail> {
  return requestJson<BotDetail>(token, `/admin/api/bots/${encodeURIComponent(botId)}`);
}

export function createBot(token: string): Promise<CreateBotResponse> {
  return requestJson<CreateBotResponse>(token, "/admin/api/bots", "POST");
}

export function startBot(token: string, botId: string): Promise<void> {
  return requestJson<void>(token, `/admin/api/bots/${encodeURIComponent(botId)}/start`, "POST");
}

export function stopBot(token: string, botId: string): Promise<void> {
  return requestJson<void>(token, `/admin/api/bots/${encodeURIComponent(botId)}/stop`, "POST");
}

export function deleteBot(token: string, botId: string): Promise<void> {
  return requestJson<void>(token, `/admin/api/bots/${encodeURIComponent(botId)}`, "DELETE");
}

export function fetchForwardPolicy(token: string, botId: string): Promise<ForwardPolicy> {
  return requestJson<ForwardPolicy>(token, `/admin/api/bots/${encodeURIComponent(botId)}/forward-policy`);
}

export function updateForwardPolicy(
  token: string,
  botId: string,
  policy: Pick<ForwardPolicy, "forwarding_enabled" | "allowed_targets">,
): Promise<ForwardPolicy> {
  return requestJson<ForwardPolicy>(
    token,
    `/admin/api/bots/${encodeURIComponent(botId)}/forward-policy`,
    "PUT",
    policy,
  );
}

export function fetchSessionHistory(
  token: string,
  sessionId: string,
  page: number,
  pageSize: number,
): Promise<SessionHistory> {
  const normalizedPage = Math.max(1, page);
  const normalizedPageSize = Math.min(200, Math.max(1, pageSize));
  return requestJson<SessionHistory>(
    token,
    `/admin/api/sessions/${encodeURIComponent(sessionId)}/history?page=${normalizedPage}&page_size=${normalizedPageSize}`,
  );
}

export function fetchSystemLog(token: string, source: "admin" | "worker", lines = 200): Promise<SystemLogPayload> {
  const normalizedLines = Math.max(1, Math.min(1000, lines));
  return requestJson<SystemLogPayload>(
    token,
    `/admin/api/system-logs/${source}?lines=${normalizedLines}`,
  );
}
