from __future__ import annotations

import os
from pathlib import Path
from typing import Optional

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        extra="ignore",
    )

    ai_api_key: str = ""
    ai_base_url: str = "https://api.openai.com/v1"
    ai_model: str = "gpt-4o"
    ai_system_prompt: str = "You are a helpful WeChat assistant."

    wechat_bot_token: str = ""
    wechat_bot_base_url: str = "https://ilinkai.weixin.qq.com"

    log_level: str = "INFO"


def load_settings(env_file: Optional[str] = None) -> Settings:
    if env_file:
        return Settings(_env_file=env_file)
    return Settings()
