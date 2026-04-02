# WeChatBot 排障记录（Python / Rust）

## 1. Python 命令执行问题

### 现象
- `uv run python -c` 直接回车后报错：`Argument expected for the -c option`
- 使用单行 `-c` 串联 `async def` 报错：`SyntaxError: invalid syntax`

### 原因
- `-c` 后必须紧跟完整 Python 代码字符串。
- `async def` 不能在这种分号拼接单行里按该写法定义。

### 处理
- 改用 heredoc 方式执行：
  - `uv run python - <<'PY' ... PY`
- 验证结果：成功打印 `QR_URL=...` 并完成登录。

## 2. Python 登录后进程退出

### 现象
- 仅执行 `await bot.login(force=True)` 后程序结束，没有消息循环。

### 原因
- 该执行只做登录，不启动长轮询消息处理。

### 处理
- 使用示例启动消息循环：
  - `uv run python examples/echo_bot.py`

## 3. Rust 扫码登录与 Echo 示例问题

### 现象
- 初始示例可打印二维码但不回消息。
- 需要每次扫码。

### 原因
- 示例原本仅打印消息，没有回包逻辑。
- 示例使用 `login(true)` 时会强制重新扫码。

### 处理
- 在 `rust/examples/echo_bot.rs` 增加 Echo 处理（收到什么回什么）。
- 使用 `Arc + tokio::spawn` 在回调里发送异步回复。
- 默认改为复用本地凭据；需要强制重扫时使用 `FORCE_QR=1`。

## 4. Rust 收到消息时报 JSON 反序列化错误

### 现象
- 报错：`invalid type: integer 1, expected string or map`
- 后续报错：`missing field ret`

### 原因
- `getupdates` 返回里枚举字段是整型值（如 `1`、`2`），而强类型解析最初按字符串枚举处理。
- 部分响应缺失 `ret` 字段，结构体里却将 `ret` 设为必填。
- 引用消息 `ref_msg.message_item` 返回形态不稳定（不总是固定对象）。
- 文件长度 `len` 可能为字符串或数字。

### 处理
- `MessageType` / `MessageState` / `MessageItemType` 改为按整型枚举反序列化（`serde_repr`）。
- `GetUpdatesResponse.ret` 改为可选，仅在 `ret` 存在且非 0 时判错。
- `RefMessage.message_item` 改为宽松 `serde_json::Value` 并按路径提取引用文本。
- `FileItem.len` 兼容字符串与数字两种形态。
- 增加对应测试并通过 `cargo test`。

## 5. Windows 下编译链接占用问题

### 现象
- 偶发 `LNK1104: cannot open file ... echo_bot.exe`

### 原因
- 旧进程仍在运行，占用目标可执行文件。

### 处理
- 先停止运行中的 `echo_bot.exe`（`Ctrl+C`），再重新 `cargo run --example echo_bot`。
