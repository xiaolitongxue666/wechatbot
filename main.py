from __future__ import annotations

import asyncio
import sys

from core.config import load_settings
from core.wechat_bot import WeChatBot


def main() -> None:
    settings = load_settings()

    if not settings.ai_api_key:
        print("Error: AI_API_KEY is not set. Copy .env.example to .env and fill in your keys.", file=sys.stderr)
        sys.exit(1)

    if not settings.wechat_bot_token:
        print("Warning: WECHAT_BOT_TOKEN is not set. The bot cannot connect to WeChat.", file=sys.stderr)

    bot = WeChatBot(settings)

    try:
        asyncio.run(bot.start())
    except KeyboardInterrupt:
        print("\n[wechatbot] Shutting down...", file=sys.stderr)


if __name__ == "__main__":
    main()
