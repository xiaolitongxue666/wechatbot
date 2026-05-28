<script setup lang="ts">
import { computed } from "vue";
import {
  botStatusTone,
  formatBotStatus,
  formatEngineReady,
  isPendingQrStatus,
} from "../botPresentation";
import type { BotDetail, ForwardPolicy, SessionHistory } from "../types";
import { buildTraceTimeline } from "../traceTimeline";
import type { UiLanguage } from "../ui";
import { translate } from "../ui";

const props = defineProps<{
  botDetail: BotDetail | null;
  selectedPolicy: ForwardPolicy | null;
  policyTargetsInput: string;
  selectedSessionId: string;
  sessionHistory: SessionHistory | null;
  historyPage: number;
  isHistoryLoading: boolean;
  historyError: string;
  forwardingLogLines: string[];
  isWorkerLogLoading: boolean;
  workerLogError: string;
  currentLanguage: UiLanguage;
}>();

const emit = defineEmits<{
  (event: "back-overview"): void;
  (event: "start-bot"): void;
  (event: "stop-bot"): void;
  (event: "delete-bot"): void;
  (event: "save-policy"): void;
  (event: "policy-targets-input", value: string): void;
  (event: "policy-enabled", value: boolean): void;
  (event: "select-session", sessionId: string): void;
  (event: "history-prev"): void;
  (event: "history-next"): void;
}>();

const traceTimeline = computed(() => {
  if (!props.botDetail) {
    return [];
  }
  const sessionUserId =
    props.botDetail.sessions.find((session) => session.session_id === props.selectedSessionId)?.user_id ?? "";
  return buildTraceTimeline(
    props.sessionHistory,
    props.forwardingLogLines,
    props.botDetail.bot_id,
    sessionUserId,
    props.selectedPolicy?.allowed_targets,
    props.policyTargetsInput,
  );
});

const statusLabel = computed(() => {
  if (!props.botDetail) {
    return "";
  }
  return formatBotStatus(props.currentLanguage, props.botDetail.status);
});

const statusToneClass = computed(() => {
  if (!props.botDetail) {
    return "tone-unknown";
  }
  return `tone-${botStatusTone(props.botDetail.status)}`;
});

const engineLabel = computed(() => {
  if (!props.botDetail) {
    return "";
  }
  return formatEngineReady(props.currentLanguage, props.botDetail.has_runtime);
});

const showScanQrHint = computed(() => {
  if (!props.botDetail) {
    return false;
  }
  return props.botDetail.sessions.length === 0 && isPendingQrStatus(props.botDetail.status);
});

function text(key: Parameters<typeof translate>[1]): string {
  return translate(props.currentLanguage, key);
}
</script>

<template>
  <section class="card detail-panel">
    <div class="row between">
      <h2>{{ text("selectedBot") }}: {{ botDetail?.bot_id }}</h2>
      <button class="button secondary" type="button" @click="emit('back-overview')">{{ text("backToOverview") }}</button>
    </div>

    <div v-if="botDetail" class="detail-layout">
      <div class="detail-main">
        <section class="status-banner" :class="statusToneClass">
          <div class="status-main">
            <span class="status-caption">{{ text("status") }}</span>
            <strong class="status-value">{{ statusLabel }}</strong>
          </div>
          <div class="status-meta">
            <span>{{ text("engine") }}: {{ engineLabel }}</span>
            <span v-if="botDetail.heartbeat_display">{{ text("heartbeat") }}: {{ botDetail.heartbeat_display }}</span>
          </div>
        </section>

        <div class="row actions">
          <button
            class="button"
            :class="{ 'action-active': botDetail.can_start }"
            type="button"
            :disabled="!botDetail.can_start"
            @click="emit('start-bot')"
          >
            {{ text("start") }}
          </button>
          <button
            class="button secondary"
            :class="{ 'action-active': botDetail.is_online }"
            type="button"
            :disabled="!botDetail.is_online"
            @click="emit('stop-bot')"
          >
            {{ text("stop") }}
          </button>
          <button class="button danger" type="button" @click="emit('delete-bot')">{{ text("delete") }}</button>
        </div>

        <section class="inner-card">
          <h3>{{ text("forwardingPolicy") }}</h3>
          <div v-if="selectedPolicy">
            <label class="row">
              <input
                :checked="selectedPolicy.forwarding_enabled"
                type="checkbox"
                @change="emit('policy-enabled', ($event.target as HTMLInputElement).checked)"
              />
              {{ text("enableForwarding") }}
            </label>
            <div class="row">
              <label for="targets">{{ text("allowedTargets") }}</label>
              <input
                id="targets"
                class="input"
                :value="policyTargetsInput"
                @input="emit('policy-targets-input', ($event.target as HTMLInputElement).value)"
              />
              <button class="button" type="button" @click="emit('save-policy')">{{ text("savePolicy") }}</button>
            </div>
          </div>
        </section>

        <section class="inner-card message-forward-card">
          <div class="panel-header">
            <div>
              <h3>{{ text("messageAndForward") }}</h3>
              <p class="muted flow-hint">{{ text("messageAndForwardHint") }}</p>
              <p class="muted trace-refresh-hint">{{ text("traceAutoRefresh") }}</p>
            </div>
            <div v-if="botDetail.sessions.length > 0 && selectedSessionId.length > 0" class="pager-actions">
              <button
                class="button secondary"
                type="button"
                :disabled="!sessionHistory || historyPage <= 1 || isHistoryLoading"
                @click="emit('history-prev')"
              >
                {{ text("prev") }}
              </button>
              <button
                class="button secondary"
                type="button"
                :disabled="
                  !sessionHistory ||
                  historyPage >= sessionHistory.total_pages ||
                  isHistoryLoading
                "
                @click="emit('history-next')"
              >
                {{ text("next") }}
              </button>
            </div>
          </div>

          <div v-if="botDetail.sessions.length > 0" class="row wrap session-list">
            <button
              v-for="session in botDetail.sessions"
              :key="session.session_id"
              class="button secondary"
              :class="{ active: session.session_id === selectedSessionId }"
              type="button"
              @click="emit('select-session', session.session_id)"
            >
              {{ session.session_id }}
            </button>
          </div>
          <div v-else-if="showScanQrHint" class="muted empty-hint">{{ text("scanQrForSession") }}</div>
          <div v-else class="muted empty-hint">{{ text("noSessions") }}</div>

          <div v-if="historyError" class="error-text">{{ historyError }}</div>
          <div v-if="workerLogError" class="error-text">{{ workerLogError }}</div>
          <div v-if="isHistoryLoading || isWorkerLogLoading" class="loading-chip">{{ text("loading") }}</div>

          <div
            v-if="botDetail.sessions.length > 0 && selectedSessionId.length === 0 && !isHistoryLoading"
            class="muted"
          >
            {{ text("selectSessionFirst") }}
          </div>

          <div
            v-if="botDetail.sessions.length === 0 || selectedSessionId.length > 0"
            class="scroll-area"
          >
            <div v-for="(entry, index) in traceTimeline" :key="index" class="log-line">
              {{ entry.display }}
            </div>
            <div v-if="traceTimeline.length === 0 && !isHistoryLoading && !isWorkerLogLoading" class="muted">
              {{ text("noTraceData") }}
            </div>
          </div>
        </section>
      </div>

      <section class="inner-card register-card">
        <h3>{{ text("openRegisterPage") }}</h3>
        <div class="register-inline-text">
          <strong>{{ text("botId") }}:</strong> {{ botDetail.bot_id }} | {{ text("scanToLogin") }}
        </div>
        <div class="register-qr-wrap">
          <img
            v-if="botDetail.register_qr_image_url"
            class="register-qr-image"
            :src="botDetail.register_qr_image_url"
            :alt="`bot-register-qr-${botDetail.bot_id}`"
          />
          <div v-else class="muted">{{ text("qrUnavailable") }}</div>
        </div>
      </section>
    </div>
    <div v-else class="muted">{{ text("selectBotFirst") }}</div>
  </section>
</template>

<style scoped>
.detail-panel {
  min-height: 0;
}

.detail-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(380px, 40%);
  gap: 12px;
  min-height: 0;
}

.detail-main {
  min-width: 0;
  min-height: 0;
}

.status-banner {
  border: 1px solid var(--card-border);
  border-radius: 8px;
  padding: 12px;
  margin-bottom: 10px;
  background: var(--cell-bg);
}

.status-banner.tone-online {
  border-color: #16a34a;
  background: color-mix(in srgb, #16a34a 12%, var(--cell-bg));
}

.status-banner.tone-pending {
  border-color: #d97706;
  background: color-mix(in srgb, #d97706 12%, var(--cell-bg));
}

.status-banner.tone-offline {
  border-color: #64748b;
}

.status-main {
  display: flex;
  align-items: baseline;
  gap: 10px;
  margin-bottom: 6px;
}

.status-caption {
  font-size: 13px;
  color: var(--muted-text);
}

.status-value {
  font-size: 20px;
  line-height: 1.2;
}

.status-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  font-size: 13px;
  color: var(--muted-text);
}

.actions {
  margin: 0 0 12px;
}

.button.action-active {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--button-active) 55%, transparent);
}

.inner-card {
  border: 1px solid var(--card-border);
  border-radius: 8px;
  padding: 12px;
  margin-top: 10px;
}

.register-card {
  margin-top: 0;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}

.register-inline-text {
  margin-bottom: 10px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.register-qr-wrap {
  flex: 1;
  min-height: 0;
  display: grid;
  place-items: center;
  border: 1px solid var(--card-border);
  border-radius: 8px;
  background: var(--cell-bg);
  overflow: hidden;
}

.register-qr-image {
  width: min(240px, 100%);
  height: auto;
  display: block;
}

.message-forward-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-height: 0;
}

.flow-hint {
  margin: 4px 0 0;
  font-size: 12px;
  line-height: 1.4;
}

.trace-refresh-hint {
  margin: 4px 0 0;
  font-size: 12px;
}

.loading-chip {
  align-self: flex-start;
  font-size: 12px;
  color: var(--muted-text);
  border: 1px solid var(--card-border);
  border-radius: 999px;
  padding: 2px 8px;
}

.message-forward-card .panel-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
  flex-wrap: wrap;
}

.empty-hint {
  line-height: 1.45;
}

.session-list {
  margin-top: 2px;
}

.pager-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.scroll-area {
  min-height: 160px;
  max-height: 320px;
  overflow: auto;
  border: 1px solid var(--table-border);
  border-radius: 6px;
  padding: 6px;
  background: var(--cell-bg);
}

.log-line {
  font-family: Menlo, Consolas, monospace;
  font-size: 12px;
  line-height: 1.45;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--text-main);
}

.error-text {
  color: var(--error-text);
}

.button.secondary.active {
  background: var(--button-active);
}

h2,
h3 {
  margin-top: 0;
}

@media (max-width: 1200px) {
  .detail-layout {
    grid-template-columns: 1fr;
  }

  .register-qr-wrap {
    min-height: 260px;
  }
}
</style>
