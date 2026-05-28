# reference-sdks

本目录为 **WeChat iLink Bot 协议的参考实现**（Node.js / Python / Go），用于协议对照、跨语言测试与独立发布。**主工程为仓库根的 Rust 栈。**

| SDK | 目录 | 包名 / 模块 | 发布 tag |
|-----|------|-------------|----------|
| Node.js | [nodejs/](nodejs/) | `@wechatbot/wechatbot` | `node-v*` |
| Python | [python/](python/) | `wechatbot-sdk` | `py-v*` |
| Go | [golang/](golang/) | `github.com/corespeed-io/wechatbot/golang` | 随仓库 `v*` 二进制 |

## 开发

```bash
cd reference-sdks/nodejs && npm install && npm run build && npx vitest run
cd reference-sdks/python && pip install -e ".[dev]" && pytest
cd reference-sdks/golang && go test ./...
```

## 说明

- 各子目录 README 顶部有「参考实现」标注。
- 修改参考 SDK 不影响仓库根 Rust Admin；但 CI 会在 PR 中跑全语言矩阵。
- Go 下 `golang/blog-server/` 为独立示例服务，非主 SDK 必需部分。
