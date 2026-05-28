# AGENTS.md

完整开发规范见 **[docs/AGENTS.md](docs/AGENTS.md)**。

**本仓库 Agent 记忆（非全局 Cursor skill）：**

| 文档 | 内容 |
|------|------|
| [docs/rust/agent-memory.md](docs/rust/agent-memory.md) | 目录结构、禁止事项、**问题与解法总表** |
| [docs/rust/troubleshooting.md](docs/rust/troubleshooting.md) | 排障步骤、旧路径对照、测试顺序 |
| [docs/rust/testing.md](docs/rust/testing.md) | 单元 / 集成 / E2E 命令 |

重组后关键命令：`bash tools/scripts/...` · 前端 `admin/web/` · E2E 用 `bash tools/scripts/test/run_e2e.sh`。
