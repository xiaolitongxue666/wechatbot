<script setup lang="ts">
import type { BotDetail, ForwardPolicy } from "../types";
import type { UiLanguage } from "../ui";
import { translate } from "../ui";

const props = defineProps<{
  botDetail: BotDetail | null;
  selectedPolicy: ForwardPolicy | null;
  policyTargetsInput: string;
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
}>();

function text(key: Parameters<typeof translate>[1]): string {
  return translate(props.currentLanguage, key);
}
</script>

<template>
  <section class="card detail-panel">
    <div class="row between">
      <h2>{{ text("selectedBot") }}</h2>
      <button class="button secondary" type="button" @click="emit('back-overview')">{{ text("backToOverview") }}</button>
    </div>

    <div v-if="botDetail" class="detail-layout">
      <div class="detail-main">
        <div class="row wrap">
          <div><strong>{{ text("botId") }}:</strong> {{ botDetail.bot_id }}</div>
          <div><strong>{{ text("status") }}:</strong> {{ botDetail.status }}</div>
          <div><strong>{{ text("runtime") }}:</strong> {{ botDetail.has_runtime ? text("yes") : text("no") }}</div>
        </div>

        <div class="row actions">
          <button class="button" type="button" :disabled="!botDetail.can_start" @click="emit('start-bot')">
            {{ text("start") }}
          </button>
          <button class="button secondary" type="button" @click="emit('stop-bot')">{{ text("stop") }}</button>
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

        <section class="inner-card">
          <h3>{{ text("sessionHistory") }}</h3>
          <div v-if="botDetail.sessions.length > 0" class="row wrap">
            <button
              v-for="session in botDetail.sessions"
              :key="session.session_id"
              class="button secondary"
              type="button"
              @click="emit('select-session', session.session_id)"
            >
              {{ session.session_id }}
            </button>
          </div>
          <div v-else class="muted">{{ text("noSessions") }}</div>
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

.actions {
  margin: 12px 0;
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
