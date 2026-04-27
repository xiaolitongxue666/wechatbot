# Warning Fix Guideline Skill

## 目标

保持 `cargo build` 与 `cargo test --no-run` 无 warning。

## 处理原则

1. 优先删除未使用代码，而不是直接 `#[allow]`。
2. 仅在测试夹具或跨目标复用代码上局部使用 `#[allow(dead_code)]`。
3. 不使用 crate 级宽泛 `allow` 隐藏真实问题。
4. 修复 warning 时不得改变协议行为与外部接口语义。

## 执行流程

```bash
cd rust
cargo build
cargo test --no-run
```

若出现 warning：

1. 按文件归类（unused import / dead_code / clippy 建议）。
2. 逐文件修复并单次提交同类问题。
3. 每轮修复后重复运行 `cargo build` 与 `cargo test --no-run`。

## 当前约束

- 集成测试运行需要测试数据库环境，不应将运行失败与 warning 混为一类。
- 日志打印保持英文，注释保持中文并仅保留必要说明。
