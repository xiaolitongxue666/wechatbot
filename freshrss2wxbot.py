"""
freshrss2wxbot.py — 轮询 FreshRSS 聚合 RSS → 去重记录 → 推送微信 Bot
"""

import feedparser
import time
import os
import requests
from datetime import datetime

# ===================== 配置（环境变量覆盖）=====================
# FreshRSS 聚合 RSS URL（优先 $FRESHRSS_RSS_URL，否则用此默认值）
FRESHRSS_RSS_URL = os.getenv("FRESHRSS_RSS_URL", "https://xiaolitongxue.com.cn/freshrss/i/?c=index&a=rss&token=<你的token>&user=<你的邮箱>")

CHECK_INTERVAL = int(os.getenv("RSS_CHECK_INTERVAL", "60"))  # 秒
HISTORY_FILE = os.getenv("RSS_HISTORY_FILE", "sent_articles.txt")

# Bot 推送配置（优先 $BOT_* 环境变量）
BOT_ADMIN_URL = os.getenv("BOT_ADMIN_URL", "")
BOT_ADMIN_TOKEN = os.getenv("BOT_ADMIN_TOKEN", "")
BOT_ID = os.getenv("BOT_ID", "")
BOT_USER_ID = os.getenv("BOT_USER_ID", "")
# ============================================================


def load_history() -> set[str]:
    if not os.path.exists(HISTORY_FILE):
        return set()
    with open(HISTORY_FILE, "r", encoding="utf-8") as f:
        return set(line.strip() for line in f if line.strip())


def save_history(ids: set[str]):
    with open(HISTORY_FILE, "w", encoding="utf-8") as f:
        for aid in sorted(ids):
            f.write(aid + "\n")


def push_to_bot(title: str, link: str, author: str = ""):
    """通过 Admin API 推送给微信 Bot"""
    if not all([BOT_ADMIN_URL, BOT_ADMIN_TOKEN, BOT_ID, BOT_USER_ID]):
        return

    text = f"【RSS 新文章】\n{title}\n{link}"
    if author:
        text = f"【RSS 新文章】\n{title}\nby {author}\n{link}"

    try:
        resp = requests.post(
            f"{BOT_ADMIN_URL}/admin/api/bots/{BOT_ID}/send",
            headers={"Authorization": f"Bearer {BOT_ADMIN_TOKEN}"},
            json={"user_id": BOT_USER_ID, "text": text},
            timeout=10,
        )
        if resp.ok:
            print(f"  ✅ Bot 推送成功: {title}")
        else:
            print(f"  ⚠️ Bot 推送失败 ({resp.status_code}): {resp.text[:100]}")
    except Exception as e:
        print(f"  ⚠️ Bot 推送异常: {e}")


def check_rss(sent_ids: set[str]) -> set[str]:
    print(f"[{datetime.now():%H:%M:%S}] 🔍 检查 FreshRSS...")
    feed = feedparser.parse(FRESHRSS_RSS_URL)

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

        # 推送到 Bot
        push_to_bot(title, link, author)

        sent_ids.add(aid)
        new_count += 1

    if new_count == 0:
        print(f"  无新文章")
    else:
        print(f"  ✅ 发现 {new_count} 篇新文章")

    return sent_ids


def main():
    print("🚀 FreshRSS 监控启动")
    print(f"   目标: {FRESHRSS_RSS_URL}")
    print(f"   间隔: {CHECK_INTERVAL}s")
    print(f"   记录: {HISTORY_FILE}")

    if all([BOT_ADMIN_URL, BOT_ADMIN_TOKEN, BOT_ID, BOT_USER_ID]):
        print(f"   Bot 推送: ON → {BOT_ADMIN_URL}")
    else:
        print(f"   Bot 推送: OFF（配置 BOT_* 变量后开启）")
    print()

    sent_ids = load_history()
    print(f"   已读文章: {len(sent_ids)} 篇")
    print()

    while True:
        try:
            sent_ids = check_rss(sent_ids)
            save_history(sent_ids)
        except Exception as e:
            print(f"  ❌ 错误: {e}")
        time.sleep(CHECK_INTERVAL)


if __name__ == "__main__":
    main()
