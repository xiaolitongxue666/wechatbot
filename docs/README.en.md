# WeChatBot

[中文文档](README.md)

Monorepo centered on the **Rust** iLink Bot stack at the repository root. Reference SDKs live under [`reference-sdks/`](../reference-sdks/); archived projects under [`legacy/`](../legacy/).

## Quick Start

```bash
cp .env.example .env
bash tools/scripts/start.sh
```

Admin UI: **http://127.0.0.1:8787/admin**

All `cargo` and `bash tools/scripts/*` commands run from the **repo root**.

## SDKs

| SDK | Location | Install |
|-----|----------|---------|
| **Rust (primary)** | repo root | `cargo add wechatbot` |
| Node.js | `reference-sdks/nodejs/` | `npm install @wechatbot/wechatbot` |
| Python | `reference-sdks/python/` | `pip install wechatbot-sdk` |
| Go | `reference-sdks/golang/` | `go get github.com/corespeed-io/wechatbot/golang` |
| Pi extension | `legacy/pi-agent/` | `pi install npm:@wechatbot/pi-agent` |

## Prebuilt binaries

```bash
curl -fsSL https://raw.githubusercontent.com/corespeed-io/wechatbot/main/install.sh | bash
```

See [GitHub Releases](https://github.com/corespeed-io/wechatbot/releases).

## Docs

- [rust/README.md](rust/README.md) — Rust ops & testing
- [rust/troubleshooting.md](rust/troubleshooting.md) — Troubleshooting (2026-05 reorg)
- [protocol.md](protocol.md) — Protocol reference
- [AGENTS.md](AGENTS.md) — Contributor & agent conventions

## License

MIT
