import { expect, test } from "@playwright/test";

test("web admin shell renders", async ({ page }) => {
  await page.route("**/admin/api/overview", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        total_bots: 2,
        online_bots: 1,
        messages_today: 12,
        forward_failures_today: 1,
        last_heartbeat_at: null,
      }),
    });
  });
  await page.route("**/admin/api/bots", async (route) => {
    if (route.request().method() === "POST") {
      await route.fulfill({
        status: 201,
        contentType: "application/json",
        body: JSON.stringify({
          bot_id: "bot-new",
          detail_api: "/admin/api/bots/bot-new",
          register_link: "http://127.0.0.1:8787/bot/bot-new",
        }),
      });
      return;
    }
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        bots: [
          {
            bot_id: "bot-001",
            status: "online",
            is_online: true,
            last_heartbeat_at: null,
            last_heartbeat_display: "2026-05-19 10:00:00 UTC",
            messages_today: 9,
            forward_failures_today: 2,
            updated_at: "2026-05-19T10:00:00Z",
          },
        ],
      }),
    });
  });
  await page.route("**/admin/api/bots/bot-001", async (route) => {
    if (route.request().method() === "DELETE") {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({ bot_id: "bot-001", action: "delete", status: "accepted" }),
      });
      return;
    }
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        bot_id: "bot-001",
        status: "online",
        is_online: true,
        can_start: false,
        has_runtime: true,
        has_qr_url: false,
        heartbeat_display: "2026-05-19 10:00:00 UTC",
        created_at: "2026-05-19T10:00:00Z",
        updated_at: "2026-05-19T10:00:00Z",
        register_link: "http://127.0.0.1:8787/bot/bot-001",
        sessions: [{ session_id: "sess-001", user_id: "wx_user", status: "active", created_at: "2026-05-19T10:00:00Z", updated_at: "2026-05-19T10:00:00Z" }],
      }),
    });
  });
  await page.route("**/admin/api/bots/bot-001/forward-policy", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        bot_id: "bot-001",
        forwarding_enabled: true,
        allowed_targets: ["webhook"],
      }),
    });
  });
  await page.route("**/admin/api/sessions/sess-001/history**", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        session_id: "sess-001",
        bot_id: "bot-001",
        page: 1,
        page_size: 30,
        total: 1,
        total_pages: 1,
        rows: [
          {
            received_at: "2026-05-19 10:00:00 UTC",
            from_user_id: "wx_user",
            to_user_id: "bot-001",
            content_type: "text",
            text_content: "hello",
            direction: "in",
          },
        ],
      }),
    });
  });
  await page.route("**/admin/api/system-logs/admin**", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        source: "admin",
        requested_lines: 200,
        returned_lines: 1,
        lines: ["admin log line"],
      }),
    });
  });
  await page.route("**/admin/api/system-logs/worker**", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        source: "worker",
        requested_lines: 200,
        returned_lines: 1,
        lines: ["worker log line"],
      }),
    });
  });

  await page.goto("/admin/");
  await expect(page.getByRole("heading", { name: /WeChatBot (Web Admin|管理后台)/ })).toBeVisible();
  await expect(page.getByRole("heading", { name: /Overview|概览/ })).toBeVisible();
  await expect(page.getByRole("heading", { name: /System Logs|系统日志/ })).toBeVisible();
  await expect(page.getByText(/(Total Messages Today|今日消息总数)/)).toBeVisible();
  await expect(page.getByText(/(Total Forward Failures Today|今日转发失败总数)/)).toBeVisible();
  await expect(page.getByText(/(Messages Today|今日消息数)/)).toBeVisible();
  await expect(page.getByText(/(Forward Failures|转发失败数)/)).toBeVisible();
  await expect(page.getByText(/admin log line/)).toBeVisible();
  await page.getByRole("button", { name: /Select|选择/ }).click();
  await expect(page.getByRole("heading", { name: /Selected Bot|当前 Bot/ })).toBeVisible();
  await expect(page.getByRole("heading", { name: /Messages & Forwarding|消息与转发/ })).toBeVisible();
  await expect(page.getByText(/worker log line/)).toBeVisible();
});
