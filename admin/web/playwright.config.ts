import { defineConfig } from "@playwright/test";

const previewHost = process.env.E2E_PREVIEW_HOST ?? "127.0.0.1";
const previewPort = process.env.E2E_PREVIEW_PORT ?? "4174";
const previewBase = process.env.WEB_ADMIN_BASE_URL ?? `http://${previewHost}:${previewPort}`;
// 由 tools/scripts/test/run_e2e.sh 后台启动 preview 时跳过，避免 Playwright webServer 子进程卡死
const skipWebServer = process.env.E2E_SKIP_WEBSERVER === "1";

export default defineConfig({
  testDir: "./tests/e2e",
  timeout: 30_000,
  expect: { timeout: 10_000 },
  retries: process.env.CI ? 1 : 0,
  webServer: skipWebServer
    ? undefined
    : {
        command: "bun run preview",
        url: `${previewBase}/admin/`,
        reuseExistingServer: !process.env.CI,
        timeout: 30_000,
        stdout: "ignore",
        stderr: "pipe",
      },
  use: {
    baseURL: previewBase,
    trace: "on-first-retry",
  },
});
