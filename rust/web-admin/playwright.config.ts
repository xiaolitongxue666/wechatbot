import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests/e2e",
  timeout: 30_000,
  webServer: {
    command: "bun run preview",
    url: "http://127.0.0.1:4174",
    reuseExistingServer: true,
    timeout: 60_000,
  },
  use: {
    baseURL: process.env.WEB_ADMIN_BASE_URL ?? "http://127.0.0.1:4174",
    trace: "on-first-retry",
  },
});
