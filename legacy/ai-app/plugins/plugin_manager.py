from __future__ import annotations

import importlib
import pkgutil
from typing import Any, Optional

from .base_plugin import BasePlugin


class PluginManager:
    def __init__(self) -> None:
        self._plugins: list[BasePlugin] = []

    def register(self, plugin: BasePlugin) -> None:
        self._plugins.append(plugin)

    def unregister(self, plugin: BasePlugin) -> None:
        self._plugins.remove(plugin)

    def load_builtin(self) -> None:
        from .example_plugin import ExamplePlugin

        self.register(ExamplePlugin())

    def discover_and_load(self, package_name: str = "plugins") -> None:
        package = importlib.import_module(package_name)
        for _, name, is_pkg in pkgutil.iter_modules(package.__path__):
            if is_pkg or name.startswith("_"):
                continue
            if name in ("base_plugin", "plugin_manager", "example_plugin"):
                continue
            module = importlib.import_module(f"{package_name}.{name}")
            for attr_name in dir(module):
                attr = getattr(module, attr_name)
                if (
                    isinstance(attr, type)
                    and issubclass(attr, BasePlugin)
                    and attr is not BasePlugin
                ):
                    self.register(attr())

    async def handle(self, message: dict[str, Any]) -> Optional[str]:
        for plugin in self._plugins:
            try:
                result = await plugin.on_message(message)
                if result is not None:
                    return result
            except Exception:
                pass
        return None

    async def on_start(self) -> None:
        for plugin in self._plugins:
            try:
                await plugin.on_start()
            except Exception:
                pass

    async def on_stop(self) -> None:
        for plugin in self._plugins:
            try:
                await plugin.on_stop()
            except Exception:
                pass

    @property
    def plugins(self) -> list[BasePlugin]:
        return list(self._plugins)
