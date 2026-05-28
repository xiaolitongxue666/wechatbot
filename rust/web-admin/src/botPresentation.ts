import type { UiLanguage } from "./ui";

const STATUS_LABELS: Record<string, { zh: string; en: string }> = {
  online: { zh: "在线", en: "Online" },
  offline: { zh: "离线", en: "Offline" },
  pending_qr: { zh: "待扫码登录", en: "Awaiting QR scan" },
  expired: { zh: "已过期", en: "Expired" },
  retrying: { zh: "重试中", en: "Retrying" },
  blocked: { zh: "已阻止", en: "Blocked" },
};

export type BotStatusTone = "online" | "offline" | "pending" | "unknown";

export function formatBotStatus(language: UiLanguage, status: string): string {
  const normalized = status.trim().toLowerCase();
  const labels = STATUS_LABELS[normalized];
  if (labels) {
    return labels[language];
  }
  return status;
}

export function botStatusTone(status: string): BotStatusTone {
  const normalized = status.trim().toLowerCase();
  if (normalized === "online") {
    return "online";
  }
  if (normalized === "pending_qr") {
    return "pending";
  }
  if (normalized === "offline" || normalized === "expired") {
    return "offline";
  }
  return "unknown";
}

export function formatEngineReady(language: UiLanguage, hasRuntime: boolean): string {
  if (hasRuntime) {
    return language === "zh" ? "已就绪（可启动/停止 Bot）" : "Ready (start/stop available)";
  }
  return language === "zh" ? "未连接（Admin 未加载 Bot 引擎）" : "Unavailable (bot engine not loaded)";
}

export function isPendingQrStatus(status: string): boolean {
  return status.trim().toLowerCase() === "pending_qr";
}
