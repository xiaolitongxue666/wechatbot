#!/usr/bin/env bash
# ==============================================================================
# Skill 生命周期管理
#
# Usage:
#   bash tools/scripts/skill.sh list                          # 列出可用 skill
#   bash tools/scripts/skill.sh start <name>                   # 启动 skill（前台）
#   bash tools/scripts/skill.sh start-bg <name>                # 启动 skill（后台 nohup）
#   bash tools/scripts/skill.sh stop <name>                    # 停止后台 skill
#   bash tools/scripts/skill.sh status                         # 查看运行中的 skill
#
# 环境变量（也可在 config/app.toml 的 [skills] 段设置）:
#   FRESHRSS_RSS_URL   — FreshRSS 聚合 RSS URL
#   BOT_ADMIN_URL      — Admin API 地址（默认 http://127.0.0.1:8787）
#   BOT_ADMIN_TOKEN    — Admin API Token
#   BOT_ID             — Bot 标识
#   BOT_USER_ID        — 推送目标微信用户
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
SKILLS_DIR="$PROJECT_DIR/skills"

PID_DIR="${PID_DIR:-$PROJECT_DIR}"
CMD="${1:-help}"
SKILL_NAME="${2:-}"

case "$CMD" in
  list)
    echo "Discovered skills:"
    cd "$PROJECT_DIR"
    python3 -m skills.run list 2>/dev/null || python3 -c "
import sys; sys.path.insert(0, '$PROJECT_DIR')
from skills.run import list_skills
for s in list_skills():
    print(f'  {s[\"name\"]:20s}  {s.get(\"description\", \"\")}')
"
    ;;

  start)
    if [ -z "$SKILL_NAME" ]; then echo "Usage: $0 start <skill_name>" >&2; exit 1; fi
    cd "$PROJECT_DIR"
    echo "Starting skill: $SKILL_NAME"
    python3 -m skills.run "$SKILL_NAME"
    ;;

  start-bg)
    if [ -z "$SKILL_NAME" ]; then echo "Usage: $0 start-bg <skill_name>" >&2; exit 1; fi
    cd "$PROJECT_DIR"
    LOGFILE="$PID_DIR/.skill-${SKILL_NAME}.log"
    PIDFILE="$PID_DIR/.skill-${SKILL_NAME}.pid"
    nohup python3 -m skills.run "$SKILL_NAME" > "$LOGFILE" 2>&1 &
    echo $! > "$PIDFILE"
    echo "Started skill '$SKILL_NAME' (PID $(cat $PIDFILE)) — log: $LOGFILE"
    ;;

  stop)
    if [ -z "$SKILL_NAME" ]; then echo "Usage: $0 stop <skill_name>" >&2; exit 1; fi
    PIDFILE="$PID_DIR/.skill-${SKILL_NAME}.pid"
    if [ -f "$PIDFILE" ]; then
      PID=$(cat "$PIDFILE")
      kill "$PID" 2>/dev/null && echo "Stopped skill '$SKILL_NAME' (PID $PID)" || echo "Skill '$SKILL_NAME' not running"
      rm -f "$PIDFILE"
    else
      echo "No PID file for skill '$SKILL_NAME' (not running?)"
    fi
    ;;

  status)
    echo "Running skills:"
    for pidfile in "$PID_DIR"/.skill-*.pid; do
      [ -f "$pidfile" ] || continue
      NAME=$(basename "$pidfile" | sed 's/^\.skill-//; s/\.pid$//')
      PID=$(cat "$pidfile")
      if kill -0 "$PID" 2>/dev/null; then
        echo "  $NAME (PID $PID) — running"
      else
        echo "  $NAME (PID $PID) — stale (pid file exists but process dead)"
      fi
    done
    ;;

  help|*)
    sed -n '2,/^$/{ s/^# \?//p }' "$0"
    ;;
esac
