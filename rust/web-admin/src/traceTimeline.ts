import type { SessionHistory, SessionHistoryRow } from "./types";

export type TraceTimelineEntry = {
  sortKey: string;
  display: string;
};

function normalizeTimestamp(raw: string): string {
  const trimmed = raw.trim();
  const isoMatch = trimmed.match(/^(\d{4}-\d{2}-\d{2}T[\d:.]+Z)/);
  if (isoMatch) {
    return isoMatch[1];
  }
  const parsed = Date.parse(trimmed);
  if (!Number.isNaN(parsed)) {
    return new Date(parsed).toISOString();
  }
  return trimmed;
}

function messageTraceLine(row: SessionHistoryRow, botId: string, sessionUserId: string): TraceTimelineEntry {
  const timestamp = normalizeTimestamp(row.received_at);
  const isFromUser = row.from_user_id === sessionUserId;
  const isToBot = row.to_user_id === botId;
  const route = isFromUser || isToBot
    ? `${row.from_user_id} -> ${botId}`
    : `${botId} -> ${row.to_user_id}`;
  const content = row.text_content.trim().length > 0 ? row.text_content.trim() : `(${row.content_type})`;
  return {
    sortKey: timestamp,
    display: `${timestamp} ${route}: ${content}`,
  };
}

function workerTraceLine(line: string, botId: string, serviceLabel: string): TraceTimelineEntry | null {
  const timestampMatch = line.match(/^(\d{4}-\d{2}-\d{2}T[\d:.]+Z)/);
  const timestamp = timestampMatch?.[1] ?? "";
  const body = timestamp.length > 0 ? line.slice(timestamp.length).trim() : line.trim();
  if (body.length === 0) {
    return null;
  }

  if (body.includes("forward event consumed")) {
    return {
      sortKey: timestamp || body,
      display: `${timestamp} ${botId} -> ${serviceLabel}: 发送请求`,
    };
  }
  if (body.includes("forwarding blocked")) {
    return {
      sortKey: timestamp || body,
      display: `${timestamp} ${botId} -> ${serviceLabel}: 转发被策略阻止`,
    };
  }
  if (body.includes("forward failed") || body.includes("failed after")) {
    return {
      sortKey: timestamp || body,
      display: `${timestamp} ${serviceLabel} -> ${botId}: 转发失败`,
    };
  }
  if (body.includes("forward endpoint returned") || body.includes("returned") || body.includes("success")) {
    return {
      sortKey: timestamp || body,
      display: `${timestamp} ${serviceLabel} -> ${botId}: 请求完成`,
    };
  }

  const compact = body.replace(/\s+/g, " ").slice(0, 160);
  return {
    sortKey: timestamp || compact,
    display: timestamp.length > 0 ? `${timestamp} ${compact}` : compact,
  };
}

function resolveServiceLabel(
  policyTargets: string[] | undefined,
  policyTargetsInput: string,
): string {
  const fromPolicy = policyTargets?.find((target) => target.trim().length > 0)?.trim();
  if (fromPolicy) {
    return fromPolicy;
  }
  const fromInput = policyTargetsInput
    .split(",")
    .map((target) => target.trim())
    .find((target) => target.length > 0);
  return fromInput ?? "webhook";
}

export function buildTraceTimeline(
  sessionHistory: SessionHistory | null,
  workerLines: string[],
  botId: string,
  sessionUserId: string,
  policyTargets: string[] | undefined,
  policyTargetsInput: string,
): TraceTimelineEntry[] {
  const serviceLabel = resolveServiceLabel(policyTargets, policyTargetsInput);
  const entries: TraceTimelineEntry[] = [];

  for (const row of sessionHistory?.rows ?? []) {
    entries.push(messageTraceLine(row, botId, sessionUserId));
  }

  for (const line of workerLines) {
    const workerEntry = workerTraceLine(line, botId, serviceLabel);
    if (workerEntry) {
      entries.push(workerEntry);
    }
  }

  entries.sort((left, right) => left.sortKey.localeCompare(right.sortKey));
  return entries;
}
