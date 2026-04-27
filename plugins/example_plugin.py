from __future__ import annotations

from typing import Any, Optional

from .base_plugin import BasePlugin


class ExamplePlugin(BasePlugin):
    name = "example"
    description = "A demo plugin that responds to /hello and /ping commands"

    async def on_message(self, message: dict[str, Any]) -> Optional[str]:
        text = message.get("content", message.get("text", ""))

        if not isinstance(text, str):
            return None

        lowered = text.strip().lower()

        if lowered == "/hello":
            return "Hello! I am a WeChat bot powered by AI."

        if lowered == "/ping":
            return "Pong!"

        if lowered == "/help":
            return (
                "Available commands:\n"
                "/hello - Say hello\n"
                "/ping  - Check if I'm alive\n"
                "/help  - Show this message\n"
                "Or just chat with me!"
            )

        return None
