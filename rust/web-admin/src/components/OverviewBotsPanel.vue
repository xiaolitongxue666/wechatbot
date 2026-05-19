<script setup lang="ts">
import type { BotListItem, Overview } from "../types";
import type { UiLanguage } from "../ui";
import { translate } from "../ui";

const props = defineProps<{
  overview: Overview | null;
  bots: BotListItem[];
  selectedBotId: string;
  currentLanguage: UiLanguage;
}>();

const emit = defineEmits<{
  (event: "create-bot"): void;
  (event: "select-bot", botId: string): void;
}>();

function text(key: Parameters<typeof translate>[1]): string {
  return translate(props.currentLanguage, key);
}
</script>

<template>
  <div class="panel-stack">
    <section class="card">
      <h2>{{ text("overview") }}</h2>
      <div v-if="overview" class="grid">
        <div class="cell">{{ text("totalBots") }}: {{ overview.total_bots }}</div>
        <div class="cell">{{ text("onlineBots") }}: {{ overview.online_bots }}</div>
        <div class="cell">{{ text("messagesTodayTotal") }}: {{ overview.messages_today }}</div>
        <div class="cell">{{ text("forwardFailuresTodayTotal") }}: {{ overview.forward_failures_today }}</div>
      </div>
      <div v-else class="muted">{{ text("noOverviewData") }}</div>
    </section>

    <section class="card">
      <div class="row between">
        <h2>{{ text("bots") }}</h2>
        <button class="button" type="button" @click="emit('create-bot')">{{ text("createBot") }}</button>
      </div>
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>{{ text("botId") }}</th>
              <th>{{ text("status") }}</th>
              <th>{{ text("heartbeat") }}</th>
              <th>{{ text("botMessagesToday") }}</th>
              <th>{{ text("botForwardFailures") }}</th>
              <th>{{ text("action") }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="botRow in bots" :key="botRow.bot_id" :class="{ selected: selectedBotId === botRow.bot_id }">
              <td>{{ botRow.bot_id }}</td>
              <td>{{ botRow.status }}</td>
              <td>{{ botRow.last_heartbeat_display || text("notAvailable") }}</td>
              <td>{{ botRow.messages_today }}</td>
              <td>{{ botRow.forward_failures_today }}</td>
              <td>
                <button class="button secondary" type="button" @click="emit('select-bot', botRow.bot_id)">
                  {{ text("select") }}
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <div v-if="bots.length === 0" class="muted">{{ text("noBots") }}</div>
    </section>
  </div>
</template>

<style scoped>
.panel-stack {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  gap: 12px;
  min-height: 0;
}

.panel-stack > .card:last-child {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.table-wrap {
  min-height: 0;
  flex: 1;
  overflow: auto;
}

h2 {
  margin-top: 0;
}
</style>
