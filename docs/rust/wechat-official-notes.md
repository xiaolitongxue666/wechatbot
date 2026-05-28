# 微信官方机器人用户标识说明

（原 `wechat_official/wechat_bot_id.md`）

## 1. 唯一能用来识别用户的核心字段

**机器人唯一用户 ID**：`user_id` / `from_user_id`

格式示例：`wxid_xxxxxxx@im.wechat`

### 特性

1. **对单个 Bot 永久唯一** — 同一用户添加你的 Bot 后 ID 不变，可用于会话、权限、黑名单等。
2. **Bot 间隔离** — 换一个 Bot，同一人的 ID 完全不同；不能跨 Bot / 小程序 / 公众号打通。

## 2. 无法获取的信息

- 真实微信号、原始 wxid
- UnionID、公众号 OpenID、小程序 OpenID
- 头像原图、完整昵称、手机号、性别等
- 无法通过该 ID 反查真人

## 3. 辅助字段（勿作唯一 ID）

- 昵称：可随时修改
- 头像 key：可更换

## 总结

- **可靠唯一标识**：`user_id`（`@im.wechat` 后缀）
- **范围**：仅当前 Bot 内有效
