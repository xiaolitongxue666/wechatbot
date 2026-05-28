# legacy

本目录存放**非 Rust 主工程**的附属项目：实验应用、Pi 扩展、无关工具与 Demo。不参与仓库根目录的默认 CI 路径。

| 子目录 | 说明 | 维护 |
|--------|------|------|
| [ai-app/](ai-app/) | 遗留 Python AI + 微信应用 | 归档，非 CI |
| [pi-agent/](pi-agent/) | Pi `/wechat` 桥接（npm `@wechatbot/pi-agent`） | 仍发布，`legacy/pi-agent-v*` |
| [devtools-bookmark/](devtools-bookmark/) | Chrome DevTools 书签扩展 | 归档，与微信无关 |
| [webchat/](webchat/) | 静态 AI 聊天前端 Demo | 归档，未接 iLink |

主工程见仓库根目录（`cargo build`、`admin/web/`）。
