"""AI-powered WeChat chatbot — connects to OpenAI-compatible API.

Usage:
  python examples/ai_bot.py

Configuration via environment variables:
  OPENAI_API_KEY        — API key (or set in ~/.wechatbot/ai_bot_config.json)
  OPENAI_BASE_URL       — API base URL (default: https://api.openai.com/v1)
  OPENAI_MODEL          — Model name (default: gpt-4o)
  SYSTEM_PROMPT         — System prompt text
  MAX_HISTORY           — Max conversation turns to keep (default: 20)
  MAX_TOKENS            — Max response tokens (default: 2000)
  TEMPERATURE           — Response temperature (default: 0.7)
"""

import asyncio
import json
import os
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import aiohttp

from wechatbot import WeChatBot, IncomingMessage

# ── Config ──────────────────────────────────────────────────────────────

CONFIG_DIR = Path.home() / ".wechatbot"
CONFIG_FILE = CONFIG_DIR / "ai_bot_config.json"

DEFAULT_CONFIG: dict[str, Any] = {
    "api_key": None,
    "base_url": "https://api.openai.com/v1",
    "model": "gpt-4o",
    "system_prompt": (
        "You are a helpful AI assistant on WeChat. "
        "Keep responses concise and friendly. "
        "Use plain text without markdown formatting."
    ),
    "max_history": 20,
    "max_tokens": 2000,
    "temperature": 0.7,
}


def load_config() -> dict[str, Any]:
    cfg = {**DEFAULT_CONFIG}
    if CONFIG_FILE.exists():
        try:
            file_cfg = json.loads(CONFIG_FILE.read_text("utf-8"))
            cfg.update(file_cfg)
        except (json.JSONDecodeError, OSError):
            pass

    env_map = {
        "OPENAI_API_KEY": "api_key",
        "OPENAI_BASE_URL": "base_url",
        "OPENAI_MODEL": "model",
        "SYSTEM_PROMPT": "system_prompt",
        "MAX_HISTORY": "max_history",
        "MAX_TOKENS": "max_tokens",
        "TEMPERATURE": "temperature",
    }
    for env_var, key in env_map.items():
        val = os.environ.get(env_var)
        if val is not None:
            if key in ("max_history", "max_tokens"):
                cfg[key] = int(val)
            elif key == "temperature":
                cfg[key] = float(val)
            else:
                cfg[key] = val

    return cfg


# ── Markdown stripping ──────────────────────────────────────────────────

_MD_LINK = re.compile(r"\[([^\]]*)\]\([^\)]+\)")
_MD_BOLD = re.compile(r"\*\*(.+?)\*\*")
_MD_ITALIC = re.compile(r"\*(.+?)\*|_(.+?)_")
_MD_CODE_BLOCK = re.compile(r"```(?:\w*\n)?(.+?)```", re.DOTALL)
_MD_INLINE_CODE = re.compile(r"`([^`]+)`")
_MD_HEADING = re.compile(r"^#{1,6}\s+", re.MULTILINE)
_MD_LIST = re.compile(r"^[\*\-\+]\s+", re.MULTILINE)
_MD_ORDERED_LIST = re.compile(r"^\d+\.\s+", re.MULTILINE)
_MD_HR = re.compile(r"^-{3,}|_{3,}|\*{3,}$", re.MULTILINE)


def strip_markdown(text: str) -> str:
    text = _MD_LINK.sub(r"\1", text)
    text = _MD_BOLD.sub(r"\1", text)
    text = _MD_ITALIC.sub(r"\1", text)
    text = _MD_CODE_BLOCK.sub(r"\1", text)
    text = _MD_INLINE_CODE.sub(r"\1", text)
    text = _MD_HEADING.sub("", text)
    text = _MD_LIST.sub("• ", text)
    text = _MD_ORDERED_LIST.sub("", text)
    text = _MD_HR.sub("", text)
    text = re.sub(r"\n{3,}", "\n\n", text)
    return text.strip()


# ── Conversation history ────────────────────────────────────────────────

def trim_history(history: list[dict[str, Any]], max_turns: int) -> list[dict[str, Any]]:
    if not history:
        return history
    # Always keep system message
    result = []
    if history[0].get("role") == "system":
        result.append(history[0])
        non_system = history[1:]
    else:
        non_system = history

    # Keep last N turns (2 messages per turn: user + assistant)
    max_msgs = max_turns * 2
    result.extend(non_system[-max_msgs:])
    return result


def build_user_content(msg: IncomingMessage, config: dict[str, Any]) -> list[dict[str, Any]]:
    """Build content array for vision-capable models."""
    content: list[dict[str, Any]] = []

    # Add text
    if msg.text:
        content.append({"type": "text", "text": msg.text})

    # Add images for vision models
    if msg.images and msg.images[0].url:
        image_url = msg.images[0].url
        content.append({
            "type": "image_url",
            "image_url": {"url": image_url, "detail": "auto"},
        })

    if not content:
        content.append({"type": "text", "text": "[non-text message]"})

    return content


# ── AI API call ─────────────────────────────────────────────────────────

async def call_ai(
    session: aiohttp.ClientSession,
    config: dict[str, Any],
    messages: list[dict[str, Any]],
) -> str | None:
    """Call OpenAI-compatible chat completions API. Returns response text."""
    url = f"{config['base_url'].rstrip('/')}/chat/completions"

    async with session.post(
        url,
        headers={
            "Authorization": f"Bearer {config['api_key']}",
            "Content-Type": "application/json",
        },
        json={
            "model": config["model"],
            "messages": messages,
            "max_tokens": config["max_tokens"],
            "temperature": config["temperature"],
        },
        timeout=aiohttp.ClientTimeout(total=60),
    ) as resp:
        data = await resp.json()

        if resp.status >= 400:
            err_msg = data.get("error", {}).get("message", f"HTTP {resp.status}")
            print(f"[ai_bot] AI API error: {err_msg}", file=sys.stderr)
            return None

        choices = data.get("choices", [])
        if not choices:
            return None

        return choices[0].get("message", {}).get("content")


# ── Session cleanup ─────────────────────────────────────────────────────

async def cleanup_old_sessions(sessions: dict[str, dict[str, Any]], max_age_hours: int = 24):
    """Remove sessions older than max_age_hours."""
    now = datetime.now(timezone.utc)
    stale = []
    for uid, sess in sessions.items():
        age = (now - sess["last_active"]).total_seconds() / 3600
        if age > max_age_hours:
            stale.append(uid)
    for uid in stale:
        del sessions[uid]


# ── Main ────────────────────────────────────────────────────────────────

async def main():
    config = load_config()

    if not config["api_key"]:
        print("ERROR: No API key configured.", file=sys.stderr)
        print("Set OPENAI_API_KEY env var or create ~/.wechatbot/ai_bot_config.json", file=sys.stderr)
        print(f"Example config file at {CONFIG_FILE}:", file=sys.stderr)
        print(json.dumps({**DEFAULT_CONFIG, "api_key": "sk-your-key"}, indent=2, ensure_ascii=False))
        sys.exit(1)

    sessions: dict[str, dict[str, Any]] = {}

    bot = WeChatBot(
        on_qr_url=lambda url: print(f"\nScan QR URL:\n{url}\n"),
        on_scanned=lambda: print("QR scanned — confirm in WeChat"),
        on_expired=lambda: print("QR expired — requesting new one"),
        on_error=lambda err: print(f"Bot error: {err}", file=sys.stderr),
    )

    creds = await bot.login()
    print(f"Logged in: {creds.account_id}")

    @bot.on_message
    async def handle(msg: IncomingMessage):
        uid = msg.user_id

        # Init or get session
        if uid not in sessions:
            sessions[uid] = {
                "history": [
                    {"role": "system", "content": config["system_prompt"]}
                ],
                "last_active": datetime.now(timezone.utc),
            }
            print(f"[ai_bot] New session for {uid}")

        sess = sessions[uid]
        sess["last_active"] = datetime.now(timezone.utc)

        # Show typing indicator
        await bot.send_typing(uid)

        try:
            # Build user message content (supports vision)
            user_content = build_user_content(msg, config)
            sess["history"].append({"role": "user", "content": user_content})

            # Trim history
            sess["history"] = trim_history(sess["history"], config["max_history"])

            # Call AI
            async with aiohttp.ClientSession() as http_session:
                reply_text = await call_ai(http_session, config, sess["history"])

            if reply_text:
                reply_text = strip_markdown(reply_text)
                sess["history"].append({"role": "assistant", "content": reply_text})
                await bot.reply(msg, reply_text)
            else:
                await bot.reply(msg, "Sorry, I couldn't process that request.")

        except Exception as e:
            print(f"[ai_bot] Error handling message: {e}", file=sys.stderr)
            try:
                await bot.reply(msg, "An error occurred. Please try again later.")
            except Exception:
                pass

        finally:
            try:
                await bot.stop_typing(uid)
            except Exception:
                pass

        # Periodic cleanup (every 100 messages)
        if len(sessions) % 100 == 0:
            await cleanup_old_sessions(sessions)

    print(f"AI Bot started (model: {config['model']})")
    print("Press Ctrl+C to stop")

    try:
        await bot.start()
    except KeyboardInterrupt:
        bot.stop()
    finally:
        print(f"Stopped. Active sessions: {len(sessions)}")


if __name__ == "__main__":
    asyncio.run(main())
