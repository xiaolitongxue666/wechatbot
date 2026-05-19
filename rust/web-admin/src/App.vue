<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
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

const systemAdminLog = ref<SystemLogPayload | null>(null);
const systemWorkerLog = ref<SystemLogPayload | null>(null);
const isSystemLogLoading = ref(false);
const systemLogError = ref("");
const systemLogUpdatedAt = ref("");

let systemLogTimer: ReturnType<typeof setInterval> | undefined;
let botStatusTimer: ReturnType<typeof setInterval> | undefined;

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
    await reloadSessionHistory();
  } else {
    selectedSessionId.value = "";
    sessionHistory.value = null;
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

async function reloadSessionHistory() {
  if (selectedSessionId.value.length === 0) {
    return;
  }
  historyError.value = "";
  isHistoryLoading.value = true;
  try {
    const history = await fetchSessionHistory(activeToken.value, selectedSessionId.value, historyPage.value, pageSize);
    sessionHistory.value = history;
  } catch (error) {
    if (error instanceof Error) {
      historyError.value = error.message;
    } else {
      historyError.value = text("unknownError");
    }
  } finally {
    isHistoryLoading.value = false;
  }
}

async function loadSessionHistory(sessionId: string) {
  selectedSessionId.value = sessionId;
  historyPage.value = 1;
  await reloadSessionHistory();
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

async function refreshSystemLogs() {
  systemLogError.value = "";
  isSystemLogLoading.value = true;
  try {
    const [adminLog, workerLog] = await Promise.all([
      fetchSystemLog(activeToken.value, "admin", 200),
      fetchSystemLog(activeToken.value, "worker", 200),
    ]);
    systemAdminLog.value = adminLog;
    systemWorkerLog.value = workerLog;
    systemLogUpdatedAt.value = `${text("time")}: ${new Date().toLocaleString()}`;
  } catch (error) {
    if (error instanceof Error) {
      systemLogError.value = error.message;
    } else {
      systemLogError.value = text("unknownError");
    }
  } finally {
    isSystemLogLoading.value = false;
  }
}

async function refreshSelectedBotStatusRealtime() {
  if (middleMode.value !== "detail" || selectedBotId.value.length === 0) {
    return;
  }
  try {
    const detail = await fetchBotDetail(activeToken.value, selectedBotId.value);
    selectedBotDetail.value = detail;
    const selectedSessionStillExists = detail.sessions.some(
      (session) => session.session_id === selectedSessionId.value,
    );
    if (!selectedSessionStillExists) {
      if (detail.sessions.length > 0) {
        selectedSessionId.value = detail.sessions[0].session_id;
        historyPage.value = 1;
        await reloadSessionHistory();
      } else {
        selectedSessionId.value = "";
        sessionHistory.value = null;
        historyError.value = "";
      }
    }
  } catch {
    // Realtime refresh should not override explicit action errors.
  }
}

function backToOverview() {
  middleMode.value = "overview";
}

onMounted(async () => {
  applyDocumentTheme();
  await reloadOverviewAndBots();
  await refreshSystemLogs();
  systemLogTimer = setInterval(() => {
    void refreshSystemLogs();
  }, 5000);
  botStatusTimer = setInterval(() => {
    void refreshSelectedBotStatusRealtime();
  }, 3000);
});

onUnmounted(() => {
  if (systemLogTimer) {
    clearInterval(systemLogTimer);
    systemLogTimer = undefined;
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
        :current-language="currentLanguage"
        @back-overview="backToOverview"
        @start-bot="startSelectedBot"
        @stop-bot="stopSelectedBot"
        @delete-bot="deleteSelectedBot"
        @save-policy="saveForwardPolicy"
        @policy-targets-input="policyTargetsInput = $event"
        @policy-enabled="selectedPolicy && (selectedPolicy.forwarding_enabled = $event)"
        @select-session="loadSessionHistory"
      />
    </main>

    <BottomLogsPanel
      :current-language="currentLanguage"
      :session-history="sessionHistory"
      :history-page="historyPage"
      :is-history-loading="isHistoryLoading"
      :history-error="historyError"
      :system-admin-log="systemAdminLog"
      :system-worker-log="systemWorkerLog"
      :is-system-log-loading="isSystemLogLoading"
      :system-log-error="systemLogError"
      :system-log-updated-at="systemLogUpdatedAt"
      @history-prev="prevHistoryPage"
      @history-next="nextHistoryPage"
      @refresh-system-logs="refreshSystemLogs"
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
  padding: 14px;
  display: grid;
  grid-template-rows: auto auto minmax(260px, 1fr) auto;
  gap: 10px;
  font-family: Arial, sans-serif;
  color: var(--text-main);
  overflow: hidden;
}

.card {
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 10px;
  padding: 16px;
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
  padding: 8px;
  border: 1px solid var(--input-border);
  border-radius: 6px;
  background: var(--input-bg);
  color: var(--text-main);
  min-width: 260px;
}

.button {
  background: var(--button-primary);
  color: #ffffff;
  border: none;
  border-radius: 6px;
  padding: 8px 12px;
  cursor: pointer;
}

.button.secondary {
  background: var(--button-secondary);
}

.button.danger {
  background: var(--button-danger);
}

.button.active {
  background: var(--button-active);
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
  border-radius: 8px;
  padding: 10px;
}

table {
  width: 100%;
  border-collapse: collapse;
}

th,
td {
  border-bottom: 1px solid var(--table-border);
  text-align: left;
  padding: 8px;
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
  gap: 6px;
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
