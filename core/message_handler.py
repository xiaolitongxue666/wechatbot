from __future__ import annotations

import asyncio
from collections import defaultdict
from typing import Any, Callable, Optional

from .ai_client import AIClient
from plugins.plugin_manager import PluginManager


MessageHandler = Callable[[dict[str, Any]], Any]


class MessageHandler:
    def __init__(
        self,
        ai_client: AIClient,
        plugin_manager: PluginManager,
        *,
        max_history: int = 20,
    ) -> None:
        self._ai = ai_client
        self._plugins = plugin_manager
        self._max_history = max_history
        self._histories: dict[str, list[dict[str, str]]] = defaultdict(list)

    async def handle(self, message: dict[str, Any]) -> Optional[str]:
        handled = await self._plugins.handle(message)
        if handled is not None:
            return handled

        user_id = message.get("from_user", "")
        text = message.get("content", message.get("text", ""))

        if not text:
            return None

        history = self._histories[user_id]
        history.append({"role": "user", "content": text})

        reply = await self._ai.chat(list(history), stream=False)

        history.append({"role": "assistant", "content": reply})
        if len(history) > self._max_history:
            self._histories[user_id] = history[-self._max_history :]

        return reply

    def clear_history(self, user_id: str) -> None:
        self._histories.pop(user_id, None)
