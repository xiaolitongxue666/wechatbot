"""Media-handling bot — demonstrates download, save, and echo-back of all media types.

Usage:
  python examples/media_bot.py

Features:
  - Download images, files, videos, voice messages
  - Save to ~/.wechatbot/media_received/ directory
  - Echo back: images (forward), files (rename and send back), voice (save silk)
  - Count and report statistics
"""

import asyncio
from datetime import datetime
from pathlib import Path

from wechatbot import WeChatBot, IncomingMessage

MEDIA_DIR = Path.home() / ".wechatbot" / "media_received"


async def main():
    MEDIA_DIR.mkdir(parents=True, exist_ok=True)

    stats = {"text": 0, "image": 0, "file": 0, "video": 0, "voice": 0}

    bot = WeChatBot(
        on_qr_url=lambda url: print(f"\nScan QR URL:\n{url}\n"),
        on_error=lambda err: print(f"Error: {err}"),
    )

    creds = await bot.login()
    print(f"Logged in: {creds.account_id}")

    @bot.on_message
    async def handle(msg: IncomingMessage):
        stats[msg.type] = stats.get(msg.type, 0) + 1
        now = datetime.now().strftime("%Y%m%d_%H%M%S")

        if msg.type == "text":
            await bot.reply(msg, f"[Media Bot] Text received: '{msg.text[:50]}...'")

        elif msg.type == "image":
            media = await bot.download(msg)
            if media:
                path = MEDIA_DIR / f"image_{now}.jpg"
                path.write_bytes(media.data)
                print(f"Saved image: {path} ({len(media.data)} bytes)")
                # Echo the image back
                await bot.reply_media(msg, {"image": media.data})

        elif msg.type == "file":
            media = await bot.download(msg)
            if media:
                filename = media.file_name or f"file_{now}.bin"
                path = MEDIA_DIR / f"file_{now}_{filename}"
                path.write_bytes(media.data)
                print(f"Saved file: {path} ({len(media.data)} bytes)")
                await bot.reply_media(msg, {
                    "file": media.data,
                    "file_name": f"echo_{filename}",
                    "caption": f"Received: {filename} ({len(media.data)} bytes)",
                })

        elif msg.type == "video":
            media = await bot.download(msg)
            if media:
                path = MEDIA_DIR / f"video_{now}.mp4"
                path.write_bytes(media.data)
                print(f"Saved video: {path} ({len(media.data)} bytes)")
                await bot.reply(msg, f"[Media Bot] Video saved ({len(media.data)} bytes)")

        elif msg.type == "voice":
            media = await bot.download(msg)
            if media:
                path = MEDIA_DIR / f"voice_{now}.silk"
                path.write_bytes(media.data)
                print(f"Saved voice: {path} ({len(media.data)} bytes)")
                await bot.reply(msg, f"[Media Bot] Voice saved ({len(media.data)} bytes, SILK format)")

        print(f"Stats: {stats}")

    print(f"Media Bot started. Files saved to {MEDIA_DIR}")
    print("Press Ctrl+C to stop")

    try:
        await bot.start()
    except KeyboardInterrupt:
        bot.stop()
    print(f"Stopped. Final stats: {stats}")


if __name__ == "__main__":
    asyncio.run(main())
