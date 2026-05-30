"""
SkillBase — abstract base for all bot skills.
BotClient — helper to call the bot's Admin API.
"""

from __future__ import annotations

import json
import os
import signal
import sys
from abc import ABC, abstractmethod
from typing import Optional

import requests


class BotClient:
    """Wraps Admin API calls so skills don't need to manage URLs/tokens."""

    def __init__(
        self,
        admin_url: str = "",
        admin_token: str = "",
        bot_id: str = "",
        user_id: str = "",
    ):
        self.admin_url = admin_url or os.getenv("BOT_ADMIN_URL", "")
        self.admin_token = admin_token or os.getenv("BOT_ADMIN_TOKEN", "")
        self.bot_id = bot_id or os.getenv("BOT_ID", "")
        self.user_id = user_id or os.getenv("BOT_USER_ID", "")

    def is_configured(self) -> bool:
        return all([self.admin_url, self.admin_token, self.bot_id, self.user_id])

    def health(self) -> bool:
        try:
            resp = requests.get(f"{self.admin_url}/healthz", timeout=5)
            return resp.status_code == 200
        except Exception:
            return False

    def send(self, text: str) -> bool:
        """Send a text message to the bot's target user. Returns True on success."""
        if not self.is_configured():
            return False
        try:
            resp = requests.post(
                f"{self.admin_url}/admin/api/bots/{self.bot_id}/send",
                headers={"Authorization": f"Bearer {self.admin_token}"},
                json={"user_id": self.user_id, "text": text},
                timeout=10,
            )
            return resp.ok
        except Exception:
            return False

    def send_markdown(self, title: str, body: str = "", link: str = "") -> bool:
        """Send a well-formatted message. Returns True on success."""
        lines = [f"【{title}】"]
        if body:
            lines.append(body)
        if link:
            lines.append(link)
        return self.send("\n".join(lines))


class SkillBase(ABC):
    """Override run() with the skill's main loop.
    The loader calls skill.run() in a subprocess.
    """

    def __init__(self, name: str = "", description: str = ""):
        self.name = name or self.__class__.__name__.lower()
        self.description = description
        self._stop_flag = False
        self.client = BotClient()

    def set_client(self, client: BotClient):
        self.client = client

    def stop(self):
        self._stop_flag = True

    @property
    def should_stop(self) -> bool:
        return self._stop_flag or False

    @abstractmethod
    def run(self):
        """Main loop — override in subclass."""
        ...


def _install_signal_handler(skill: SkillBase):
    """Install SIGTERM/SIGINT handler so run() can exit cleanly."""

    def _handler(signum, frame):
        skill.stop()

    signal.signal(signal.SIGTERM, _handler)
    signal.signal(signal.SIGINT, _handler)
