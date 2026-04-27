# 项目分析、搭建与测试流程

## 1. 项目分析入口

建议按以下顺序阅读代码与文档：

1. `src/lib.rs`：公共导出与模块边界。
2. `src/runtime.rs`：运行时装配与依赖连接。
3. `src/admin/server.rs`：管理后台路由与服务启动。
4. `src/bot.rs`：协议事件处理、登录与消息主流程。
5. `doc/code-analysis.md`：关键链路与扩展风险。

## 2. 本地搭建流程

在 `rust/` 目录执行：

```bash
# 一键启动依赖、迁移、可选种子、后台服务
bash scripts/start.sh

# 仅启动协议回环验证示例
bash scripts/dev.sh
```

常用参数：

```bash
# 仅建表，不灌种
bash scripts/start.sh --no-seed

# 不启动 admin，仅完成环境准备
bash scripts/start.sh --no-admin
```

## 3. 测试流程

### 3.1 编译与告警门禁

```bash
cargo build
cargo test --no-run
```

- `cargo build`：验证主工程编译与库级 warning。
- `cargo test --no-run`：编译所有测试目标并检查 warning，不执行需要外部依赖的测试体。

### 3.2 单元测试（无外部依赖）

```bash
bash scripts/test.sh --no-nextest
```

### 3.3 全量集成测试（需要 Docker）

```bash
bash scripts/test_all.sh
```

## 4. 常见问题

### 缺少测试数据库环境变量

症状：`cargo test` 执行集成测试时报 `WECHATBOT_TEST_DATABASE_URL is required`。

处理：

1. 使用 `bash scripts/test_all.sh` 自动拉起测试依赖并注入环境。
2. 或手动设置 `WECHATBOT_TEST_DATABASE_URL` 后再运行目标测试。

### 本地环境文件误提交风险

- 本地开发使用的 `.env` 不应提交。
- 已在 `rust/.gitignore` 增加 `.env` 与临时运行文件忽略规则。
