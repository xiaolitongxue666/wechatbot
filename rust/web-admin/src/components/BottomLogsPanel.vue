<script setup lang="ts">
import type { SessionHistory, SystemLogPayload } from "../types";
import type { UiLanguage } from "../ui";
import { translate } from "../ui";

const props = defineProps<{
  currentLanguage: UiLanguage;
  sessionHistory: SessionHistory | null;
  historyPage: number;
  isHistoryLoading: boolean;
  historyError: string;
  systemAdminLog: SystemLogPayload | null;
  systemWorkerLog: SystemLogPayload | null;
  isSystemLogLoading: boolean;
  systemLogError: string;
  systemLogUpdatedAt: string;
}>();

const emit = defineEmits<{
  (event: "history-prev"): void;
  (event: "history-next"): void;
  (event: "refresh-system-logs"): void;
}>();

function text(key: Parameters<typeof translate>[1]): string {
  return translate(props.currentLanguage, key);
}
</script>

<template>
  <section class="bottom-bar card">
    <div class="panel-header top-header">
      <h2>{{ text("logsPanel") }}</h2>
      <div class="header-actions">
        <span class="muted">{{ systemLogUpdatedAt }}</span>
        <button class="button secondary" type="button" @click="emit('refresh-system-logs')">
          {{ text("refreshLogs") }}
        </button>
      </div>
    </div>

    <div class="logs-grid">
      <article class="log-panel">
        <div class="panel-header">
          <h3>{{ text("sessionLogs") }}</h3>
          <div class="pager-actions">
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

        <div v-if="historyError" class="error-text">{{ historyError }}</div>
        <div v-if="isHistoryLoading" class="muted">{{ text("loading") }}</div>

        <div v-if="sessionHistory" class="scroll-area">
          <div v-for="row in sessionHistory.rows" :key="`${row.received_at}-${row.from_user_id}-${row.to_user_id}`" class="log-line">
            [{{ row.received_at }}] [{{ row.direction }}] {{ row.from_user_id }} -> {{ row.to_user_id }}: {{ row.text_content }}
          </div>
        </div>
        <div v-else class="muted">{{ text("noLogData") }}</div>
      </article>

      <article class="log-panel">
        <div class="panel-header">
          <h3>{{ text("systemLogs") }}</h3>
          <span class="muted">{{ text("adminLogs") }} / {{ text("workerLogs") }}</span>
        </div>
        <div v-if="systemLogError" class="error-text">{{ systemLogError }}</div>
        <div v-if="isSystemLogLoading" class="muted">{{ text("loading") }}</div>
        <div class="system-logs">
          <section class="sub-panel">
            <h4>{{ text("adminLogs") }}</h4>
            <div class="scroll-area">
              <div v-for="line in systemAdminLog?.lines ?? []" :key="line" class="log-line">{{ line }}</div>
              <div v-if="!systemAdminLog || systemAdminLog.lines.length === 0" class="muted">{{ text("noLogData") }}</div>
            </div>
          </section>
          <section class="sub-panel">
            <h4>{{ text("workerLogs") }}</h4>
            <div class="scroll-area">
              <div v-for="line in systemWorkerLog?.lines ?? []" :key="line" class="log-line">{{ line }}</div>
              <div v-if="!systemWorkerLog || systemWorkerLog.lines.length === 0" class="muted">{{ text("noLogData") }}</div>
            </div>
          </section>
        </div>
      </article>
    </div>
  </section>
</template>

<style scoped>
.bottom-bar {
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
  overflow: hidden;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  min-height: 36px;
  flex-wrap: wrap;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.header-actions .muted {
  white-space: nowrap;
}

.top-header .header-actions {
  margin-left: auto;
}

.pager-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.logs-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 12px;
  min-height: 0;
  flex: 1;
  overflow: hidden;
  align-items: stretch;
}

.log-panel {
  border: 1px solid var(--card-border);
  border-radius: 8px;
  padding: 10px;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.sub-panel {
  border: 1px solid var(--card-border);
  border-radius: 8px;
  padding: 8px;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.system-logs {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 10px;
  min-height: 0;
  flex: 1;
  overflow: hidden;
}

.scroll-area {
  min-height: 0;
  overflow: auto;
  border: 1px solid var(--table-border);
  border-radius: 6px;
  padding: 6px;
  background: var(--cell-bg);
  flex: 1;
}

.log-line {
  font-family: Menlo, Consolas, monospace;
  font-size: 12px;
  line-height: 1.45;
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--text-main);
}

.error-text {
  color: var(--error-text);
  margin-bottom: 6px;
}

@media (max-width: 1100px) {
  .logs-grid {
    grid-template-columns: minmax(0, 1fr);
  }

  .system-logs {
    grid-template-columns: minmax(0, 1fr);
  }
}

h2,
h3,
h4 {
  margin-top: 0;
  margin-bottom: 0;
}

@media (max-width: 860px) {
  .top-header {
    align-items: flex-start;
  }

  .top-header .header-actions {
    margin-left: 0;
    width: 100%;
    justify-content: flex-start;
  }
}
</style>
