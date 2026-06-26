<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { filterWorkerLogLines } from "./logFilter";
import { assignIfChanged, jsonEquals, linesEquals } from "./silentRefresh";
import {
  createBot,
  deleteBot,
  fetchBotDetail,
  fetchBots,
  fetchForwardPolicy,
  fetchOverview,
  fetchSessionHistory,
  fetchSystemLog,
  startBot,
  stopBot,
  updateForwardPolicy,
} from "./api";
import BottomLogsPanel from "./components/BottomLogsPanel.vue";
import BotDetailPanel from "./components/BotDetailPanel.vue";
import OverviewBotsPanel from "./components/OverviewBotsPanel.vue";
import TopBar from "./components/TopBar.vue";
import type { BotDetail, BotListItem, ForwardPolicy, Overview, SessionHistory, SystemLogPayload } from "./types";
import type { ThemeMode, UiLanguage } from "./ui";
import { translate } from "./ui";

type MiddleMode = "overview" | "detail";

const savedToken = localStorage.getItem("admin_token") ?? "dev-admin-token";
const tokenInput = ref(savedToken);
const activeToken = ref(savedToken);
const currentTheme = ref<ThemeMode>((localStorage.getItem("admin_theme") as ThemeMode) ?? "dark");
const currentLanguage = ref<UiLanguage>((localStorage.getItem("admin_lang") as UiLanguage) ?? "zh");

const isLoading = ref(false);
const errorMessage = ref("");
const middleMode = ref<MiddleMode>("overview");

const overview = ref<Overview | null>(null);
const bots = ref<BotListItem[]>([]);
const selectedBotId = ref("");
const selectedBotDetail = ref<BotDetail | null>(null);
const selectedPolicy = ref<ForwardPolicy | null>(null);
const policyTargetsInput = ref("");

const selectedSessionId = ref("");
const sessionHistory = ref<SessionHistory | null>(null);
const historyPage = ref(1);
const pageSize = 30;
const isHistoryLoading = ref(false);
const historyError = ref("");

const systemLog = ref<SystemLogPayload | null>(null);
const isSystemLogLoading = ref(false);
const systemLogError = ref("");
const systemLogUpdatedAt = ref("");

const botWorkerLog = ref<SystemLogPayload | null>(null);
const isWorkerLogLoading = ref(false);
const workerLogError = ref("");

const forwardingLogLines = computed(() =>
  filterWorkerLogLines(
    botWorkerLog.value?.lines ?? [],
    selectedBotId.value,
    selectedSessionId.value,
  ),
);

const REFRESH_INTERVAL_MS = 1000;

let systemLogTimer: ReturnType<typeof setInterval> | undefined;
let traceTimer: ReturnType<typeof setInterval> | undefined;
let botStatusTimer: ReturnType<typeof setInterval> | undefined;

function botDetailSignature(detail: BotDetail): string {
  return [
    detail.status,
    detail.is_online,
    detail.can_start,
    detail.has_runtime,
    detail.heartbeat_display,
    detail.sessions.map((session) => `${session.session_id}:${session.status}`).join("|"),
  ].join(";");
}

function text(key: Parameters<typeof translate>[1]): string {
  return translate(currentLanguage.value, key);
}

function applyDocumentTheme() {
  document.body.dataset.theme = currentTheme.value;
  document.documentElement.lang = currentLanguage.value === "zh" ? "zh-CN" : "en";
}

function setTheme(nextTheme: ThemeMode) {
  currentTheme.value = nextTheme;
  localStorage.setItem("admin_theme", nextTheme);
  applyDocumentTheme();
}

function setLanguage(nextLanguage: UiLanguage) {
  currentLanguage.value = nextLanguage;
  localStorage.setItem("admin_lang", nextLanguage);
  applyDocumentTheme();
}

function setError(error: unknown) {
  if (error instanceof Error) {
    errorMessage.value = error.message;
    return;
  }
  errorMessage.value = text("unknownError");
}

function clearError() {
  errorMessage.value = "";
}

function saveToken() {
  const normalizedToken = tokenInput.value.trim();
  activeToken.value = normalizedToken;
  localStorage.setItem("admin_token", normalizedToken);
}

async function reloadOverviewAndBots() {
  clearError();
  isLoading.value = true;
  try {
    const [overviewPayload, botRows] = await Promise.all([
      fetchOverview(activeToken.value),
      fetchBots(activeToken.value),
    ]);
    overview.value = overviewPayload;
    bots.value = botRows;
  } catch (error) {
    setError(error);
  } finally {
    isLoading.value = false;
  }
}

async function reloadBotContext(botId: string) {
  const [detail, policy] = await Promise.all([
    fetchBotDetail(activeToken.value, botId),
    fetchForwardPolicy(activeToken.value, botId),
  ]);
  selectedBotId.value = detail.bot_id;
  selectedBotDetail.value = detail;
  selectedPolicy.value = policy;
  policyTargetsInput.value = policy.allowed_targets.join(",");
  middleMode.value = "detail";

  if (detail.sessions.length > 0) {
    selectedSessionId.value = detail.sessions[0].session_id;
    historyPage.value = 1;
    await Promise.all([reloadSessionHistory(), refreshWorkerLogs()]);
  } else {
    selectedSessionId.value = "";
    sessionHistory.value = null;
    await refreshWorkerLogs();
  }
}

async function selectBot(botId: string) {
  clearError();
  isLoading.value = true;
  try {
    await reloadBotContext(botId);
  } catch (error) {
    setError(error);
  } finally {
    isLoading.value = false;
  }
}

async function createBotNow() {
  clearError();
  isLoading.value = true;
  try {
    const created = await createBot(activeToken.value);
    await reloadOverviewAndBots();
    await selectBot(created.bot_id);
  } catch (error) {
    setError(error);
  } finally {
    isLoading.value = false;
  }
}

async function startSelectedBot() {
  if (selectedBotId.value.length === 0) {
    return;
  }
  clearError();
  isLoading.value = true;
  try {
    await startBot(activeToken.value, selectedBotId.value);
    await reloadOverviewAndBots();
    await selectBot(selectedBotId.value);
  } catch (error) {
    setError(error);
  } finally {
    isLoading.value = false;
  }
}

async function stopSelectedBot() {
  if (selectedBotId.value.length === 0) {
    return;
  }
  clearError();
  isLoading.value = true;
  try {
    await stopBot(activeToken.value, selectedBotId.value);
    await reloadOverviewAndBots();
    await selectBot(selectedBotId.value);
  } catch (error) {
    setError(error);
  } finally {
    isLoading.value = false;
  }
}

async function deleteSelectedBot() {
  if (selectedBotId.value.length === 0) {
    return;
  }
  clearError();
  isLoading.value = true;
  try {
    const botIdToDelete = selectedBotId.value;
    await deleteBot(activeToken.value, botIdToDelete);
    selectedBotId.value = "";
    selectedBotDetail.value = null;
    selectedPolicy.value = null;
    selectedSessionId.value = "";
    sessionHistory.value = null;
    middleMode.value = "overview";
    await reloadOverviewAndBots();
  } catch (error) {
    setError(error);
  } finally {
    isLoading.value = false;
  }
}

async function saveForwardPolicy() {
  if (!selectedPolicy.value) {
    return;
  }
  clearError();
  isLoading.value = true;
  try {
    const normalizedTargets = policyTargetsInput.value
      .split(",")
      .map((target) => target.trim())
      .filter((target) => target.length > 0);
    const nextPolicy = await updateForwardPolicy(activeToken.value, selectedPolicy.value.bot_id, {
      forwarding_enabled: selectedPolicy.value.forwarding_enabled,
      allowed_targets: normalizedTargets,
    });
    selectedPolicy.value = nextPolicy;
    policyTargetsInput.value = nextPolicy.allowed_targets.join(",");
  } catch (error) {
    setError(error);
  } finally {
    isLoading.value = false;
  }
}

async function reloadSessionHistory(silent = false) {
  if (selectedSessionId.value.length === 0) {
    return;
  }
  if (!silent) {
    historyError.value = "";
    isHistoryLoading.value = true;
  }
  try {
    const history = await fetchSessionHistory(activeToken.value, selectedSessionId.value, historyPage.value, pageSize);
    assignIfChanged(sessionHistory, history, jsonEquals);
  } catch (error) {
    if (!silent) {
      if (error instanceof Error) {
        historyError.value = error.message;
      } else {
        historyError.value = text("unknownError");
      }
    }
  } finally {
    if (!silent) {
      isHistoryLoading.value = false;
    }
  }
}

async function loadSessionHistory(sessionId: string) {
  selectedSessionId.value = sessionId;
  historyPage.value = 1;
  await Promise.all([reloadSessionHistory(), refreshWorkerLogs()]);
}

async function nextHistoryPage() {
  if (!sessionHistory.value || historyPage.value >= sessionHistory.value.total_pages) {
    return;
  }
  historyPage.value += 1;
  await reloadSessionHistory();
}

async function prevHistoryPage() {
  if (!sessionHistory.value || historyPage.value <= 1) {
    return;
  }
  historyPage.value -= 1;
  await reloadSessionHistory();
}

async function refreshSystemLogs(options?: { silent?: boolean; manual?: boolean }) {
  const silent = options?.silent ?? false;
  const manual = options?.manual ?? false;
  if (!silent) {
    systemLogError.value = "";
    isSystemLogLoading.value = true;
  }
  try {
    const next = await fetchSystemLog(activeToken.value, "admin", 200);
    const changed = assignIfChanged(systemLog, next, jsonEquals);
    if (!silent || changed || manual) {
      systemLogUpdatedAt.value = `${text("time")}: ${new Date().toLocaleString()}`;
    }
  } catch (error) {
    if (!silent) {
      if (error instanceof Error) {
        systemLogError.value = error.message;
      } else {
        systemLogError.value = text("unknownError");
      }
    }
  } finally {
    if (!silent) {
      isSystemLogLoading.value = false;
    }
  }
}

async function refreshWorkerLogs(silent = false) {
  if (middleMode.value !== "detail" || selectedBotId.value.length === 0) {
    return;
  }
  if (!silent) {
    workerLogError.value = "";
    isWorkerLogLoading.value = true;
  }
  try {
    const next = await fetchSystemLog(activeToken.value, "worker", 200);
    assignIfChanged(botWorkerLog, next, (left, right) => {
      if (left === null) {
        return false;
      }
      return linesEquals(left.lines, right.lines);
    });
  } catch (error) {
    if (!silent) {
      if (error instanceof Error) {
        workerLogError.value = error.message;
      } else {
        workerLogError.value = text("unknownError");
      }
    }
  } finally {
    if (!silent) {
      isWorkerLogLoading.value = false;
    }
  }
}

async function refreshTraceSnapshot(silent = true) {
  if (middleMode.value !== "detail" || selectedBotId.value.length === 0) {
    return;
  }
  const tasks: Promise<void>[] = [refreshWorkerLogs(silent)];
  if (selectedSessionId.value.length > 0) {
    tasks.push(reloadSessionHistory(silent));
  }
  await Promise.all(tasks);
}

async function refreshSelectedBotStatusRealtime() {
  if (middleMode.value !== "detail" || selectedBotId.value.length === 0) {
    return;
  }
  try {
    const detail = await fetchBotDetail(activeToken.value, selectedBotId.value);
    const nextSignature = botDetailSignature(detail);
    const previousSignature = selectedBotDetail.value ? botDetailSignature(selectedBotDetail.value) : "";
    if (nextSignature !== previousSignature) {
      selectedBotDetail.value = detail;
    }
    if (detail.sessions.length > 0 && selectedSessionId.value.length === 0) {
      selectedSessionId.value = detail.sessions[0].session_id;
      historyPage.value = 1;
      await Promise.all([reloadSessionHistory(), refreshWorkerLogs()]);
    }
    const selectedSessionStillExists = detail.sessions.some(
      (session) => session.session_id === selectedSessionId.value,
    );
    if (!selectedSessionStillExists) {
      if (detail.sessions.length > 0) {
        selectedSessionId.value = detail.sessions[0].session_id;
        historyPage.value = 1;
        await Promise.all([reloadSessionHistory(), refreshWorkerLogs()]);
      } else {
        selectedSessionId.value = "";
        sessionHistory.value = null;
        historyError.value = "";
        await refreshWorkerLogs();
      }
    }
  } catch {
    // Realtime refresh should not override explicit action errors.
  }
}

function backToOverview() {
  middleMode.value = "overview";
  botWorkerLog.value = null;
  workerLogError.value = "";
}

onMounted(async () => {
  applyDocumentTheme();
  await reloadOverviewAndBots();
  await refreshSystemLogs({ manual: true });
  systemLogTimer = setInterval(() => {
    void refreshSystemLogs({ silent: true });
  }, REFRESH_INTERVAL_MS);
  traceTimer = setInterval(() => {
    void refreshTraceSnapshot(true);
  }, REFRESH_INTERVAL_MS);
  botStatusTimer = setInterval(() => {
    void refreshSelectedBotStatusRealtime();
  }, REFRESH_INTERVAL_MS * 3);
});

onUnmounted(() => {
  if (systemLogTimer) {
    clearInterval(systemLogTimer);
    systemLogTimer = undefined;
  }
  if (traceTimer) {
    clearInterval(traceTimer);
    traceTimer = undefined;
  }
  if (botStatusTimer) {
    clearInterval(botStatusTimer);
    botStatusTimer = undefined;
  }
});
</script>

<template>
  <div class="layout">
    <TopBar
      :token-input="tokenInput"
      :is-loading="isLoading"
      :current-theme="currentTheme"
      :current-language="currentLanguage"
      @token-input="tokenInput = $event"
      @save-token="saveToken"
      @reload-data="reloadOverviewAndBots"
      @set-theme="setTheme"
      @set-language="setLanguage"
    />

    <section v-if="errorMessage.length > 0" class="card error">
      {{ errorMessage }}
    </section>

    <main class="middle-area card">
      <OverviewBotsPanel
        v-if="middleMode === 'overview'"
        :overview="overview"
        :bots="bots"
        :selected-bot-id="selectedBotId"
        :current-language="currentLanguage"
        @create-bot="createBotNow"
        @select-bot="selectBot"
      />
      <BotDetailPanel
        v-else
        :bot-detail="selectedBotDetail"
        :selected-policy="selectedPolicy"
        :policy-targets-input="policyTargetsInput"
        :selected-session-id="selectedSessionId"
        :session-history="sessionHistory"
        :history-page="historyPage"
        :is-history-loading="isHistoryLoading"
        :history-error="historyError"
        :forwarding-log-lines="forwardingLogLines"
        :is-worker-log-loading="isWorkerLogLoading"
        :worker-log-error="workerLogError"
        :current-language="currentLanguage"
        @back-overview="backToOverview"
        @start-bot="startSelectedBot"
        @stop-bot="stopSelectedBot"
        @delete-bot="deleteSelectedBot"
        @save-policy="saveForwardPolicy"
        @policy-targets-input="policyTargetsInput = $event"
        @policy-enabled="selectedPolicy && (selectedPolicy.forwarding_enabled = $event)"
        @select-session="loadSessionHistory"
        @history-prev="prevHistoryPage"
        @history-next="nextHistoryPage"
      />
    </main>

    <BottomLogsPanel
      :current-language="currentLanguage"
      :system-log="systemLog"
      :is-system-log-loading="isSystemLogLoading"
      :system-log-error="systemLogError"
      :system-log-updated-at="systemLogUpdatedAt"
      @refresh-system-logs="refreshSystemLogs({ manual: true })"
    />
  </div>
</template>

<style>
html,
body,
#app {
  min-height: 100%;
}

body {
  margin: 0;
  transition: background-color 0.15s ease, color 0.15s ease;
}

body[data-theme="dark"] {
  --page-bg: #0b1220;
  --text-main: #e2e8f0;
  --card-bg: #1e293b;
  --card-border: #334155;
  --error-text: #fecaca;
  --input-bg: #0f172a;
  --input-border: #475569;
  --button-primary: #2563eb;
  --button-secondary: #334155;
  --button-danger: #b91c1c;
  --button-active: #0ea5e9;
  --muted-text: #94a3b8;
  --table-border: #334155;
  --cell-bg: #0f172a;
  --selected-row-bg: #0f172a;
  --link-color: #93c5fd;
}

body[data-theme="light"] {
  --page-bg: #f8fafc;
  --text-main: #0f172a;
  --card-bg: #ffffff;
  --card-border: #dbe3ee;
  --error-text: #991b1b;
  --input-bg: #ffffff;
  --input-border: #cbd5e1;
  --button-primary: #2563eb;
  --button-secondary: #64748b;
  --button-danger: #dc2626;
  --button-active: #0ea5e9;
  --muted-text: #64748b;
  --table-border: #dbe3ee;
  --cell-bg: #f1f5f9;
  --selected-row-bg: #eef2ff;
  --link-color: #1d4ed8;
}

body {
  background: var(--page-bg);
  color: var(--text-main);
}
</style>

<style>
.layout {
  --bottom-panel-height: clamp(150px, 24vh, 280px);
  height: 100vh;
  box-sizing: border-box;
  max-width: 1320px;
  margin: 0 auto;
  padding: 16px;
  display: grid;
  grid-template-rows: auto auto minmax(260px, 1fr) auto;
  gap: 16px;
  font-family: var(--font-ui);
  font-size: var(--text-sm);
  line-height: var(--leading-body);
  letter-spacing: var(--tracking-caption);
  color: var(--text-main);
  overflow: hidden;
}

h1 {
  font-size: var(--text-lg);
  font-weight: 600;
  letter-spacing: 0.011em;
  margin: 0;
}

h2 {
  font-size: var(--text-lg);
  font-weight: 600;
  margin-top: 0;
}

h3 {
  font-size: var(--text-md);
  font-weight: 600;
  letter-spacing: var(--tracking-display);
  margin-top: 0;
}

.card {
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: var(--radius-lg);
  padding: 20px;
  margin: 0;
}

.error {
  color: var(--error-text);
  border: 1px solid #ef4444;
}

.middle-area {
  overflow: auto;
  min-height: 0;
}

.layout > * {
  min-height: 0;
}

.layout > .bottom-bar {
  height: var(--bottom-panel-height);
}

.row {
  display: flex;
  gap: 12px;
  align-items: center;
  flex-wrap: wrap;
}

.row.wrap {
  flex-wrap: wrap;
}

.row.between {
  justify-content: space-between;
}

.input {
  min-height: var(--control-height);
  padding: 12px 20px;
  border: 1px solid var(--input-border);
  border-radius: var(--radius-sm);
  background: var(--input-bg);
  color: var(--text-main);
  font-size: var(--text-sm);
  font-family: inherit;
  box-sizing: border-box;
  min-width: 260px;
}

.button {
  background: var(--button-primary);
  color: #ffffff;
  border: 2px solid transparent;
  border-radius: var(--radius-pill);
  padding: var(--btn-primary-py) var(--btn-primary-px);
  min-height: var(--control-height);
  font-size: var(--text-sm);
  font-family: inherit;
  line-height: var(--leading-caption);
  cursor: pointer;
  transition: transform 0.1s ease;
}

.button.secondary,
.button.danger {
  border-radius: var(--radius-sm);
  padding: var(--btn-utility-py) var(--btn-utility-px);
  min-height: auto;
}

.button.secondary {
  background: var(--button-secondary);
}

.button.danger {
  background: var(--button-danger);
}

.button.secondary.active {
  background: var(--button-secondary);
  border-color: var(--button-active);
  border-radius: var(--radius-pill);
  padding: 8px 14px;
}

.button.active {
  background: var(--button-active);
}

.button:focus-visible {
  outline: 2px solid var(--button-active);
  outline-offset: 2px;
}

.button:active:not(:disabled) {
  transform: scale(0.95);
}

.button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.muted {
  color: var(--muted-text);
}

.grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(150px, 1fr));
  gap: 12px;
}

.cell {
  background: var(--cell-bg);
  border-radius: var(--radius-sm);
  padding: var(--space-sm);
}

table {
  width: 100%;
  border-collapse: collapse;
}

th,
td {
  border-bottom: 1px solid var(--table-border);
  text-align: left;
  padding: 10px 12px;
}

tr.selected {
  background: var(--selected-row-bg);
}

.history {
  margin-top: 14px;
}

.message-cell {
  max-width: 360px;
  white-space: pre-wrap;
  word-break: break-word;
}

a {
  color: var(--link-color);
}

.switch-group {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
}

@media (max-height: 900px) {
  .layout {
    --bottom-panel-height: clamp(130px, 20vh, 220px);
    grid-template-rows: auto auto minmax(220px, 1fr) auto;
  }
}

@media (max-height: 760px) {
  .layout {
    --bottom-panel-height: clamp(110px, 17vh, 180px);
    grid-template-rows: auto auto minmax(180px, 1fr) auto;
    padding: 10px;
    gap: 8px;
  }
}
</style>
