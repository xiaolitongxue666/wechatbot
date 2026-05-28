# 产品待办（从 plan_skill 归档）

## Bot 在线状态

- 查询：Bot 多久没有对话会从 online 变为 offline？

## Bot 详情页

### 问题

1. 扫码成功后状态没有实时刷新
2. offline 之后重新进入详情页，看不到二维码，也无法恢复 online

### 目标

1. 新建 bot，扫码后自动更新状态为 online
2. 详情页恢复「启动」按钮：online 时禁用；首次扫码前禁用；曾扫码但 offline 时可点击启动
