<script setup lang="ts">
import type { ThemeMode, UiLanguage } from "../ui";
import { translate } from "../ui";

const props = defineProps<{
  tokenInput: string;
  isLoading: boolean;
  currentTheme: ThemeMode;
  currentLanguage: UiLanguage;
}>();

const emit = defineEmits<{
  (event: "token-input", value: string): void;
  (event: "save-token"): void;
  (event: "reload-data"): void;
  (event: "set-theme", value: ThemeMode): void;
  (event: "set-language", value: UiLanguage): void;
}>();

function text(key: Parameters<typeof translate>[1]): string {
  return translate(props.currentLanguage, key);
}
</script>

<template>
  <section class="top-bar card row">
    <h1 class="title">{{ text("title") }}</h1>

    <div class="controls row">
      <label for="token">{{ text("tokenLabel") }}</label>
      <input
        id="token"
        class="input"
        :value="tokenInput"
        @input="emit('token-input', ($event.target as HTMLInputElement).value)"
      />
      <button class="button" type="button" @click="emit('save-token')">{{ text("saveToken") }}</button>
      <button class="button secondary" type="button" @click="emit('reload-data')">{{ text("loadData") }}</button>
      <span v-if="isLoading" class="muted">{{ text("loading") }}</span>
    </div>

    <div class="switch-area row">
      <div class="switch-group">
        <span class="muted">{{ text("theme") }}</span>
        <button
          class="button secondary"
          :class="{ active: currentTheme === 'dark' }"
          type="button"
          @click="emit('set-theme', 'dark')"
        >
          {{ text("dark") }}
        </button>
        <button
          class="button secondary"
          :class="{ active: currentTheme === 'light' }"
          type="button"
          @click="emit('set-theme', 'light')"
        >
          {{ text("light") }}
        </button>
      </div>
      <div class="switch-group">
        <span class="muted">{{ text("language") }}</span>
        <button
          class="button secondary"
          :class="{ active: currentLanguage === 'zh' }"
          type="button"
          @click="emit('set-language', 'zh')"
        >
          {{ text("chinese") }}
        </button>
        <button
          class="button secondary"
          :class="{ active: currentLanguage === 'en' }"
          type="button"
          @click="emit('set-language', 'en')"
        >
          {{ text("english") }}
        </button>
      </div>
    </div>
  </section>
</template>

<style scoped>
.top-bar {
  display: grid;
  grid-template-columns: 1fr;
  gap: 12px;
}

.title {
  margin: 0;
}

.controls {
  align-items: center;
}

.switch-area {
  align-items: center;
  justify-content: space-between;
}

.switch-group {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
}
</style>
