<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import type { SystemLogPayload } from "../types";
import type { UiLanguage } from "../ui";
import { translate } from "../ui";

const scrollContainer = ref<HTMLElement | null>(null);

const props = defineProps<{
  currentLanguage: UiLanguage;
  systemLog: SystemLogPayload | null;
  isSystemLogLoading: boolean;
  systemLogError: string;
  systemLogUpdatedAt: string;
}>();

const emit = defineEmits<{
  (event: "refresh-system-logs"): void;
}>();

function text(key: Parameters<typeof translate>[1]): string {
  return translate(props.currentLanguage, key);
}

watch(
  () => props.systemLog?.lines.length ?? 0,
  async () => {
    const element = scrollContainer.value;
    if (!element) {
      return;
    }
    const distanceFromBottom = element.scrollHeight - element.scrollTop - element.clientHeight;
    const stickToBottom = distanceFromBottom < 48;
    await nextTick();
    if (stickToBottom) {
      element.scrollTop = element.scrollHeight;
    }
  },
);
</script>

<template>
  <section class="bottom-bar card">
    <div class="panel-header top-header">
      <h2>{{ text("systemLogs") }}</h2>
      <div class="header-actions">
        <span class="auto-refresh-badge">{{ text("autoRefreshLogs") }}</span>
        <span class="muted">{{ systemLogUpdatedAt }}</span>
        <button class="button secondary" type="button" @click="emit('refresh-system-logs')">
          {{ text("refreshLogs") }}
        </button>
      </div>
    </div>

    <p class="muted system-log-hint">{{ text("systemLogsHint") }}</p>

    <div v-if="systemLogError" class="error-text">{{ systemLogError }}</div>
    <div v-if="isSystemLogLoading" class="loading-overlay">{{ text("loading") }}</div>

    <div ref="scrollContainer" class="scroll-area">
      <div v-for="(line, index) in systemLog?.lines ?? []" :key="index" class="log-line">{{ line }}</div>
      <div v-if="!systemLog || systemLog.lines.length === 0" class="muted">{{ text("noLogData") }}</div>
    </div>
  </section>
</template>

<style scoped>
.bottom-bar {
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
  overflow: hidden;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  min-height: var(--control-height);
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

.auto-refresh-badge {
  font-size: var(--text-xs);
  letter-spacing: -0.12px;
  line-height: 1;
  color: var(--muted-text);
  border: 1px solid var(--card-border);
  border-radius: var(--radius-md);
  padding: 4px 10px;
  white-space: nowrap;
}

.top-header .header-actions {
  margin-left: auto;
}

.system-log-hint {
  margin: 0;
  font-size: var(--text-xs);
  letter-spacing: -0.12px;
  line-height: 1;
}

.scroll-area {
  position: relative;
  min-height: 0;
  flex: 1;
  overflow: auto;
  border: 1px solid var(--table-border);
  border-radius: var(--radius-sm);
  padding: 6px;
  background: var(--cell-bg);
}

.loading-overlay {
  position: absolute;
  top: 8px;
  right: 10px;
  z-index: 1;
  font-size: var(--text-xs);
  color: var(--muted-text);
  background: color-mix(in srgb, var(--card-bg) 88%, transparent);
  border-radius: var(--radius-xs);
  padding: 2px 6px;
  pointer-events: none;
}

.log-line {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  line-height: 1.45;
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--text-main);
}

.error-text {
  color: var(--error-text);
}

h2 {
  margin: 0;
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
