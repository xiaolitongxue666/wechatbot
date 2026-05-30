"""wechatbot Skill Framework

Skills are self-contained Python modules under skills/<name>/.
Each skill exports a `skill` instance of a SkillBase subclass.
"""

from .base import SkillBase, BotClient
from .run import main as run_skill, list_skills, discover_skills

__all__ = ["SkillBase", "BotClient", "run_skill", "list_skills", "discover_skills"]
