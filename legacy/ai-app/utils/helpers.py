from __future__ import annotations

import hashlib
import re
from typing import Any


def truncate(text: str, max_len: int = 200, suffix: str = "...") -> str:
    if len(text) <= max_len:
        return text
    return text[:max_len - len(suffix)] + suffix


def sanitize_text(text: str) -> str:
    return text.strip()


def is_command(text: str, prefix: str = "/") -> bool:
    return bool(text) and text[0] == prefix


def extract_command(text: str) -> tuple[str, str]:
    parts = text.strip().split(maxsplit=1)
    cmd = parts[0].lstrip("/").lower() if parts else ""
    arg = parts[1] if len(parts) > 1 else ""
    return cmd, arg


def content_hash(content: str) -> str:
    return hashlib.md5(content.encode("utf-8")).hexdigest()


def parse_message_content(msg: dict[str, Any]) -> str:
    return msg.get("content", msg.get("text", ""))


def parse_user_id(msg: dict[str, Any]) -> str:
    return msg.get("from_user", msg.get("fromUser", msg.get("user_id", "")))
