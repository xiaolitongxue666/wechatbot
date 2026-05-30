#!/usr/bin/env python3
"""
freshrss2wxbot.py — 兼容入口，委派给 skills 框架。

用法:
    python freshrss2wxbot.py

等效于:
    python -m skills.run freshrss

配置见 skills/freshrss/ 或设置环境变量 FRESHRSS_RSS_URL / BOT_*
"""

import sys
import os

if __name__ == "__main__":
    from skills.run import main
    sys.argv = [sys.argv[0], "freshrss"]
    main()
