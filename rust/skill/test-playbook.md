# Test Playbook Skill

## 目标

在不同环境下稳定执行 Rust SDK 测试，并快速定位失败原因。

## 推荐执行顺序

```bash
cd rust
cargo build
cargo test --no-run
bash scripts/test.sh --no-nextest
```

## 全量集成测试

```bash
bash scripts/test_all.sh
```

## 常见失败与处理

### 缺少 `WECHATBOT_TEST_DATABASE_URL`

- 现象：集成测试启动即 panic。
- 处理：优先改用 `bash scripts/test_all.sh`。

### nextest 不存在

- 现象：脚本提示 `cargo-nextest not installed`。
- 处理：
  - 安装：`cargo install cargo-nextest --locked`
  - 或使用：`bash scripts/test.sh --no-nextest`

### 某个集成测试失败

- 定位命令：`cargo test --test <test_name> -- --nocapture`
- 建议先检查数据库和 Redis 连通性，再检查测试数据准备流程。
