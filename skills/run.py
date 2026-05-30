"""
CLI entry point for running wechatbot skills.

Usage:
    python -m skills.run <skill_name>
    python -m skills.list                        # list discovered skills

Environment variables (may also be set in config/app.toml [skills]):
    FRESHRSS_RSS_URL       — FreshRSS aggregated RSS URL
    RSS_CHECK_INTERVAL     — poll interval in seconds (default 60)
    BOT_ADMIN_URL          — Admin API base URL
    BOT_ADMIN_TOKEN        — Admin API Bearer token
    BOT_ID                 — Bot identifier
    BOT_USER_ID            — WeChat user to deliver to
"""

import importlib
import json
import os
import pathlib
import sys
import time
from typing import Any

from .base import SkillBase, _install_signal_handler

SKILLS_DIR = pathlib.Path(__file__).resolve().parent


def discover_skills() -> dict[str, dict[str, Any]]:
    """Scan skills/<name>/ for discoverable skills."""
    results: dict[str, dict[str, Any]] = {}
    for entry in sorted(SKILLS_DIR.iterdir()):
        if not entry.is_dir() or entry.name.startswith("_") or entry.name.startswith("."):
            continue
        init_file = entry / "__init__.py"
        meta_file = entry / "skill.json"
        meta: dict[str, Any] = {"name": entry.name, "path": str(entry)}
        if meta_file.exists():
            try:
                with open(meta_file, encoding="utf-8") as f:
                    meta.update(json.load(f))
            except Exception:
                pass
        meta["has_init"] = init_file.exists()
        results[entry.name] = meta
    return results


def list_skills() -> list[dict[str, Any]]:
    """Return discovered skills as a sorted list."""
    return list(discover_skills().values())


def load_skill(name: str) -> SkillBase:
    """Dynamically import skills/<name>/ and return its `skill` instance."""
    mod_name = f"skills.{name}"
    if mod_name not in sys.modules:
        importlib.import_module(mod_name)
    mod = sys.modules[mod_name]
    skill: SkillBase = getattr(mod, "skill", None)
    if skill is None:
        raise ImportError(f"skills/{name}/__init__.py must export a `skill` instance")
    if not isinstance(skill, SkillBase):
        raise TypeError(f"skills/{name}/__init__.py `skill` must be a SkillBase subclass")
    return skill


def main():
    """CLI entry: python -m skills.run <name>"""
    if len(sys.argv) < 2:
        print("Usage: python -m skills.run <skill_name>", file=sys.stderr)
        print(f"\nDiscovered skills: {[s['name'] for s in list_skills()]}", file=sys.stderr)
        sys.exit(1)

    name = sys.argv[1]

    if name == "--list" or name == "list":
        for s in list_skills():
            print(f"  {s['name']:20s}  {s.get('description', '')}")
        sys.exit(0)

    skill = load_skill(name)
    _install_signal_handler(skill)

    print(f"[skill]  starting: {skill.name} — {skill.description}")
    print(f"[skill]  admin={skill.client.admin_url}  bot={skill.client.bot_id}")
    print(f"[skill]  push={'ON' if skill.client.is_configured() else 'OFF'}")

    try:
        skill.run()
    except KeyboardInterrupt:
        pass
    finally:
        print(f"[skill]  stopped: {skill.name}")
