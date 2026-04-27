# Rust 文档目录与使用说明

本目录用于维护 Rust SDK 的架构、配置、部署、存储、运维、测试与代码分析文档。

## 文档结构

- `architecture.md`：系统架构与模块分层
- `configuration.md`：配置项说明与环境变量
- `deployment.md`：部署方式与环境准备
- `operations.md`：运维与日常操作手册
- `storage.md`：数据库、缓存、媒体存储设计
- `testing.md`：测试策略与测试命令
- `code-analysis.md`：核心代码链路与扩展风险分析
- `workflow.md`：项目分析、搭建与测试流程总览

## 推荐阅读路径

### 新同学快速上手

1. `architecture.md`
2. `configuration.md`
3. `deployment.md`
4. `operations.md`
5. `testing.md`
6. `code-analysis.md`

### 开发与调试

1. `configuration.md`
2. `code-analysis.md`
3. `testing.md`
4. `operations.md`

### 线上问题排查

1. `operations.md`
2. `storage.md`
3. `testing.md`
4. `code-analysis.md`

## 常用命令入口（在 `rust/` 目录执行）

### 启动与开发

- `bash scripts/start.sh`
- `bash scripts/start.sh --no-seed`
- `bash scripts/start.sh --no-admin`
- `bash scripts/dev.sh`
- `cargo run --bin admin`

### 测试

- `bash scripts/test.sh`
- `bash scripts/test_all.sh`
- `cargo test`

## 关键配置与路径

- 应用配置：`config/app.toml`
- 管理服务配置来源：`WECHATBOT_CONFIG`（未设置时使用默认配置路径）
- 本地凭据默认路径：`~/.wechatbot/credentials.json`

## 维护约定

- 新增文档时，必须同步更新本文件中的“文档结构”和“推荐阅读路径”。
- 命令说明优先引用 `scripts/*.sh`，避免不同环境下操作不一致。
- 文档中出现的模块名、命令、路径需与仓库实现保持一致。


## Skill 手册（`rust/skill/`）

- `build-and-run.md`：环境搭建与启动操作手册
- `test-playbook.md`：测试执行与故障排查手册
- `warning-fix-guideline.md`：零 warning 治理规则
