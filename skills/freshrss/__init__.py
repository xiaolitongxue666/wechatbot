"""
freshrss skill — Poll FreshRSS aggregated RSS, track sent articles, push to WeChat.

Config via environment variables:
    FRESHRSS_RSS_URL       — FreshRSS aggregated RSS URL (required)
    RSS_CHECK_INTERVAL     — poll interval (default 60)
    BOT_ADMIN_URL          — Admin API URL
    BOT_ADMIN_TOKEN        — Admin API Bearer token
    BOT_ID                 — Bot identifier
    BOT_USER_ID            — WeChat user to push to

History file (auto-created, gitignored):
    sent_articles.txt
"""

import feedparser
import os
import time
from datetime import datetime

from ..base import SkillBase, BotClient

HISTORY_FILE = os.getenv("RSS_HISTORY_FILE", "sent_articles.txt")


def load_history() -> set[str]:
    if not os.path.exists(HISTORY_FILE):
        return set()
    with open(HISTORY_FILE, "r", encoding="utf-8") as f:
        return set(line.strip() for line in f if line.strip())


def save_history(ids: set[str]):
    with open(HISTORY_FILE, "w", encoding="utf-8") as f:
        for aid in sorted(ids):
            f.write(aid + "\n")


class FreshrssSkill(SkillBase):
    name = "freshrss"
    description = "Poll FreshRSS aggregated RSS, dedup, push to WeChat"

    def run(self):
        check_interval = int(os.getenv("RSS_CHECK_INTERVAL", "60"))

        if not self.client.is_configured():
            print("[freshrss] WARNING: Bot client not configured — push disabled")

        sent_ids = load_history()
        print(f"[freshrss] 已读文章: {len(sent_ids)} 篇")
        print(f"[freshrss] 检查间隔: {check_interval}s")
        print(f"[freshrss] 历史文件: {HISTORY_FILE}")

        while not self.should_stop:
            try:
                self._check(sent_ids)
                save_history(sent_ids)
            except Exception as e:
                print(f"[freshrss] ❌ {e}")
            for _ in range(check_interval):
                if self.should_stop:
                    return
                time.sleep(1)

    def _check(self, sent_ids: set[str]):
        rss_url = os.getenv("FRESHRSS_RSS_URL", "")
        if not rss_url:
            print("[freshrss] ⚠️  FRESHRSS_RSS_URL 未设置")
            return

        print(f"[{datetime.now():%H:%M:%S}] 🔍 检查 FreshRSS...")
        feed = feedparser.parse(rss_url)

        if feed.bozo:
            print(f"  ⚠️  解析异常: {feed.bozo_exception}")

        new_count = 0
        for entry in feed.entries:
            aid = entry.id
            if aid in sent_ids:
                continue

            title = entry.get("title", "(无标题)")
            link = entry.get("link", "")
            author = entry.get("author", "")
            published = entry.get("published", "")

            print(f"  📄 {title}")
            print(f"     {link}")
            if author:
                print(f"     作者: {author}")
            if published:
                print(f"     时间: {published}")

            # Push to WeChat
            if self.client.is_configured():
                body = f"by {author}" if author else ""
                ok = self.client.send_markdown(title, body, link)
                if ok:
                    print(f"  ✅ Bot 推送成功: {title}")
                else:
                    print(f"  ⚠️ Bot 推送失败: {title}")

            sent_ids.add(aid)
            new_count += 1

        if new_count == 0:
            print(f"  无新文章")
        else:
            print(f"  ✅ 发现 {new_count} 篇新文章")


# Module-level instance — discovered by skills.run
skill = FreshrssSkill()
