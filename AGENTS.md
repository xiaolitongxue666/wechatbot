# AGENTS.md — WeChatBot Project Conventions

## Project Overview

Monorepo with 4 independent SDK implementations of the WeChat iLink Bot protocol, plus a Pi extension. Each SDK is a self-contained package in its own subdirectory.

```
wechatbot/
├── nodejs/       # TypeScript SDK → @wechatbot/wechatbot (npm)
├── python/       # Python SDK    → wechatbot-sdk        (PyPI)
├── golang/       # Go SDK       → wechatbot             (Go module)
├── rust/         # Rust SDK     → wechatbot             (Cargo)
├── pi-agent/     # Pi extension → @wechatbot/pi-agent   (npm)
└── docs/         # Shared protocol & architecture docs
```

## Working Directory

All dev commands MUST be run from the relevant SDK subdirectory, NOT the repo root:

| SDK      | Working Directory (use `--workdir` or `cd`) |
|----------|---------------------------------------------|
| Node.js  | `nodejs/`                                   |
| Python   | `python/`                                   |
| Go       | `golang/`                                   |
| Rust     | `rust/`                                     |
| Pi Agent | `pi-agent/`                                 |

## Commands by SDK

### Node.js (`nodejs/`)

| Task          | Command                                    |
|---------------|--------------------------------------------|
| Install       | `npm install`                              |
| Build         | `npm run build` (`tsc`)                    |
| Watch build   | `npm run dev` (`tsc --watch`)              |
| Test          | `npm test`                                 |
| Test (watch)  | `npm run test:watch`                       |
| Lint/Typecheck| `npm run lint` (`tsc --noEmit`)            |
| CI test       | `npx vitest run` (via NODE_OPTIONS=--experimental-vm-modules) |

- TypeScript 5.x, ESM-only (`"type": "module"`, `NodeNext` module resolution)
- Zero runtime dependencies; devDeps: `typescript`, `vitest`, `@types/node`
- Requires Node.js >= 22
- No ESLint/Prettier configured; TypeScript compiler is the sole quality gate

### Python (`python/`)

| Task        | Command                     |
|-------------|-----------------------------|
| Install     | `pip install -e ".[dev]"`   |
| Test        | `pytest`                    |
| Build       | `python -m build` (hatchling backend) |

- Uses `uv` for dependency management (`uv.lock`)
- Python >= 3.9; dev requires 3.12
- Runtime deps: `aiohttp`, `cryptography`; dev deps: `pytest`, `pytest-asyncio`
- **No linter, formatter, or type checker configured** (no ruff, mypy, black)

### Go (`golang/`)

| Task     | Command               |
|----------|-----------------------|
| Build    | `go build ./...`      |
| Test     | `go test ./...`       |
| Vet      | `go vet ./...`        |
| Format   | `gofmt -w ./...`      |

- Module: `github.com/corespeed-io/wechatbot/golang`
- Go 1.22+, **zero external dependencies** (stdlib only)
- No `.golangci.yml` or third-party linters configured

### Rust (`rust/`)

| Task          | Command                                              |
|---------------|------------------------------------------------------|
| Build         | `cargo build`                                        |
| Test (unit)   | `cargo test` or `bash scripts/test.sh`               |
| Test (full)   | `bash scripts/test_all.sh` (requires Docker)         |
| Run admin     | `cargo run --bin admin`                              |
| Start env     | `bash scripts/start.sh`                              |
| Dev verify    | `bash scripts/dev.sh`                                |

- Edition 2021; binary: `admin`; examples: `echo_bot`, `multi_bot_runtime`
- Full integration tests require PostgreSQL + Redis + MinIO (Docker Compose)
- **No `cargo clippy` or `cargo fmt --check` in CI**; no clippy.toml or rustfmt.toml
- CI runs only `cargo build && cargo test` (unit tests, no docker services)

### Pi Agent (`pi-agent/`)

| Task      | Command              |
|-----------|----------------------|
| Install   | `npm install`        |
| Lint      | `tsc --noEmit`       |

- Depends on `@wechatbot/wechatbot` (the Node.js SDK)

## Code Conventions (All SDKs)

- **Section separators**: ASCII-art comment blocks used as visual separators between logical sections
- **No comments on implementation details** unless absolutely necessary; public APIs get doc comments
- **Error hierarchy**: `WeChatBotError` base class/struct with `code` property; subclasses for `ApiError` (with `isSessionExpired` / `errcode === -14` check), `AuthError`, `MediaError`, `NoContextError`
- **Logging**: `[wechatbot]` prefix, output to stderr
- **Credentials**: Stored at `~/.wechatbot/credentials.json`; auto re-login on session expiry (`-14`)
- **Colors**: `^[[1;33m` (bold yellow) for QR codes in terminal, `^[[0m` for reset

### Node.js Specific
- Named exports only (no default exports)
- `.js` extension on all relative ESM imports (required by `NodeNext`)
- `node:` prefix for built-in modules
- `type` imports for type-only references
- Fluent builder pattern for message construction (`MessageBuilder.to(...).text(...).build()`)
- Middleware: Koa/Express-style `(ctx, next) => void`

### Python Specific
- `from __future__ import annotations` in every module
- Modern union syntax (`str | None`, not `Optional[str]`)
- `@dataclass` for domain objects
- Private helpers prefixed with `_`
- Section headers: `# ── Section ──`

### Go Specific
- `sync.Map` for concurrency, `sync.Mutex` for credentials/state
- `context.Context` propagated to all network calls
- Table-driven tests with `t.Errorf`/`t.Fatal`
- Exported: PascalCase; unexported: camelCase

### Rust Specific
- `thiserror` for error enum, `async_trait` for trait objects
- `#[serde(rename = "camelCase")]` for JSON field names
- Integer enums use `serde_repr` with `#[repr(i32)]`
- `Arc<RwLock<>>` for shared mutable state
- `tracing` crate for structured logging (`info!`, `warn!`, `error!`)

## TDD / Testing

- All test files live in a `tests/` directory or inline `#[cfg(test)]` (Rust)
- Tests are the primary quality gate; linting/formatting is secondary or absent
- Node.js: 69 unit tests (vitest), Python: 18 tests (pytest), Go: table-driven stdlib tests, Rust: 50+ unit + integration tests
- CI runs on ubuntu, windows, macos across all SDKs

## Commit / PR Guidelines

- Commit messages should be concise, describing the "why" not the "what"
- The repo is public; never commit secrets, `.env` files, or credentials
- Each SDK publishes independently; changelogs are per-SDK
