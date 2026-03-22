/**
 * Pi Extension: WeChat Bridge
 *
 * Type /wechat or /weixin in pi → QR code appears in terminal →
 * scan with WeChat → your WeChat messages become pi prompts →
 * pi responses sent back to WeChat.
 *
 * Uses @wechatbot/wechatbot SDK for iLink protocol.
 * Uses qrcode-terminal for QR display.
 *
 * Install:
 *   pi -e /path/to/wechatbot/pi-agent/src/index.ts
 */

import type { ExtensionAPI } from '@mariozechner/pi-coding-agent'
import { WeChatBot, type IncomingMessage } from '@wechatbot/wechatbot'
import qrTerminal from 'qrcode-terminal'

export default function wechatBridge(pi: ExtensionAPI) {
  let bot: WeChatBot | null = null
  let connected = false
  let activeUserId: string | null = null
  let pendingReply: IncomingMessage | null = null
  let assistantText = ''
  let isStreaming = false

  // ── Capture pi response → send back to WeChat ────────────────────

  pi.on('agent_end', async (event, ctx) => {
    if (!bot || !connected || !pendingReply) return

    const reply = pendingReply
    pendingReply = null
    isStreaming = false

    // Extract assistant's final text
    const messages = event.messages ?? []
    let finalText = ''
    for (const msg of messages) {
      if (msg.role === 'assistant') {
        for (const block of msg.content) {
          if (block.type === 'text') finalText += block.text
        }
      }
    }
    if (!finalText.trim()) finalText = assistantText || '[No response]'

    try {
      await bot.stopTyping(reply.userId)
      await bot.reply(reply, finalText)
      ctx.ui.setStatus('wechat', `✓ Replied to WeChat`)
    } catch (e) {
      ctx.ui.setStatus('wechat', `✗ Reply failed: ${e instanceof Error ? e.message : e}`)
    }
    assistantText = ''
  })

  pi.on('message_update', async (event) => {
    if (!isStreaming) return
    if (event.message.role === 'assistant') {
      let text = ''
      for (const block of event.message.content) {
        if (block.type === 'text') text += block.text
      }
      assistantText = text
    }
  })

  // ── /wechat and /weixin commands ──────────────────────────────────

  const startWechat = async (args: string | undefined, ctx: any) => {
    // Already connected?
    if (connected && bot) {
      const action = await ctx.ui.select('WeChat is connected', [
        'Disconnect',
        'Status',
        'Cancel',
      ])
      if (action === 'Disconnect') {
        bot.stop()
        connected = false
        ctx.ui.setStatus('wechat', undefined)
        ctx.ui.notify('WeChat disconnected', 'info')
      } else if (action === 'Status') {
        const creds = bot.getCredentials()
        ctx.ui.notify(`Account: ${creds?.accountId}\nUser: ${creds?.userId}`, 'info')
      }
      return
    }

    // Create bot via SDK
    bot = new WeChatBot({ storage: 'file', logLevel: 'warn' })
    const forceLogin = args?.trim() === '--force'

    ctx.ui.notify('Starting WeChat login…', 'info')
    ctx.ui.setStatus('wechat', '⏳ Waiting for QR scan…')

    try {
      const creds = await bot.login({
        force: forceLogin,
        callbacks: {
          // ★ Render QR code in terminal using qrcode-terminal
          onQrUrl: (url) => {
            qrTerminal.generate(url, { small: true }, (qr: string) => {
              // Show QR in pi TUI widget
              const lines = [
                '📱 Scan this QR code in WeChat:',
                '',
                ...qr.split('\n'),
                '',
                `URL: ${url}`,
              ]
              ctx.ui.setWidget('wechat-qr', lines)
            })
          },
          onScanned: () => {
            ctx.ui.setStatus('wechat', '📱 Scanned — confirm in WeChat…')
          },
          onExpired: () => {
            ctx.ui.setStatus('wechat', '⏳ QR expired — new one coming…')
          },
        },
      })

      // Login success — clear QR widget
      ctx.ui.setWidget('wechat-qr', undefined)
      ctx.ui.setStatus('wechat', `✓ WeChat: ${creds.accountId}`)
      ctx.ui.notify(`WeChat connected!\nAccount: ${creds.accountId}`, 'info')
      connected = true

      // ── WeChat messages → pi prompts ─────────────────────────────
      bot.onMessage(async (msg: IncomingMessage) => {
        if (msg.type !== 'text' || !msg.text.trim()) {
          await bot!.reply(msg, `[${msg.type} received — text only for now]`)
          return
        }

        activeUserId = msg.userId
        pendingReply = msg
        isStreaming = true
        assistantText = ''

        try { await bot!.sendTyping(msg.userId) } catch {}

        ctx.ui.setStatus('wechat', `📱 ${msg.text.slice(0, 60)}`)

        // ★ Core: WeChat message → pi prompt
        pi.sendUserMessage(msg.text)
      })

      // SDK events → pi TUI status
      bot.on('error', (err) => {
        ctx.ui.setStatus('wechat', `⚠ ${err instanceof Error ? err.message : String(err)}`)
      })
      bot.on('session:expired', () => {
        ctx.ui.setStatus('wechat', '⚠ Session expired — re-login…')
      })
      bot.on('session:restored', (c) => {
        ctx.ui.setStatus('wechat', `✓ Reconnected: ${c.accountId}`)
      })

      // Start long-poll (background)
      bot.start().catch((e) => {
        ctx.ui.setStatus('wechat', `✗ Poll error: ${e instanceof Error ? e.message : e}`)
        connected = false
      })

    } catch (e) {
      ctx.ui.setWidget('wechat-qr', undefined)
      ctx.ui.setStatus('wechat', undefined)
      ctx.ui.notify(`Login failed: ${e instanceof Error ? e.message : e}`, 'error')
      bot = null
    }
  }

  // Register both /wechat and /weixin
  pi.registerCommand('wechat', {
    description: 'Connect WeChat — scan QR to chat with Pi from your phone',
    handler: startWechat,
  })

  pi.registerCommand('weixin', {
    description: 'Connect WeChat (alias for /wechat)',
    handler: startWechat,
  })

  pi.registerCommand('wechat-disconnect', {
    description: 'Disconnect WeChat',
    handler: async (_args, ctx) => {
      if (bot) { bot.stop(); bot = null }
      connected = false; activeUserId = null; pendingReply = null
      ctx.ui.setStatus('wechat', undefined)
      ctx.ui.setWidget('wechat-qr', undefined)
      ctx.ui.notify('WeChat disconnected', 'info')
    },
  })

  pi.registerCommand('wechat-send', {
    description: 'Send text to WeChat user: /wechat-send <message>',
    handler: async (args, ctx) => {
      if (!bot || !connected || !activeUserId) {
        ctx.ui.notify('Not connected. Run /wechat first.', 'error')
        return
      }
      const text = args?.trim()
      if (!text) { ctx.ui.notify('Usage: /wechat-send <message>', 'error'); return }
      try {
        await bot.send(activeUserId, text)
        ctx.ui.notify(`Sent: ${text.slice(0, 50)}…`, 'info')
      } catch (e) {
        ctx.ui.notify(`Failed: ${e instanceof Error ? e.message : e}`, 'error')
      }
    },
  })

  // Cleanup
  pi.on('session_shutdown', async () => {
    if (bot) { bot.stop(); bot = null }
    connected = false
  })

  pi.on('session_start', async (_event, ctx) => {
    if (connected && bot) {
      ctx.ui.setStatus('wechat', `✓ WeChat: ${bot.getCredentials()?.accountId ?? 'connected'}`)
    }
  })
}
