# 项目分析、搭建与测试流程

## 1. 项目分析入口

建议按以下顺序阅读代码与文档：

1. `src/lib.rs`：公共导出与模块边界。
2. `src/runtime.rs`：运行时装配与依赖连接。
3. `src/admin/server.rs`：管理后台路由与服务启动。
4. `src/bot.rs`：协议事件处理、登录与消息主流程。
5. `code-analysis.md`：关键链路与扩展风险。

## 2. 本地搭建流程

在仓库根目录执行。

### 2.1 测试 / 演示（带 mock 数据，默认）

```bash
bash tools/scripts/start.sh
# 全栈 admin + worker：
bash tools/scripts/start_all.sh
```

### 2.2 部署（不带 mock 数据）

```bash
bash tools/scripts/start.sh --deploy
# 或手动：services.sh up → db.sh migrate（勿 seed）→ admin.sh start
```

### 2.3 其他

```bash
# 仅协议回环验证
bash tools/scripts/dev.sh

# 不启动 admin，仅准备环境（仍按 --dev/--deploy 决定是否 seed）
bash tools/scripts/start.sh --deploy --no-admin
```

常用参数：

```bash
# 显式 dev（与默认相同）
bash tools/scripts/start.sh --dev

# deploy 别名
bash tools/scripts/start.sh --no-seed
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
bash tools/scripts/test.sh --no-nextest
```

### 3.3 全量集成测试（需要 Docker）

```bash
bash tools/scripts/test_all.sh
```

## 4. 常见问题

### 缺少测试数据库环境变量

症状：`cargo test` 执行集成测试时报 `WECHATBOT_TEST_DATABASE_URL is required`。

处理：

1. 使用 `bash tools/scripts/test_all.sh` 自动拉起测试依赖并注入环境。
2. 或手动设置 `WECHATBOT_TEST_DATABASE_URL` 后再运行目标测试。

### 本地环境文件误提交风险

- 本地开发使用的 `.env` 不应提交。
- 已在根 `.gitignore` 增加 `.env` 与临时运行文件忽略规则。
