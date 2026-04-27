from __future__ import annotations

from abc import ABC, abstractmethod
from typing import Any, Optional


class BasePlugin(ABC):
    name: str = "base"
    description: str = ""

    async def on_start(self) -> None:
        pass

    async def on_stop(self) -> None:
        pass

    @abstractmethod
    async def on_message(self, message: dict[str, Any]) -> Optional[str]:
        ...
