from __future__ import annotations

import asyncio
import sys
from typing import Any, Optional

import aiohttp

from .ai_client import AIClient
from .config import Settings
from .message_handler import MessageHandler
from plugins.plugin_manager import PluginManager
from utils.logger import setup_logger


class WeChatBot:
    def __init__(self, settings: Settings) -> None:
        self._settings = settings
        self._ai = AIClient(settings)
        self._plugins = PluginManager()
        self._handler = MessageHandler(self._ai, self._plugins)
        self._http: Optional[aiohttp.ClientSession] = None

    @property
    def plugins(self) -> PluginManager:
        return self._plugins

    def register_plugin(self, plugin: Any) -> None:
        self._plugins.register(plugin)

    async def _get_auth_headers(self) -> dict[str, str]:
        import base64
        import os
        import struct

        token = self._settings.wechat_bot_token
        val = struct.unpack(">I", os.urandom(4))[0]
        uin = base64.b64encode(str(val).encode("utf-8")).decode("ascii")

        return {
            "Content-Type": "application/json",
            "AuthorizationType": "ilink_bot_token",
            "Authorization": f"Bearer {token}",
            "X-WECHAT-UIN": uin,
        }

    async def start(self) -> None:
        logger = setup_logger(self._settings.log_level)
        logger.info("WeChatBot starting...")

        self._plugins.load_builtin()

        self._http = aiohttp.ClientSession()
        try:
            await self._run_poll_loop()
        finally:
            await self._http.close()

    async def _run_poll_loop(self) -> None:
        base_url = self._settings.wechat_bot_base_url.rstrip("/")

        while True:
            try:
                headers = await self._get_auth_headers()
                async with self._http.post(
                    f"{base_url}/ilink/v2/bot/sync",
                    json={"channel_version": "2.0.0", "count": 10},
                    headers=headers,
                    timeout=aiohttp.ClientTimeout(total=30),
                ) as resp:
                    if resp.status == 200:
                        data = await resp.json()
                        messages = data.get("messages") or []
                        for msg in messages:
                            reply = await self._handler.handle(msg)
                            if reply:
                                await self._send_reply(msg, reply)
                    else:
                        text = await resp.text()
                        print(f"[wechatbot] HTTP {resp.status}: {text}", file=sys.stderr)

                await asyncio.sleep(1.0)

            except asyncio.CancelledError:
                break
            except Exception as e:
                print(f"[wechatbot] poll error: {e}", file=sys.stderr)
                await asyncio.sleep(3.0)

    async def _send_reply(self, original_msg: dict[str, Any], text: str) -> None:
        base_url = self._settings.wechat_bot_base_url.rstrip("/")
        headers = await self._get_auth_headers()

        payload = {
            "to_user": original_msg.get("from_user", ""),
            "msg_type": "text",
            "content": text,
        }

        async with self._http.post(
            f"{base_url}/ilink/v2/bot/send_message",
            json=payload,
            headers=headers,
            timeout=aiohttp.ClientTimeout(total=15),
        ) as resp:
            if resp.status != 200:
                text_body = await resp.text()
                print(
                    f"[wechatbot] send failed {resp.status}: {text_body}",
                    file=sys.stderr,
                )
