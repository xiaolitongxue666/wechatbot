# 配置说明

主配置文件位于 `rust/config/app.toml`。

## 数据库模式切换
- `database.mode=local`：使用 `database.local_url`
- `database.mode=container`：使用 `database.container_url`
- `database.mode=remote`：使用 `database.remote_url`

Redis 配置同理，通过 `redis.mode` 切换对应 URL。

## 媒体存储
- `media.backend=localfs`：媒体落地到 `media.local_root`
- `media.backend=s3`：使用 `media.bucket` + `media.endpoint`

## 转发配置
- `forwarder.endpoint`：目标 webhook 地址
- `forwarder.hmac_secret`：请求签名密钥
- `forwarder.max_retries`：最大重试次数
- `forwarder.timeout_ms`：单次请求超时

配置解析为 fail-fast，关键字段缺失时会直接启动失败。
